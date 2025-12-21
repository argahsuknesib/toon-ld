//! TOON-LD Parser
//!
//! This module provides the `ToonParser` struct for parsing TOON-LD format
//! back to JSON/JSON-LD values.

use once_cell::sync::Lazy;
use regex::Regex;
use serde_json::{Map, Value};

use crate::context::JsonLdContext;
use crate::error::{Result, ToonError};

/// Regex for parsing tabular array headers: key[N]{field1,field2}:
/// Supports @ prefix for JSON-LD keywords
static TABULAR_HEADER_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r#"^(@?\w+)\[(\d+)\]\{([^}]+)\}:$"#).expect("TABULAR_HEADER_REGEX is invalid")
});

/// Regex for parsing primitive array headers: key[N]:
/// Supports @ prefix for JSON-LD keywords
static PRIMITIVE_ARRAY_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r#"^(@?\w+)\[(\d+)\]:(.*)$"#).expect("PRIMITIVE_ARRAY_REGEX is invalid")
});

/// Regex for parsing key-value pairs
/// Supports @ prefix for JSON-LD keywords and prefixed URIs (e.g., foaf:name)
static KEY_VALUE_REGEX: Lazy<Regex> =
    Lazy::new(|| Regex::new(r#"^(@?\w+(?::\w+)?):\s*(.*)$"#).expect("KEY_VALUE_REGEX is invalid"));

/// Parser state machine modes
#[derive(Debug, Clone, PartialEq)]
enum ParseMode {
    /// Standard YAML-like indented mode
    Indented,
    /// CSV mode for tabular arrays
    Csv {
        /// Field names from the tabular header
        fields: Vec<String>,
        /// Number of rows remaining to parse
        remaining_rows: usize,
    },
}

/// TOON-LD Parser
///
/// Parses TOON-LD format back to JSON/JSON-LD values. The parser handles:
/// - Indented key-value pairs
/// - Tabular arrays (CSV format with header)
/// - Primitive arrays (inline or multi-line)
/// - Quoted and unquoted string values
///
/// # Example
///
/// ```
/// use toon_core::ToonParser;
///
/// let parser = ToonParser::new();
/// let toon = r#"
/// name: Alice
/// age: 30
/// "#;
///
/// let value = parser.parse(toon).unwrap();
/// assert_eq!(value.get("name").unwrap(), "Alice");
/// assert_eq!(value.get("age").unwrap(), 30);
/// ```
#[derive(Debug, Clone)]
pub struct ToonParser {
    /// JSON-LD context for URI expansion
    context: JsonLdContext,
}

impl Default for ToonParser {
    fn default() -> Self {
        Self::new()
    }
}

impl ToonParser {
    /// Create a new parser with default settings.
    ///
    /// # Example
    ///
    /// ```
    /// use toon_core::ToonParser;
    ///
    /// let parser = ToonParser::new();
    /// ```
    pub fn new() -> Self {
        Self {
            context: JsonLdContext::new(),
        }
    }

    /// Set the JSON-LD context for URI expansion.
    ///
    /// # Arguments
    ///
    /// * `context` - The JSON-LD context to use
    ///
    /// # Example
    ///
    /// ```
    /// use toon_core::{ToonParser, JsonLdContext};
    ///
    /// let mut ctx = JsonLdContext::new();
    /// ctx.add_prefix("foaf", "http://xmlns.com/foaf/0.1/");
    ///
    /// let parser = ToonParser::new().with_context(ctx);
    /// ```
    pub fn with_context(mut self, context: JsonLdContext) -> Self {
        self.context = context;
        self
    }

    /// Get a reference to the current context.
    pub fn context(&self) -> &JsonLdContext {
        &self.context
    }

    /// Parse TOON-LD string to JSON value.
    ///
    /// # Arguments
    ///
    /// * `input` - The TOON-LD string to parse
    ///
    /// # Returns
    ///
    /// A `Result` containing the parsed JSON value or an error.
    ///
    /// # Example
    ///
    /// ```
    /// use toon_core::ToonParser;
    ///
    /// let parser = ToonParser::new();
    /// let value = parser.parse("name: Alice\nage: 30").unwrap();
    ///
    /// assert_eq!(value.get("name").unwrap(), "Alice");
    /// ```
    pub fn parse(&self, input: &str) -> Result<Value> {
        let lines: Vec<&str> = input.lines().collect();
        let (value, _) = self.parse_lines(&lines, 0, 0)?;
        Ok(value)
    }

    /// Parse TOON-LD string to pretty-printed JSON string.
    ///
    /// # Arguments
    ///
    /// * `input` - The TOON-LD string to parse
    ///
    /// # Returns
    ///
    /// A `Result` containing the JSON string or an error.
    ///
    /// # Example
    ///
    /// ```
    /// use toon_core::ToonParser;
    ///
    /// let parser = ToonParser::new();
    /// let json = parser.parse_to_json("name: Alice").unwrap();
    ///
    /// assert!(json.contains("\"name\""));
    /// assert!(json.contains("\"Alice\""));
    /// ```
    pub fn parse_to_json(&self, input: &str) -> Result<String> {
        let value = self.parse(input)?;
        serde_json::to_string_pretty(&value).map_err(|e| e.into())
    }

    /// Parse lines starting from a given position and base indentation.
    fn parse_lines(
        &self,
        lines: &[&str],
        start: usize,
        base_indent: usize,
    ) -> Result<(Value, usize)> {
        let mut obj = Map::new();
        let mut i = start;
        let mut mode = ParseMode::Indented;
        let mut current_array_key: Option<String> = None;
        let mut current_array: Vec<Value> = Vec::new();

        while i < lines.len() {
            let line = lines[i];

            // Skip empty lines
            if line.trim().is_empty() {
                i += 1;
                continue;
            }

            let indent = self.get_indent(line);

            // If we've dedented past our base, we're done with this block
            if indent < base_indent && !line.trim().is_empty() {
                break;
            }

            // Handle CSV mode for tabular arrays
            match &mode {
                ParseMode::Csv {
                    fields,
                    remaining_rows,
                } => {
                    if *remaining_rows > 0 {
                        // Parse CSV row
                        let row_value = self.parse_csv_row(line.trim(), fields, i + 1)?;
                        current_array.push(row_value);

                        let new_remaining = remaining_rows - 1;
                        if new_remaining == 0 {
                            // Done with CSV block
                            if let Some(key) = current_array_key.take() {
                                obj.insert(key, Value::Array(std::mem::take(&mut current_array)));
                            }
                            mode = ParseMode::Indented;
                        } else {
                            mode = ParseMode::Csv {
                                fields: fields.clone(),
                                remaining_rows: new_remaining,
                            };
                        }
                        i += 1;
                        continue;
                    }
                }
                ParseMode::Indented => {}
            }

            let trimmed = line.trim();

            // Check for tabular array header: key[N]{field1,field2}:
            if let Some(caps) = TABULAR_HEADER_REGEX.captures(trimmed) {
                let key = self.get_capture_str(&caps, 1, i + 1, "tabular array key")?;
                let count = self.parse_array_count(&caps, 2, i + 1)?;
                let fields_str = self.get_capture_str(&caps, 3, i + 1, "tabular array fields")?;
                let fields: Vec<String> = fields_str
                    .split(',')
                    .map(|s| s.trim().to_string())
                    .collect();

                // Check if this key already exists - if so, we'll append to it
                let should_merge = obj.contains_key(&key.to_string());

                current_array_key = Some(key.to_string());

                // If merging, start with existing array; otherwise create new
                if should_merge {
                    if let Some(Value::Array(existing)) = obj.get(&key.to_string()) {
                        current_array = existing.clone();
                        current_array.reserve(count);
                    } else {
                        current_array = Vec::with_capacity(count);
                    }
                } else {
                    current_array = Vec::with_capacity(count);
                }

                if count > 0 {
                    mode = ParseMode::Csv {
                        fields,
                        remaining_rows: count,
                    };
                } else if let Some(key) = current_array_key.take() {
                    if !should_merge {
                        obj.insert(key, Value::Array(Vec::new()));
                    }
                }
                i += 1;
                continue;
            }

            // Check for primitive array header: key[N]: val1, val2 or key[N]:
            if let Some(caps) = PRIMITIVE_ARRAY_REGEX.captures(trimmed) {
                let key = self.get_capture_str(&caps, 1, i + 1, "primitive array key")?;
                let count = self.parse_array_count(&caps, 2, i + 1)?;
                let inline_values = caps.get(3).map(|m| m.as_str().trim()).unwrap_or("");

                if !inline_values.is_empty() {
                    // Inline primitive array
                    let values = self.parse_csv_values(inline_values, i + 1)?;
                    obj.insert(key.to_string(), Value::Array(values));
                } else if count > 0 {
                    // Multi-line primitive array
                    let mut arr = Vec::with_capacity(count);
                    for j in 0..count {
                        i += 1;
                        if i < lines.len() {
                            let val_line = lines[i].trim();
                            let parsed = if let Some(stripped) = val_line.strip_prefix("- ") {
                                self.parse_primitive(stripped, i + 1)?
                            } else {
                                self.parse_primitive(val_line, i + 1)?
                            };
                            arr.push(parsed);
                        } else {
                            return Err(ToonError::parse_error(
                                i + 1,
                                format!(
                                    "unexpected end of input while parsing array (expected {} more values)",
                                    count - j
                                ),
                            ));
                        }
                    }
                    obj.insert(key.to_string(), Value::Array(arr));
                } else {
                    obj.insert(key.to_string(), Value::Array(Vec::new()));
                }
                i += 1;
                continue;
            }

            // Check for key-value pair
            if let Some(caps) = KEY_VALUE_REGEX.captures(trimmed) {
                let key = self.get_capture_str(&caps, 1, i + 1, "key")?;
                let value_str = caps.get(2).map(|m| m.as_str().trim()).unwrap_or("");

                if value_str.is_empty() {
                    // Nested object - parse recursively
                    let (nested, consumed) = self.parse_lines(lines, i + 1, indent + 2)?;
                    obj.insert(key.to_string(), nested);
                    i = consumed;
                } else {
                    // Simple value
                    obj.insert(key.to_string(), self.parse_primitive(value_str, i + 1)?);
                    i += 1;
                }
                continue;
            }

            i += 1;
        }

        // Handle any remaining CSV data
        if let Some(key) = current_array_key.take() {
            if !current_array.is_empty() {
                obj.insert(key, Value::Array(current_array));
            }
        }

        Ok((Value::Object(obj), i))
    }

    /// Get a capture group as a string with error handling.
    fn get_capture_str<'a>(
        &self,
        caps: &'a regex::Captures<'a>,
        group: usize,
        line: usize,
        description: &str,
    ) -> Result<&'a str> {
        caps.get(group)
            .map(|m| m.as_str())
            .ok_or_else(|| ToonError::parse_error(line, format!("missing {}", description)))
    }

    /// Parse array count from capture group with error handling.
    fn parse_array_count(
        &self,
        caps: &regex::Captures<'_>,
        group: usize,
        line: usize,
    ) -> Result<usize> {
        let count_str = self.get_capture_str(caps, group, line, "array count")?;
        count_str.parse::<usize>().map_err(|_| {
            ToonError::parse_error(line, format!("invalid array count: {}", count_str))
        })
    }

    /// Get the indentation level of a line (number of leading spaces).
    #[inline]
    fn get_indent(&self, line: &str) -> usize {
        line.len() - line.trim_start().len()
    }

    /// Parse a CSV row into a JSON object.
    fn parse_csv_row(&self, line: &str, fields: &[String], line_num: usize) -> Result<Value> {
        let values = self.parse_csv_values(line, line_num)?;
        let mut obj = Map::new();

        for (i, field) in fields.iter().enumerate() {
            let value = values.get(i).cloned().unwrap_or(Value::Null);
            obj.insert(field.clone(), value);
        }

        Ok(Value::Object(obj))
    }

    /// Parse comma-separated values from a line.
    fn parse_csv_values(&self, line: &str, line_num: usize) -> Result<Vec<Value>> {
        let mut values = Vec::new();
        let mut current = String::new();
        let mut in_quotes = false;
        let mut chars = line.chars().peekable();

        while let Some(c) = chars.next() {
            if in_quotes {
                if c == '"' {
                    if chars.peek() == Some(&'"') {
                        // Escaped quote (CSV style)
                        current.push('"');
                        chars.next();
                    } else {
                        in_quotes = false;
                    }
                } else if c == '\\' {
                    // Handle backslash escapes
                    if let Some(next) = chars.next() {
                        match next {
                            '"' => current.push('"'),
                            '\\' => current.push('\\'),
                            'n' => current.push('\n'),
                            't' => current.push('\t'),
                            'r' => current.push('\r'),
                            _ => {
                                current.push('\\');
                                current.push(next);
                            }
                        }
                    }
                } else {
                    current.push(c);
                }
            } else if c == '"' {
                in_quotes = true;
            } else if c == ',' {
                values.push(self.parse_primitive(current.trim(), line_num)?);
                current.clear();
            } else {
                current.push(c);
            }
        }

        // Don't forget the last value
        if !current.is_empty() || !values.is_empty() {
            values.push(self.parse_primitive(current.trim(), line_num)?);
        }

        Ok(values)
    }

    /// Parse a primitive value (null, boolean, number, or string).
    fn parse_primitive(&self, s: &str, _line_num: usize) -> Result<Value> {
        let s = s.trim();

        // Handle quoted strings
        if s.starts_with('"') && s.ends_with('"') && s.len() >= 2 {
            let inner = &s[1..s.len() - 1];
            return Ok(Value::String(
                inner.replace("\\\"", "\"").replace("\\\\", "\\"),
            ));
        }

        // Handle null
        if s == "null" {
            return Ok(Value::Null);
        }

        // Handle booleans
        if s == "true" {
            return Ok(Value::Bool(true));
        }
        if s == "false" {
            return Ok(Value::Bool(false));
        }

        // Try to parse as integer
        if let Ok(n) = s.parse::<i64>() {
            return Ok(Value::Number(n.into()));
        }

        // Try to parse as float
        if let Ok(n) = s.parse::<f64>() {
            if let Some(num) = serde_json::Number::from_f64(n) {
                return Ok(Value::Number(num));
            }
        }

        // Default to string (unquoted)
        Ok(Value::String(s.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_parser() {
        let parser = ToonParser::new();
        assert!(parser.context().is_empty());
    }

    #[test]
    fn test_with_context() {
        let mut ctx = JsonLdContext::new();
        ctx.add_prefix("foaf", "http://xmlns.com/foaf/0.1/");

        let parser = ToonParser::new().with_context(ctx);
        assert!(parser.context().has_prefixes());
    }

    #[test]
    fn test_parse_primitives() {
        let parser = ToonParser::new();

        let toon = r#"
name: Alice
age: 30
score: 3.15
active: true
disabled: false
nothing: null
"#;

        let value = parser.parse(toon).unwrap();
        assert_eq!(value.get("name").unwrap(), "Alice");
        assert_eq!(value.get("age").unwrap(), 30);
        assert_eq!(value.get("score").unwrap(), 3.15);
        assert_eq!(value.get("active").unwrap(), true);
        assert_eq!(value.get("disabled").unwrap(), false);
        assert!(value.get("nothing").unwrap().is_null());
    }

    #[test]
    fn test_parse_quoted_string() {
        let parser = ToonParser::new();

        let toon = r#"message: "Hello, World!""#;
        let value = parser.parse(toon).unwrap();
        assert_eq!(value.get("message").unwrap(), "Hello, World!");
    }

    #[test]
    fn test_parse_nested_object() {
        let parser = ToonParser::new();

        let toon = r#"
person:
  name: Alice
  address:
    city: Seattle
    zip: 98101
"#;

        let value = parser.parse(toon).unwrap();
        let person = value.get("person").unwrap();
        assert_eq!(person.get("name").unwrap(), "Alice");

        let address = person.get("address").unwrap();
        assert_eq!(address.get("city").unwrap(), "Seattle");
        assert_eq!(address.get("zip").unwrap(), 98101);
    }

    #[test]
    fn test_parse_primitive_array_inline() {
        let parser = ToonParser::new();

        let toon = "tags[3]: rust, wasm, python";
        let value = parser.parse(toon).unwrap();

        let tags = value.get("tags").unwrap().as_array().unwrap();
        assert_eq!(tags.len(), 3);
        assert_eq!(tags[0], "rust");
        assert_eq!(tags[1], "wasm");
        assert_eq!(tags[2], "python");
    }

    #[test]
    fn test_parse_primitive_array_multiline() {
        let parser = ToonParser::new();

        let toon = r#"
numbers[3]:
  1
  2
  3
"#;

        let value = parser.parse(toon).unwrap();
        let numbers = value.get("numbers").unwrap().as_array().unwrap();
        assert_eq!(numbers.len(), 3);
        assert_eq!(numbers[0], 1);
        assert_eq!(numbers[1], 2);
        assert_eq!(numbers[2], 3);
    }

    #[test]
    fn test_parse_empty_array() {
        let parser = ToonParser::new();

        let toon = "items[0]:";
        let value = parser.parse(toon).unwrap();

        let items = value.get("items").unwrap().as_array().unwrap();
        assert!(items.is_empty());
    }

    #[test]
    fn test_parse_tabular_array() {
        let parser = ToonParser::new();

        let toon = r#"
people[2]{name,age}:
  Alice, 30
  Bob, 25
"#;

        let value = parser.parse(toon).unwrap();
        let people = value.get("people").unwrap().as_array().unwrap();
        assert_eq!(people.len(), 2);

        assert_eq!(people[0].get("name").unwrap(), "Alice");
        assert_eq!(people[0].get("age").unwrap(), 30);
        assert_eq!(people[1].get("name").unwrap(), "Bob");
        assert_eq!(people[1].get("age").unwrap(), 25);
    }

    #[test]
    fn test_parse_tabular_array_with_null() {
        let parser = ToonParser::new();

        let toon = r#"
items[2]{a,b,c}:
  1, 2, null
  3, null, 4
"#;

        let value = parser.parse(toon).unwrap();
        let items = value.get("items").unwrap().as_array().unwrap();

        assert_eq!(items[0].get("a").unwrap(), 1);
        assert_eq!(items[0].get("b").unwrap(), 2);
        assert!(items[0].get("c").unwrap().is_null());

        assert_eq!(items[1].get("a").unwrap(), 3);
        assert!(items[1].get("b").unwrap().is_null());
        assert_eq!(items[1].get("c").unwrap(), 4);
    }

    #[test]
    fn test_parse_tabular_array_with_quoted_values() {
        let parser = ToonParser::new();

        let toon = r#"
messages[2]{id,text}:
  1, "Hello, World!"
  2, "Goodbye, World!"
"#;

        let value = parser.parse(toon).unwrap();
        let messages = value.get("messages").unwrap().as_array().unwrap();

        assert_eq!(messages[0].get("id").unwrap(), 1);
        assert_eq!(messages[0].get("text").unwrap(), "Hello, World!");
        assert_eq!(messages[1].get("id").unwrap(), 2);
        assert_eq!(messages[1].get("text").unwrap(), "Goodbye, World!");
    }

    #[test]
    fn test_parse_jsonld_keywords() {
        let parser = ToonParser::new();

        let toon = r#"
@context:
  foaf: http://xmlns.com/foaf/0.1/
@id: http://example.org/alice
@type: Person
"#;

        let value = parser.parse(toon).unwrap();
        assert!(value.get("@context").is_some());
        assert_eq!(value.get("@id").unwrap(), "http://example.org/alice");
        assert_eq!(value.get("@type").unwrap(), "Person");
    }

    #[test]
    fn test_parse_to_json() {
        let parser = ToonParser::new();

        let toon = "name: Alice\nage: 30";
        let json = parser.parse_to_json(toon).unwrap();

        assert!(json.contains("\"name\""));
        assert!(json.contains("\"Alice\""));
        assert!(json.contains("\"age\""));
        assert!(json.contains("30"));
    }

    #[test]
    fn test_parse_escaped_quotes() {
        let parser = ToonParser::new();

        let toon = r#"message: "She said \"Hello\"""#;
        let value = parser.parse(toon).unwrap();
        assert_eq!(value.get("message").unwrap(), "She said \"Hello\"");
    }

    #[test]
    fn test_parse_csv_escaped_quotes() {
        let parser = ToonParser::new();

        let toon = r#"
items[1]{name,note}:
  Test, "Said ""Hi"""
"#;

        let value = parser.parse(toon).unwrap();
        let items = value.get("items").unwrap().as_array().unwrap();
        assert_eq!(items[0].get("note").unwrap(), "Said \"Hi\"");
    }

    #[test]
    fn test_parse_prefixed_keys() {
        let parser = ToonParser::new();

        let toon = "foaf:name: Alice\nschema:age: 30";
        let value = parser.parse(toon).unwrap();

        assert_eq!(value.get("foaf:name").unwrap(), "Alice");
        assert_eq!(value.get("schema:age").unwrap(), 30);
    }

    #[test]
    fn test_roundtrip_basic() {
        use crate::ToonSerializer;

        let original = serde_json::json!({
            "name": "Alice",
            "age": 30,
            "active": true
        });

        let serializer = ToonSerializer::new();
        let parser = ToonParser::new();

        let toon = serializer.serialize(&original).unwrap();
        let parsed = parser.parse(&toon).unwrap();

        assert_eq!(parsed.get("name").unwrap(), "Alice");
        assert_eq!(parsed.get("age").unwrap(), 30);
        assert_eq!(parsed.get("active").unwrap(), true);
    }

    #[test]
    fn test_roundtrip_tabular_array() {
        use crate::ToonSerializer;

        let original = serde_json::json!({
            "people": [
                {"name": "Alice", "age": 30},
                {"name": "Bob", "age": 25}
            ]
        });

        let serializer = ToonSerializer::new();
        let parser = ToonParser::new();

        let toon = serializer.serialize(&original).unwrap();
        let parsed = parser.parse(&toon).unwrap();

        let people = parsed.get("people").unwrap().as_array().unwrap();
        assert_eq!(people.len(), 2);
        assert_eq!(people[0].get("name").unwrap(), "Alice");
        assert_eq!(people[1].get("name").unwrap(), "Bob");
    }

    #[test]
    fn test_parse_multiple_array_blocks_same_key() {
        // Test parsing multiple tabular array blocks with the same key
        // This simulates shape-based partitioning output
        let parser = ToonParser::new();

        let toon = r#"
@graph[2]{@id,@type,name,age}:
  ex:1, Person, Alice, 30
  ex:2, Person, Bob, 25

@graph[1]{@id,@type,name,industry}:
  ex:3, Organization, ACME, Tech
"#;

        let parsed = parser.parse(toon).unwrap();
        let graph = parsed.get("@graph").expect("Should have @graph");
        assert!(graph.is_array());

        let graph_arr = graph.as_array().unwrap();
        assert_eq!(graph_arr.len(), 3, "Should have merged all 3 entities");

        // Verify entities are present
        assert!(graph_arr
            .iter()
            .any(|v| v.get("@id").and_then(|id| id.as_str()) == Some("ex:1")));
        assert!(graph_arr
            .iter()
            .any(|v| v.get("@id").and_then(|id| id.as_str()) == Some("ex:2")));
        assert!(graph_arr
            .iter()
            .any(|v| v.get("@id").and_then(|id| id.as_str()) == Some("ex:3")));

        // Verify different shapes are preserved
        let alice = graph_arr
            .iter()
            .find(|v| v.get("@id").and_then(|id| id.as_str()) == Some("ex:1"))
            .unwrap();
        assert_eq!(alice.get("age").and_then(|v| v.as_i64()), Some(30));
        assert!(alice.get("industry").is_none());

        let acme = graph_arr
            .iter()
            .find(|v| v.get("@id").and_then(|id| id.as_str()) == Some("ex:3"))
            .unwrap();
        assert_eq!(acme.get("industry").and_then(|v| v.as_str()), Some("Tech"));
        assert!(acme.get("age").is_none());
    }
}
