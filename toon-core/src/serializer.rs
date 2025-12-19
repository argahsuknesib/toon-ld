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

/// TOON-LD Serializer
///
/// Converts JSON/JSON-LD values to TOON-LD format. The serializer handles:
/// - Tabular arrays (arrays of objects with union-of-keys)
/// - Primitive arrays (inline or multi-line)
/// - JSON-LD keywords and context-based URI compaction
/// - Value nodes with language tags and datatypes
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

        // Check if this is a @value node - handle it specially upfront
        if obj.contains_key(JSONLD_VALUE) {
            return self.serialize_value_node(obj, depth, output);
        }

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
            // @id and @type use shorthand for primitives, but arrays need proper handling
            JSONLD_ID | JSONLD_TYPE => match value {
                Value::Array(arr) => {
                    self.serialize_keyed_array(&display_key, arr, depth, output)?;
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
            // @value node - already handled above
            JSONLD_VALUE => {}
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

    /// Serialize a @value node in compact form: "value"@lang or "value"^^type
    fn serialize_value_node(
        &self,
        obj: &Map<String, Value>,
        depth: usize,
        output: &mut String,
    ) -> Result<()> {
        let indent = self.make_indent(depth);

        let value = obj.get(JSONLD_VALUE);
        let language = obj.get(JSONLD_LANGUAGE);
        let type_val = obj.get(JSONLD_TYPE);
        let direction = obj.get(JSONLD_DIRECTION);

        if let Some(val) = value {
            let val_str = match val {
                Value::String(s) => self.quote_if_needed(s),
                _ => self.value_to_csv_cell(val),
            };

            if let Some(Value::String(lang)) = language {
                // Language-tagged string: "value"@lang or "value"@lang:dir
                if let Some(Value::String(dir)) = direction {
                    output.push_str(&format!("{}{}@{}:{}\n", indent, val_str, lang, dir));
                } else {
                    output.push_str(&format!("{}{}@{}\n", indent, val_str, lang));
                }
            } else if let Some(Value::String(typ)) = type_val {
                // Typed literal: "value"^^type
                let compact_type = self.context.compact_uri(typ);
                output.push_str(&format!("{}{}^^{}\n", indent, val_str, compact_type));
            } else if let Some(Value::String(dir)) = direction {
                // Value with direction only: "value"^dir
                output.push_str(&format!("{}{}^{}\n", indent, val_str, dir));
            } else {
                // Just @value without @language or @type
                output.push_str(&format!("{}{}\n", indent, val_str));
            }
        }

        Ok(())
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
        assert!(toon.contains("Bonjour"));
        assert!(toon.contains("@fr"));
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
        assert!(toon.contains("2024-01-15"));
        assert!(toon.contains("^^xsd:date"));
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
        let serializer = ToonSerializer::new();

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
}
