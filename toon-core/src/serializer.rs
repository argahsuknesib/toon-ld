//! TOON-LD Serializer
//!
//! This module provides the `ToonSerializer` struct for converting JSON/JSON-LD
//! values to TOON-LD format.

use once_cell::sync::Lazy;
use regex::Regex;
use serde_json::{Map, Value};
use std::collections::HashSet;

use crate::context::JsonLdContext;
use crate::error::Result;
use crate::keywords::*;

/// Regex for detecting values that need quoting
static NEEDS_QUOTE_REGEX: Lazy<Regex> =
    Lazy::new(|| Regex::new(r#"[,:|]|^\s|\s$"#).expect("NEEDS_QUOTE_REGEX is invalid"));

/// Default indentation size (2 spaces)
const DEFAULT_INDENT_SIZE: usize = 2;

/// Maximum inline array length before switching to multi-line format
const MAX_INLINE_ARRAY_LENGTH: usize = 60;

/// Sparsity threshold for enabling shape-based partitioning (30%)
/// If null cells exceed this percentage, arrays will be partitioned by shape
const SPARSITY_THRESHOLD: f64 = 0.30;

/// TOON-LD Serializer
///
/// Converts JSON/JSON-LD values to TOON-LD format. The serializer handles:
/// - Tabular arrays (arrays of objects with union-of-keys)
/// - Primitive arrays (inline or multi-line)
/// - JSON-LD keywords and context-based URI compaction
/// - Value nodes with language tags and datatypes (using standard TOON object syntax)
///
/// # Example
///
/// ```
/// use toon_core::{ToonSerializer, JsonLdContext};
/// use serde_json::json;
///
/// let serializer = ToonSerializer::new();
/// let value = json!({
///     "name": "Alice",
///     "age": 30
/// });
///
/// let toon = serializer.serialize(&value).unwrap();
/// assert!(toon.contains("name: Alice"));
/// assert!(toon.contains("age: 30"));
/// ```
#[derive(Debug, Clone)]
pub struct ToonSerializer {
    /// JSON-LD context for URI compaction
    context: JsonLdContext,
    /// Number of spaces per indentation level
    indent_size: usize,
    /// Enable shape-based partitioning for sparse arrays
    enable_shape_partitioning: bool,
}

impl Default for ToonSerializer {
    fn default() -> Self {
        Self::new()
    }
}

impl ToonSerializer {
    /// Create a new serializer with default settings.
    ///
    /// # Example
    ///
    /// ```
    /// use toon_core::ToonSerializer;
    ///
    /// let serializer = ToonSerializer::new();
    /// ```
    pub fn new() -> Self {
        Self {
            context: JsonLdContext::new(),
            indent_size: DEFAULT_INDENT_SIZE,
            enable_shape_partitioning: true,
        }
    }

    /// Set the JSON-LD context for URI compaction.
    ///
    /// # Arguments
    ///
    /// * `context` - The JSON-LD context to use
    ///
    /// # Example
    ///
    /// ```
    /// use toon_core::{ToonSerializer, JsonLdContext};
    ///
    /// let mut ctx = JsonLdContext::new();
    /// ctx.add_prefix("foaf", "http://xmlns.com/foaf/0.1/");
    ///
    /// let serializer = ToonSerializer::new().with_context(ctx);
    /// ```
    pub fn with_context(mut self, context: JsonLdContext) -> Self {
        self.context = context;
        self
    }

    /// Set the indentation size.
    ///
    /// # Arguments
    ///
    /// * `size` - Number of spaces per indentation level
    ///
    /// # Example
    ///
    /// ```
    /// use toon_core::ToonSerializer;
    ///
    /// let serializer = ToonSerializer::new().with_indent_size(4);
    /// ```
    pub fn with_indent_size(mut self, size: usize) -> Self {
        self.indent_size = size;
        self
    }

    /// Get a reference to the current context.
    pub fn context(&self) -> &JsonLdContext {
        &self.context
    }

    /// Get the current indentation size.
    pub fn indent_size(&self) -> usize {
        self.indent_size
    }

    /// Enable or disable shape-based partitioning for sparse arrays.
    ///
    /// When enabled, arrays with high sparsity (> 30% null values) will be
    /// automatically partitioned by entity shape to reduce null delimiter overhead.
    ///
    /// # Arguments
    ///
    /// * `enable` - Whether to enable shape-based partitioning
    ///
    /// # Example
    ///
    /// ```
    /// use toon_core::ToonSerializer;
    ///
    /// let serializer = ToonSerializer::new().with_shape_partitioning(true);
    /// ```
    pub fn with_shape_partitioning(mut self, enable: bool) -> Self {
        self.enable_shape_partitioning = enable;
        self
    }

    /// Serialize a JSON value to TOON-LD format.
    ///
    /// # Arguments
    ///
    /// * `value` - The JSON value to serialize
    ///
    /// # Returns
    ///
    /// A `Result` containing the TOON-LD string or an error.
    ///
    /// # Example
    ///
    /// ```
    /// use toon_core::ToonSerializer;
    /// use serde_json::json;
    ///
    /// let serializer = ToonSerializer::new();
    /// let value = json!({"name": "Alice", "age": 30});
    /// let toon = serializer.serialize(&value).unwrap();
    /// ```
    pub fn serialize(&self, value: &Value) -> Result<String> {
        let mut output = String::new();
        self.serialize_value(value, 0, &mut output)?;
        Ok(output)
    }

    /// Serialize a JSON string to TOON-LD string.
    ///
    /// # Arguments
    ///
    /// * `json` - A JSON string to parse and serialize
    ///
    /// # Returns
    ///
    /// A `Result` containing the TOON-LD string or an error.
    ///
    /// # Example
    ///
    /// ```
    /// use toon_core::ToonSerializer;
    ///
    /// let serializer = ToonSerializer::new();
    /// let toon = serializer.serialize_json(r#"{"name": "Alice"}"#).unwrap();
    /// ```
    pub fn serialize_json(&self, json: &str) -> Result<String> {
        let value: Value = serde_json::from_str(json)?;
        self.serialize(&value)
    }

    /// Serialize a JSON value at a given depth.
    fn serialize_value(&self, value: &Value, depth: usize, output: &mut String) -> Result<()> {
        match value {
            Value::Null => output.push_str("null"),
            Value::Bool(b) => output.push_str(if *b { "true" } else { "false" }),
            Value::Number(n) => output.push_str(&n.to_string()),
            Value::String(s) => output.push_str(&self.quote_if_needed(s)),
            Value::Array(arr) => self.serialize_standalone_array(arr, depth, output)?,
            Value::Object(obj) => self.serialize_object(obj, depth, output)?,
        }
        Ok(())
    }

    /// Serialize a standalone array (without a key, e.g., top-level array).
    fn serialize_standalone_array(
        &self,
        arr: &[Value],
        depth: usize,
        output: &mut String,
    ) -> Result<()> {
        let indent = self.make_indent(depth);

        if arr.is_empty() {
            output.push_str("[]");
            return Ok(());
        }

        // Check if this is an array of objects (can use tabular format)
        if let Some(fields) = self.get_tabular_fields(arr) {
            // Use anonymous tabular format
            let compact_fields: Vec<String> =
                fields.iter().map(|f| self.context.compact_uri(f)).collect();
            output.push_str(&format!(
                "[{}]{{{}}}:\n",
                arr.len(),
                compact_fields.join(",")
            ));
            let row_indent = self.make_indent(depth + 1);
            for item in arr {
                if let Value::Object(obj) = item {
                    let values: Vec<String> = fields
                        .iter()
                        .map(|field| {
                            obj.get(field)
                                .map(|v| self.value_to_csv_cell(v))
                                .unwrap_or_else(|| "null".to_string())
                        })
                        .collect();
                    output.push_str(&format!("{}{}\n", row_indent, values.join(", ")));
                }
            }
        } else if self.is_primitive_array(arr) {
            self.serialize_inline_primitive_array(arr, depth, output)?;
        } else {
            // Mixed array
            output.push_str(&format!("{}[{}]:\n", indent, arr.len()));
            for item in arr {
                let item_indent = self.make_indent(depth + 1);
                output.push_str(&item_indent);
                output.push_str("- ");
                match item {
                    Value::Object(obj) => {
                        output.push('\n');
                        self.serialize_object(obj, depth + 2, output)?;
                    }
                    _ => {
                        self.serialize_value(item, depth + 1, output)?;
                        output.push('\n');
                    }
                }
            }
        }
        Ok(())
    }

    /// Serialize an inline primitive array.
    fn serialize_inline_primitive_array(
        &self,
        arr: &[Value],
        depth: usize,
        output: &mut String,
    ) -> Result<()> {
        let values: Vec<String> = arr.iter().map(|v| self.value_to_csv_cell(v)).collect();
        let inline = values.join(", ");

        if inline.len() < MAX_INLINE_ARRAY_LENGTH {
            output.push_str(&format!("[{}]: {}", arr.len(), inline));
        } else {
            output.push_str(&format!("[{}]:\n", arr.len()));
            let row_indent = self.make_indent(depth + 1);
            for value in &values {
                output.push_str(&format!("{}{}\n", row_indent, value));
            }
        }
        Ok(())
    }

    /// Serialize a JSON object.
    fn serialize_object(
        &self,
        obj: &Map<String, Value>,
        depth: usize,
        output: &mut String,
    ) -> Result<()> {
        let indent = self.make_indent(depth);

        // Sort keys by keyword order, then alphabetically
        let mut keys: Vec<&String> = obj.keys().collect();
        keys.sort_by(|a, b| {
            keyword_order(a)
                .cmp(&keyword_order(b))
                .then_with(|| a.cmp(b))
        });

        for key in keys {
            // Safe: we're iterating over keys that exist
            let value = obj.get(key).expect("key exists in object we're iterating");
            self.serialize_object_entry(key, value, depth, &indent, output)?;
        }
        Ok(())
    }

    /// Serialize a single object entry (key-value pair).
    fn serialize_object_entry(
        &self,
        key: &str,
        value: &Value,
        depth: usize,
        indent: &str,
        output: &mut String,
    ) -> Result<()> {
        let display_key = self.get_display_key(key);

        match key {
            // Special handling for @graph - always use tabular if possible
            JSONLD_GRAPH => {
                if let Value::Array(arr) = value {
                    self.serialize_keyed_array(&display_key, arr, depth, output)?;
                } else {
                    output.push_str(&format!("{}{}:\n", indent, display_key));
                    self.serialize_value(value, depth + 1, output)?;
                }
            }
            // @context gets special nested formatting
            JSONLD_CONTEXT => {
                self.serialize_context(value, depth, output)?;
            }
            // @base and @vocab are simple string values
            JSONLD_BASE | JSONLD_VOCAB => {
                output.push_str(&format!("{}{}: ", indent, display_key));
                self.serialize_value(value, depth, output)?;
                output.push('\n');
            }
            // @id uses simple serialization
            JSONLD_ID => match value {
                Value::Array(arr) => {
                    self.serialize_keyed_array(&display_key, arr, depth, output)?;
                }
                _ => {
                    output.push_str(&format!("{}{}: ", indent, display_key));
                    self.serialize_value(value, depth, output)?;
                    output.push('\n');
                }
            },
            // @type values should be compacted with context
            JSONLD_TYPE => match value {
                Value::Array(arr) => {
                    self.serialize_keyed_array(&display_key, arr, depth, output)?;
                }
                Value::String(s) => {
                    // Compact the type URI using context
                    let compact_type = self.context.compact_uri(s);
                    output.push_str(&format!("{}{}: {}\n", indent, display_key, compact_type));
                }
                _ => {
                    output.push_str(&format!("{}{}: ", indent, display_key));
                    self.serialize_value(value, depth, output)?;
                    output.push('\n');
                }
            },
            // @reverse contains nested object with reverse properties
            JSONLD_REVERSE => {
                output.push_str(&format!("{}{}:\n", indent, TOON_REVERSE));
                if let Value::Object(rev_obj) = value {
                    self.serialize_object(rev_obj, depth + 1, output)?;
                }
            }
            // @list is an ordered array
            JSONLD_LIST => {
                if let Value::Array(arr) = value {
                    self.serialize_keyed_array(TOON_LIST, arr, depth, output)?;
                }
            }
            // @set is an explicit unordered set
            JSONLD_SET => {
                if let Value::Array(arr) = value {
                    self.serialize_keyed_array(TOON_SET, arr, depth, output)?;
                }
            }
            // @value, @language - serialize as normal keys (value nodes use standard TOON object syntax)
            // Note: @type is handled above as it can be either a node type or a value node datatype
            JSONLD_VALUE | JSONLD_LANGUAGE => {
                output.push_str(&format!("{}{}: ", indent, display_key));
                self.serialize_value(value, depth, output)?;
                output.push('\n');
            }
            // @included contains an array of included nodes
            JSONLD_INCLUDED => {
                if let Value::Array(arr) = value {
                    self.serialize_keyed_array(TOON_INCLUDED, arr, depth, output)?;
                } else {
                    output.push_str(&format!("{}{}:\n", indent, TOON_INCLUDED));
                    self.serialize_value(value, depth + 1, output)?;
                }
            }
            // @index is a simple string value
            JSONLD_INDEX => {
                output.push_str(&format!("{}{}: ", indent, TOON_INDEX));
                self.serialize_value(value, depth, output)?;
                output.push('\n');
            }
            // @nest contains nested properties object
            JSONLD_NEST => {
                output.push_str(&format!("{}{}:\n", indent, TOON_NEST));
                if let Value::Object(nest_obj) = value {
                    self.serialize_object(nest_obj, depth + 1, output)?;
                }
            }
            // @container specifies container type
            JSONLD_CONTAINER => match value {
                Value::Array(arr) => {
                    self.serialize_keyed_array(TOON_CONTAINER, arr, depth, output)?;
                }
                _ => {
                    output.push_str(&format!("{}{}: ", indent, TOON_CONTAINER));
                    self.serialize_value(value, depth, output)?;
                    output.push('\n');
                }
            },
            // @direction specifies text direction (ltr/rtl)
            JSONLD_DIRECTION => {
                output.push_str(&format!("{}{}: ", indent, TOON_DIRECTION));
                self.serialize_value(value, depth, output)?;
                output.push('\n');
            }
            // @import specifies external context to import
            JSONLD_IMPORT => {
                output.push_str(&format!("{}{}: ", indent, TOON_IMPORT));
                self.serialize_value(value, depth, output)?;
                output.push('\n');
            }
            // @json marks a JSON literal
            JSONLD_JSON => {
                output.push_str(&format!("{}{}: ", indent, TOON_JSON));
                // Serialize as JSON string
                let json_str = serde_json::to_string(value).unwrap_or_else(|_| "null".to_string());
                output.push_str(&format!("\"{}\"", json_str.replace('"', "\\\"")));
                output.push('\n');
            }
            // @none is the default index value
            JSONLD_NONE => {
                output.push_str(&format!("{}{}: ", indent, TOON_NONE));
                self.serialize_value(value, depth, output)?;
                output.push('\n');
            }
            // @prefix flag
            JSONLD_PREFIX => {
                output.push_str(&format!("{}{}: ", indent, TOON_PREFIX));
                self.serialize_value(value, depth, output)?;
                output.push('\n');
            }
            // @propagate flag
            JSONLD_PROPAGATE => {
                output.push_str(&format!("{}{}: ", indent, TOON_PROPAGATE));
                self.serialize_value(value, depth, output)?;
                output.push('\n');
            }
            // @protected flag
            JSONLD_PROTECTED => {
                output.push_str(&format!("{}{}: ", indent, TOON_PROTECTED));
                self.serialize_value(value, depth, output)?;
                output.push('\n');
            }
            // @version specifies JSON-LD version
            JSONLD_VERSION => {
                output.push_str(&format!("{}{}: ", indent, TOON_VERSION));
                self.serialize_value(value, depth, output)?;
                output.push('\n');
            }
            // Regular keys
            _ => {
                let compact_key = self.context.compact_uri(key);
                match value {
                    Value::Array(arr) => {
                        self.serialize_keyed_array(&compact_key, arr, depth, output)?;
                    }
                    Value::Object(nested) => {
                        output.push_str(&format!("{}{}:\n", indent, compact_key));
                        self.serialize_object(nested, depth + 1, output)?;
                    }
                    _ => {
                        output.push_str(&format!("{}{}: ", indent, compact_key));
                        self.serialize_value(value, depth, output)?;
                        output.push('\n');
                    }
                }
            }
        }
        Ok(())
    }

    /// Get display key for JSON-LD keywords.
    fn get_display_key(&self, key: &str) -> String {
        // Use the centralized keyword function, but fall back to context compaction
        // for non-keyword keys
        if let Some(toon_key) = get_toon_keyword(key) {
            toon_key.to_string()
        } else {
            self.context.compact_uri(key)
        }
    }

    /// Serialize @context in a compact format.
    fn serialize_context(&self, value: &Value, depth: usize, output: &mut String) -> Result<()> {
        let indent = self.make_indent(depth);
        output.push_str(&format!("{}{}:\n", indent, JSONLD_CONTEXT));

        match value {
            Value::Object(ctx) => {
                let ctx_indent = self.make_indent(depth + 1);
                for (prefix, uri) in ctx {
                    output.push_str(&format!("{}{}: ", ctx_indent, prefix));
                    self.serialize_value(uri, depth + 1, output)?;
                    output.push('\n');
                }
            }
            Value::Array(arr) => {
                // Multiple contexts
                for item in arr {
                    self.serialize_context(item, depth + 1, output)?;
                }
            }
            Value::String(s) => {
                let ctx_indent = self.make_indent(depth + 1);
                output.push_str(&format!("{}{}\n", ctx_indent, self.quote_if_needed(s)));
            }
            _ => {
                self.serialize_value(value, depth + 1, output)?;
                output.push('\n');
            }
        }
        Ok(())
    }

    /// Serialize a keyed array (array with a key prefix).
    pub fn serialize_keyed_array(
        &self,
        key: &str,
        arr: &[Value],
        depth: usize,
        output: &mut String,
    ) -> Result<()> {
        let indent = self.make_indent(depth);

        if arr.is_empty() {
            output.push_str(&format!("{}{}[0]:\n", indent, key));
            return Ok(());
        }

        // Check if this is an array of objects (can use tabular format)
        if let Some(fields) = self.get_tabular_fields(arr) {
            // Calculate sparsity and decide whether to partition
            if self.enable_shape_partitioning {
                let sparsity = self.calculate_sparsity(arr, &fields);

                // If sparsity exceeds threshold, use shape-based partitioning
                if sparsity > SPARSITY_THRESHOLD {
                    return self.serialize_partitioned_array(key, arr, depth, output);
                }
            }

            // Otherwise use standard union schema approach
            self.serialize_tabular_array(key, arr, &fields, depth, output)?;
        } else if self.is_primitive_array(arr) {
            self.serialize_primitive_array(key, arr, depth, output)?;
        } else {
            // Mixed array - serialize each element indented
            output.push_str(&format!("{}{}[{}]:\n", indent, key, arr.len()));
            for item in arr {
                let item_indent = self.make_indent(depth + 1);
                output.push_str(&item_indent);
                output.push_str("- ");
                match item {
                    Value::Object(obj) => {
                        output.push('\n');
                        self.serialize_object(obj, depth + 2, output)?;
                    }
                    _ => {
                        self.serialize_value(item, depth + 1, output)?;
                        output.push('\n');
                    }
                }
            }
        }
        Ok(())
    }

    /// Get tabular fields from an array of objects.
    ///
    /// Returns `Some(fields)` with the union of all keys if all elements are objects.
    /// Missing fields in individual objects will be filled with null during serialization.
    fn get_tabular_fields(&self, arr: &[Value]) -> Option<Vec<String>> {
        if arr.is_empty() {
            return None;
        }

        // Collect union of all keys from all objects
        let mut all_keys: HashSet<String> = HashSet::new();

        for item in arr {
            match item {
                Value::Object(obj) => {
                    for key in obj.keys() {
                        all_keys.insert(key.clone());
                    }
                }
                // If any element is not an object, cannot use tabular format
                _ => return None,
            }
        }

        if all_keys.is_empty() {
            return None;
        }

        // Return keys in consistent order (sorted, with keywords first)
        let mut fields: Vec<String> = all_keys.into_iter().collect();
        fields.sort_by(|a, b| {
            keyword_order(a)
                .cmp(&keyword_order(b))
                .then_with(|| a.cmp(b))
        });
        Some(fields)
    }

    /// Check if array contains only primitives.
    fn is_primitive_array(&self, arr: &[Value]) -> bool {
        arr.iter().all(|v| {
            matches!(
                v,
                Value::Null | Value::Bool(_) | Value::Number(_) | Value::String(_)
            )
        })
    }

    /// Serialize a tabular array: key[N]{field1,field2}:
    fn serialize_tabular_array(
        &self,
        key: &str,
        arr: &[Value],
        fields: &[String],
        depth: usize,
        output: &mut String,
    ) -> Result<()> {
        let indent = self.make_indent(depth);
        let row_indent = self.make_indent(depth + 1);

        // Compact field names
        let compact_fields: Vec<String> =
            fields.iter().map(|f| self.context.compact_uri(f)).collect();

        // Write header: key[N]{field1,field2}:
        output.push_str(&format!(
            "{}{}[{}]{{{}}}:\n",
            indent,
            key,
            arr.len(),
            compact_fields.join(",")
        ));

        // Write CSV rows
        for item in arr {
            if let Value::Object(obj) = item {
                let values: Vec<String> = fields
                    .iter()
                    .map(|field| {
                        obj.get(field)
                            .map(|v| self.value_to_csv_cell(v))
                            .unwrap_or_else(|| "null".to_string())
                    })
                    .collect();
                output.push_str(&format!("{}{}\n", row_indent, values.join(", ")));
            }
        }

        Ok(())
    }

    /// Serialize a primitive array.
    fn serialize_primitive_array(
        &self,
        key: &str,
        arr: &[Value],
        depth: usize,
        output: &mut String,
    ) -> Result<()> {
        let indent = self.make_indent(depth);

        let values: Vec<String> = arr.iter().map(|v| self.value_to_csv_cell(v)).collect();
        let inline = values.join(", ");

        // If short enough, keep on one line
        if inline.len() < MAX_INLINE_ARRAY_LENGTH {
            output.push_str(&format!("{}{}[{}]: {}\n", indent, key, arr.len(), inline));
        } else {
            // Multi-line format
            output.push_str(&format!("{}{}[{}]:\n", indent, key, arr.len()));
            let row_indent = self.make_indent(depth + 1);
            for value in &values {
                output.push_str(&format!("{}{}\n", row_indent, value));
            }
        }

        Ok(())
    }

    /// Convert a value to a CSV cell string.
    fn value_to_csv_cell(&self, value: &Value) -> String {
        match value {
            Value::Null => "null".to_string(),
            Value::Bool(b) => if *b { "true" } else { "false" }.to_string(),
            Value::Number(n) => n.to_string(),
            Value::String(s) => self.quote_if_needed(s),
            Value::Array(_) | Value::Object(_) => {
                // Nested structures in CSV cells - serialize as JSON and quote
                let json = serde_json::to_string(value).unwrap_or_else(|_| "null".to_string());
                format!("\"{}\"", json.replace('"', "\\\""))
            }
        }
    }

    /// Quote a string if it contains special characters.
    fn quote_if_needed(&self, s: &str) -> String {
        if s.is_empty() {
            return "\"\"".to_string();
        }
        if NEEDS_QUOTE_REGEX.is_match(s) {
            format!("\"{}\"", s.replace('"', "\\\""))
        } else {
            s.to_string()
        }
    }

    /// Create indentation string for given depth.
    #[inline]
    fn make_indent(&self, depth: usize) -> String {
        " ".repeat(depth * self.indent_size)
    }

    /// Calculate sparsity of an array with given fields.
    /// Returns the ratio of null values to total cells.
    fn calculate_sparsity(&self, arr: &[Value], fields: &[String]) -> f64 {
        if arr.is_empty() || fields.is_empty() {
            return 0.0;
        }

        let mut null_count = 0;
        let total_cells = arr.len() * fields.len();

        for item in arr {
            if let Value::Object(obj) = item {
                for field in fields {
                    if !obj.contains_key(field) {
                        null_count += 1;
                    }
                }
            }
        }

        null_count as f64 / total_cells as f64
    }

    /// Generate a deterministic signature for an entity based on its keys.
    /// Keys are sorted alphabetically to ensure consistency.
    fn entity_signature(&self, obj: &Map<String, Value>) -> String {
        let mut keys: Vec<&String> = obj.keys().collect();
        keys.sort();
        keys.into_iter()
            .map(|k| k.as_str())
            .collect::<Vec<&str>>()
            .join("|")
    }

    /// Partition array entities by their shape signature.
    /// Returns a Vec of (signature, fields, entities) tuples.
    fn partition_by_shape<'a>(
        &self,
        arr: &'a [Value],
    ) -> Vec<(String, Vec<String>, Vec<&'a Value>)> {
        use std::collections::HashMap;

        let mut shape_map: HashMap<String, Vec<&Value>> = HashMap::new();

        // Group entities by signature
        for item in arr {
            if let Value::Object(obj) = item {
                let sig = self.entity_signature(obj);
                shape_map.entry(sig).or_insert_with(Vec::new).push(item);
            }
        }

        // Convert to sorted output format
        let mut partitions: Vec<(String, Vec<String>, Vec<&Value>)> = shape_map
            .into_iter()
            .map(|(sig, entities)| {
                let fields: Vec<String> = sig.split('|').map(String::from).collect();
                (sig, fields, entities)
            })
            .collect();

        // Sort by entity count (largest groups first) for better readability
        partitions.sort_by(|a, b| b.2.len().cmp(&a.2.len()));

        partitions
    }

    /// Serialize a keyed array using shape-based partitioning.
    /// Emits multiple array blocks, each with entities of the same shape.
    fn serialize_partitioned_array(
        &self,
        key: &str,
        arr: &[Value],
        depth: usize,
        output: &mut String,
    ) -> Result<()> {
        let partitions = self.partition_by_shape(arr);
        let indent = self.make_indent(depth);
        let row_indent = self.make_indent(depth + 1);

        for (idx, (_sig, fields, entities)) in partitions.iter().enumerate() {
            // Add spacing between partitions (except before first)
            if idx > 0 {
                output.push('\n');
            }

            // Compact field names
            let compact_fields: Vec<String> =
                fields.iter().map(|f| self.context.compact_uri(f)).collect();

            // Write header: key[N]{field1,field2}:
            output.push_str(&format!(
                "{}{}[{}]{{{}}}:\n",
                indent,
                key,
                entities.len(),
                compact_fields.join(",")
            ));

            // Write CSV rows
            for entity in entities {
                if let Value::Object(obj) = entity {
                    let values: Vec<String> = fields
                        .iter()
                        .map(|field| {
                            obj.get(field)
                                .map(|v| self.value_to_csv_cell(v))
                                .unwrap_or_else(|| "null".to_string())
                        })
                        .collect();
                    output.push_str(&format!("{}{}\n", row_indent, values.join(", ")));
                }
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_new_serializer() {
        let serializer = ToonSerializer::new();
        assert_eq!(serializer.indent_size(), DEFAULT_INDENT_SIZE);
        assert!(serializer.context().is_empty());
    }

    #[test]
    fn test_with_indent_size() {
        let serializer = ToonSerializer::new().with_indent_size(4);
        assert_eq!(serializer.indent_size(), 4);
    }

    #[test]
    fn test_with_context() {
        let mut ctx = JsonLdContext::new();
        ctx.add_prefix("foaf", "http://xmlns.com/foaf/0.1/");

        let serializer = ToonSerializer::new().with_context(ctx);
        assert!(serializer.context().has_prefixes());
    }

    #[test]
    fn test_serialize_primitives() {
        let serializer = ToonSerializer::new();

        let value = json!({
            "name": "Alice",
            "age": 30,
            "active": true,
            "score": null
        });

        let toon = serializer.serialize(&value).unwrap();
        assert!(toon.contains("name: Alice"));
        assert!(toon.contains("age: 30"));
        assert!(toon.contains("active: true"));
        assert!(toon.contains("score: null"));
    }

    #[test]
    fn test_serialize_primitive_array() {
        let serializer = ToonSerializer::new();

        let value = json!({
            "tags": ["rust", "wasm", "python"]
        });

        let toon = serializer.serialize(&value).unwrap();
        assert!(toon.contains("tags[3]:"));
        assert!(toon.contains("rust"));
    }

    #[test]
    fn test_serialize_tabular_array() {
        let serializer = ToonSerializer::new();

        let value = json!({
            "people": [
                {"name": "Alice", "age": 30},
                {"name": "Bob", "age": 25}
            ]
        });

        let toon = serializer.serialize(&value).unwrap();
        assert!(toon.contains("people[2]{"));
        assert!(toon.contains("Alice"));
        assert!(toon.contains("Bob"));
    }

    #[test]
    fn test_serialize_empty_array() {
        let serializer = ToonSerializer::new();

        let value = json!({
            "items": []
        });

        let toon = serializer.serialize(&value).unwrap();
        assert!(toon.contains("items[0]:"));
    }

    #[test]
    fn test_serialize_nested_object() {
        let serializer = ToonSerializer::new();

        let value = json!({
            "person": {
                "name": "Alice",
                "address": {
                    "city": "Seattle"
                }
            }
        });

        let toon = serializer.serialize(&value).unwrap();
        assert!(toon.contains("person:"));
        assert!(toon.contains("address:"));
        assert!(toon.contains("city: Seattle"));
    }

    #[test]
    fn test_quote_if_needed() {
        let serializer = ToonSerializer::new();

        assert_eq!(serializer.quote_if_needed("hello"), "hello");
        assert_eq!(
            serializer.quote_if_needed("hello, world"),
            "\"hello, world\""
        );
        assert_eq!(serializer.quote_if_needed("key: value"), "\"key: value\"");
        assert_eq!(serializer.quote_if_needed("a|b"), "\"a|b\"");
        assert_eq!(serializer.quote_if_needed(""), "\"\"");
        assert_eq!(serializer.quote_if_needed(" leading"), "\" leading\"");
        assert_eq!(serializer.quote_if_needed("trailing "), "\"trailing \"");
    }

    #[test]
    fn test_serialize_with_context_compaction() {
        let mut ctx = JsonLdContext::new();
        ctx.add_prefix("foaf", "http://xmlns.com/foaf/0.1/");

        let serializer = ToonSerializer::new().with_context(ctx);

        let value = json!({
            "http://xmlns.com/foaf/0.1/name": "Alice"
        });

        let toon = serializer.serialize(&value).unwrap();
        assert!(toon.contains("foaf:name"));
    }

    #[test]
    fn test_serialize_value_node_with_language() {
        let serializer = ToonSerializer::new();

        let value = json!({
            "title": {
                "@value": "Bonjour",
                "@language": "fr"
            }
        });

        let toon = serializer.serialize(&value).unwrap();
        // Value nodes now use standard TOON object syntax
        assert!(toon.contains("@value"));
        assert!(toon.contains("Bonjour"));
        assert!(toon.contains("@language"));
        assert!(toon.contains("fr"));
    }

    #[test]
    fn test_serialize_value_node_with_type() {
        let mut ctx = JsonLdContext::new();
        ctx.add_prefix("xsd", "http://www.w3.org/2001/XMLSchema#");

        let serializer = ToonSerializer::new().with_context(ctx);

        let value = json!({
            "date": {
                "@value": "2024-01-15",
                "@type": "http://www.w3.org/2001/XMLSchema#date"
            }
        });

        let toon = serializer.serialize(&value).unwrap();
        // Value nodes now use standard TOON object syntax
        assert!(toon.contains("@value"));
        assert!(toon.contains("2024-01-15"));
        assert!(toon.contains("@type"));
        assert!(toon.contains("xsd:date"));
    }

    #[test]
    fn test_serialize_context() {
        let serializer = ToonSerializer::new();

        let value = json!({
            "@context": {
                "foaf": "http://xmlns.com/foaf/0.1/",
                "schema": "http://schema.org/"
            },
            "name": "Test"
        });

        let toon = serializer.serialize(&value).unwrap();
        assert!(toon.contains("@context:"));
        assert!(toon.contains("foaf:"));
        assert!(toon.contains("schema:"));
    }

    #[test]
    fn test_serialize_graph() {
        let serializer = ToonSerializer::new();

        let value = json!({
            "@graph": [
                {"@id": "ex:1", "name": "Alice"},
                {"@id": "ex:2", "name": "Bob"}
            ]
        });

        let toon = serializer.serialize(&value).unwrap();
        assert!(toon.contains("@graph[2]"));
    }

    #[test]
    fn test_serialize_json_string() {
        let serializer = ToonSerializer::new();

        let toon = serializer
            .serialize_json(r#"{"name": "Alice", "age": 30}"#)
            .unwrap();
        assert!(toon.contains("name: Alice"));
        assert!(toon.contains("age: 30"));
    }

    #[test]
    fn test_tabular_array_union_of_keys() {
        // Disable partitioning to test union schema explicitly
        let serializer = ToonSerializer::new().with_shape_partitioning(false);

        let value = json!({
            "items": [
                {"a": 1, "b": 2},
                {"a": 3, "c": 4}
            ]
        });

        let toon = serializer.serialize(&value).unwrap();
        // Should have union of keys
        assert!(toon.contains("items[2]{a,b,c}:"));
        // Missing fields should be null
        assert!(toon.contains("1, 2, null"));
        assert!(toon.contains("3, null, 4"));
    }

    #[test]
    fn test_shape_partitioning_disabled() {
        // Test with partitioning disabled - should use union schema
        let serializer = ToonSerializer::new().with_shape_partitioning(false);

        let value = json!({
            "items": [
                {"a": 1, "b": 2},
                {"a": 3, "c": 4},
                {"x": 5, "y": 6}
            ]
        });

        let toon = serializer.serialize(&value).unwrap();
        // Should use union schema even with high sparsity
        assert!(toon.contains("items[3]{a,b,c,x,y}:"));
    }

    #[test]
    fn test_shape_partitioning_low_sparsity() {
        // Test with low sparsity - should NOT partition
        let serializer = ToonSerializer::new();

        let value = json!({
            "items": [
                {"a": 1, "b": 2},
                {"a": 3, "b": 4},
                {"a": 5, "b": 6}
            ]
        });

        let toon = serializer.serialize(&value).unwrap();
        // Low sparsity - should use single table
        assert!(toon.contains("items[3]{a,b}:"));
        assert!(!toon.contains("items[1]")); // No partitions
    }

    #[test]
    fn test_shape_partitioning_high_sparsity() {
        // Test with high sparsity - SHOULD partition
        let serializer = ToonSerializer::new();

        let value = json!({
            "people": [
                {"@id": "ex:1", "name": "Alice", "age": 30, "email": "alice@example.com"},
                {"@id": "ex:2", "name": "Bob", "phone": "+1234567890", "address": "123 Main St"},
                {"@id": "ex:3", "name": "Carol", "company": "ACME", "role": "Engineer", "salary": 100000}
            ]
        });

        let toon = serializer.serialize(&value).unwrap();

        // High sparsity - should partition into separate blocks
        // Each entity has completely different fields, so they should be separate
        assert!(
            toon.contains("people[1]"),
            "Should have partitioned blocks with [1]"
        );

        // Count how many people[] blocks we have
        let people_blocks = toon.matches("people[").count();
        assert_eq!(
            people_blocks, 3,
            "Should have 3 separate blocks (one per entity) due to completely different shapes"
        );

        // Should NOT have the union of all keys in one header
        assert!(
            !toon.contains("people[3]"),
            "Should not have a single block with all 3 entities"
        );
    }

    #[test]
    fn test_shape_partitioning_heterogeneous_graph() {
        let serializer = ToonSerializer::new();

        let value = json!({
            "@graph": [
                {"@id": "ex:person1", "@type": "Person", "name": "Alice", "age": 30, "email": "alice@example.com"},
                {"@id": "ex:person2", "@type": "Person", "name": "Bob", "age": 25, "email": "bob@example.com"},
                {"@id": "ex:org1", "@type": "Organization", "name": "ACME", "industry": "Tech", "founded": 2000, "employees": 500, "revenue": 10000000},
                {"@id": "ex:org2", "@type": "Organization", "name": "XYZ", "industry": "Finance", "founded": 1995, "employees": 300, "revenue": 5000000}
            ]
        });

        let toon = serializer.serialize(&value).unwrap();
        // Should partition by shape
        assert!(toon.contains("@graph[2]"));
        // Should have separate blocks for Person and Organization
        let graph_count = toon.matches("@graph[").count();
        assert_eq!(graph_count, 2, "Should have 2 @graph blocks");
    }

    #[test]
    fn test_calculate_sparsity() {
        let serializer = ToonSerializer::new();

        // Test high sparsity
        let high_sparse = vec![json!({"a": 1}), json!({"b": 2}), json!({"c": 3})];
        let fields = vec!["a".to_string(), "b".to_string(), "c".to_string()];
        let sparsity = serializer.calculate_sparsity(&high_sparse, &fields);
        assert!(sparsity > 0.6, "Should have high sparsity (~66%)");

        // Test low sparsity
        let low_sparse = vec![json!({"a": 1, "b": 2}), json!({"a": 3, "b": 4})];
        let fields = vec!["a".to_string(), "b".to_string()];
        let sparsity = serializer.calculate_sparsity(&low_sparse, &fields);
        assert_eq!(sparsity, 0.0, "Should have zero sparsity");
    }

    #[test]
    fn test_entity_signature() {
        let serializer = ToonSerializer::new();

        let obj1 =
            serde_json::from_str::<Map<String, Value>>(r#"{"name": "Alice", "age": 30}"#).unwrap();
        let obj2 =
            serde_json::from_str::<Map<String, Value>>(r#"{"age": 30, "name": "Bob"}"#).unwrap();
        let obj3 = serde_json::from_str::<Map<String, Value>>(
            r#"{"name": "Carol", "email": "c@example.com"}"#,
        )
        .unwrap();

        let sig1 = serializer.entity_signature(&obj1);
        let sig2 = serializer.entity_signature(&obj2);
        let sig3 = serializer.entity_signature(&obj3);

        // Same keys should produce same signature (order independent)
        assert_eq!(sig1, sig2);
        assert_eq!(sig1, "age|name");

        // Different keys should produce different signature
        assert_ne!(sig1, sig3);
        assert_eq!(sig3, "email|name");
    }

    #[test]
    fn test_partition_by_shape() {
        let serializer = ToonSerializer::new();

        let arr = vec![
            json!({"a": 1, "b": 2}),
            json!({"a": 3, "b": 4}),
            json!({"x": 5, "y": 6}),
            json!({"x": 7, "y": 8}),
            json!({"x": 9, "y": 10}),
        ];

        let partitions = serializer.partition_by_shape(&arr);

        // Should have 2 partitions
        assert_eq!(partitions.len(), 2);

        // Largest partition should come first (3 entities with x,y)
        assert_eq!(partitions[0].2.len(), 3);
        assert_eq!(partitions[1].2.len(), 2);
    }

    #[test]
    fn test_shape_partitioning_roundtrip() {
        use crate::ToonParser;

        let serializer = ToonSerializer::new();
        let parser = ToonParser::new();

        let original = json!({
            "@graph": [
                {"@id": "ex:1", "@type": "Person", "name": "Alice", "age": 30},
                {"@id": "ex:2", "@type": "Person", "name": "Bob", "age": 25},
                {"@id": "ex:3", "@type": "Org", "name": "ACME", "industry": "Tech"}
            ]
        });

        // Serialize with partitioning
        let toon = serializer.serialize(&original).unwrap();

        // Parse back
        let parsed = parser.parse(&toon).unwrap();

        // Should have @graph array with all 3 entities
        let graph = parsed.get("@graph").expect("Should have @graph");
        assert!(graph.is_array());
        let graph_arr = graph.as_array().unwrap();
        assert_eq!(
            graph_arr.len(),
            3,
            "Should have all 3 entities after parsing"
        );

        // Verify all entities are present
        assert!(graph_arr
            .iter()
            .any(|v| v.get("@id").and_then(|id| id.as_str()) == Some("ex:1")));
        assert!(graph_arr
            .iter()
            .any(|v| v.get("@id").and_then(|id| id.as_str()) == Some("ex:2")));
        assert!(graph_arr
            .iter()
            .any(|v| v.get("@id").and_then(|id| id.as_str()) == Some("ex:3")));
    }
}
