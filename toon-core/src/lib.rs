//! TOON-LD Core Library
//!
//! High-performance serializer/parser for TOON-LD (Token-Oriented Object Notation for Linked Data).
//! Implements the TOON format v3.0 specification with JSON-LD context expansion support.
//!
//! # Overview
//!
//! TOON-LD is a text serialization format designed to minimize token count while preserving
//! the semantic richness of JSON-LD. It achieves 40-60% token reduction compared to equivalent
//! JSON-LD representations.
//!
//! # Features
//!
//! - **Tabular Arrays**: Arrays of objects are serialized as CSV-like tables with a header
//! - **Primitive Arrays**: Simple value arrays use compact inline notation
//! - **JSON-LD Context**: Full support for URI compaction/expansion via `@context`
//! - **JSON-LD 1.1 Keywords**: Support for all standard JSON-LD keywords
//!
//! # Example
//!
//! ```
//! use toon_core::{jsonld_to_toonld, toonld_to_jsonld};
//!
//! let json_ld = r#"{
//!     "@context": {"foaf": "http://xmlns.com/foaf/0.1/"},
//!     "http://xmlns.com/foaf/0.1/name": "Alice",
//!     "http://xmlns.com/foaf/0.1/age": 30
//! }"#;
//!
//! // Convert JSON-LD to TOON-LD
//! let toon = jsonld_to_toonld(json_ld).unwrap();
//! assert!(toon.contains("foaf:name: Alice"));
//!
//! // Convert back to JSON-LD
//! let back = toonld_to_jsonld(&toon).unwrap();
//! ```
//!
//! # Modules
//!
//! - [`context`]: JSON-LD context handling for URI compaction/expansion
//! - [`error`]: Error types for TOON-LD operations
//! - [`keywords`]: JSON-LD keyword constants and utilities
//! - [`parser`]: TOON-LD parser implementation
//! - [`serializer`]: TOON-LD serializer implementation

pub mod context;
pub mod error;
pub mod keywords;
pub mod parser;
pub mod serializer;

// Re-export main types at crate root for convenience
pub use context::JsonLdContext;
pub use error::{Result, ToonError};
pub use keywords::*;
pub use parser::ToonParser;
pub use serializer::ToonSerializer;

use serde_json::Value;

/// Convenience function to convert JSON-LD to TOON-LD.
///
/// This function parses the JSON-LD input, extracts the context for URI compaction,
/// and serializes the result to TOON-LD format.
///
/// # Arguments
///
/// * `json` - A JSON-LD formatted string
///
/// # Returns
///
/// A `Result` containing the TOON-LD string or an error.
///
/// # Example
///
/// ```
/// use toon_core::jsonld_to_toonld;
///
/// let json_ld = r#"{"name": "Alice", "age": 30}"#;
/// let toon = jsonld_to_toonld(json_ld).unwrap();
/// assert!(toon.contains("name: Alice"));
/// ```
pub fn jsonld_to_toonld(json: &str) -> Result<String> {
    let value: Value = serde_json::from_str(json)?;
    let context = JsonLdContext::from_value(&value);
    let serializer = ToonSerializer::new().with_context(context);
    serializer.serialize(&value)
}

/// Convenience function to convert TOON-LD to JSON-LD.
///
/// This function parses the TOON-LD input and returns a pretty-printed JSON string.
///
/// # Arguments
///
/// * `toon` - A TOON-LD formatted string
///
/// # Returns
///
/// A `Result` containing the JSON-LD string or an error.
///
/// # Example
///
/// ```
/// use toon_core::toonld_to_jsonld;
///
/// let toon = "name: Alice\nage: 30";
/// let json = toonld_to_jsonld(toon).unwrap();
/// assert!(json.contains("\"name\""));
/// ```
pub fn toonld_to_jsonld(toon: &str) -> Result<String> {
    let parser = ToonParser::new();
    parser.parse_to_json(toon)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_primitive_values() {
        let serializer = ToonSerializer::new();
        let parser = ToonParser::new();

        let json = r#"{"name": "Alice", "age": 30, "active": true}"#;
        let toon = serializer.serialize_json(json).unwrap();
        let back = parser.parse(&toon).unwrap();

        assert_eq!(back.get("name").unwrap(), "Alice");
        assert_eq!(back.get("age").unwrap(), 30);
        assert_eq!(back.get("active").unwrap(), true);
    }

    #[test]
    fn test_tabular_array() {
        let serializer = ToonSerializer::new();

        let json = r#"{
            "people": [
                {"name": "Alice", "age": 30},
                {"name": "Bob", "age": 25}
            ]
        }"#;

        let toon = serializer.serialize_json(json).unwrap();
        assert!(toon.contains("people[2]{"));
        assert!(toon.contains("Alice"));
        assert!(toon.contains("Bob"));
    }

    #[test]
    fn test_primitive_array() {
        let serializer = ToonSerializer::new();

        let json = r#"{"tags": ["rust", "wasm", "python"]}"#;
        let toon = serializer.serialize_json(json).unwrap();
        assert!(toon.contains("tags[3]:"));
    }

    #[test]
    fn test_quoting() {
        let serializer = ToonSerializer::new();

        // Value with comma should be quoted
        let json = r#"{"note": "Hello, World"}"#;
        let toon = serializer.serialize_json(json).unwrap();
        assert!(toon.contains("\"Hello, World\""));
    }

    #[test]
    fn test_roundtrip() {
        let original = r#"{
            "users": [
                {"id": 1, "name": "Alice"},
                {"id": 2, "name": "Bob"}
            ],
            "count": 2,
            "active": true
        }"#;

        let toon = jsonld_to_toonld(original).unwrap();
        let back_json = toonld_to_jsonld(&toon).unwrap();
        let back: Value = serde_json::from_str(&back_json).unwrap();

        assert_eq!(back.get("count").unwrap(), 2);
        assert_eq!(back.get("active").unwrap(), true);
    }

    #[test]
    fn test_jsonld_context() {
        let json = r#"{
            "@context": {
                "foaf": "http://xmlns.com/foaf/0.1/"
            },
            "http://xmlns.com/foaf/0.1/name": "Alice"
        }"#;

        let toon = jsonld_to_toonld(json).unwrap();
        assert!(toon.contains("foaf:name"));
    }

    #[test]
    fn test_empty_array() {
        let serializer = ToonSerializer::new();
        let parser = ToonParser::new();

        let json = r#"{"items": []}"#;
        let toon = serializer.serialize_json(json).unwrap();
        assert!(toon.contains("items[0]:"));

        let back = parser.parse(&toon).unwrap();
        assert!(back.get("items").unwrap().as_array().unwrap().is_empty());
    }

    #[test]
    fn test_nested_objects() {
        let serializer = ToonSerializer::new();

        let json = r#"{
            "person": {
                "name": "Alice",
                "address": {
                    "city": "Seattle",
                    "zip": "98101"
                }
            }
        }"#;

        let toon = serializer.serialize_json(json).unwrap();
        assert!(toon.contains("person:"));
        assert!(toon.contains("address:"));
        assert!(toon.contains("city: Seattle"));
    }

    #[test]
    fn test_missing_fields_in_tabular() {
        // Disable partitioning to test union schema explicitly
        let serializer = ToonSerializer::new().with_shape_partitioning(false);
        let parser = ToonParser::new();

        // Non-uniform array should still use tabular format with union of keys
        let json = r#"{
            "items": [
                {"a": 1, "b": 2},
                {"a": 3, "c": 4}
            ]
        }"#;

        let toon = serializer.serialize_json(json).unwrap();
        // Should use tabular format with union of all keys {a,b,c}
        assert!(toon.contains("items[2]{a,b,c}:"));
        // Missing fields should be null
        assert!(toon.contains("1, 2, null"));
        assert!(toon.contains("3, null, 4"));

        // Verify roundtrip
        let back = parser.parse(&toon).unwrap();
        let items = back.get("items").unwrap().as_array().unwrap();
        assert_eq!(items[0].get("a").unwrap(), 1);
        assert_eq!(items[0].get("b").unwrap(), 2);
        assert!(items[0].get("c").unwrap().is_null());
        assert_eq!(items[1].get("a").unwrap(), 3);
        assert!(items[1].get("b").unwrap().is_null());
        assert_eq!(items[1].get("c").unwrap(), 4);
    }

    #[test]
    fn test_special_characters_in_values() {
        let serializer = ToonSerializer::new();
        let parser = ToonParser::new();

        let json = r#"{"message": "Hello: World", "note": "A|B"}"#;
        let toon = serializer.serialize_json(json).unwrap();

        // Values with : or | should be quoted
        assert!(toon.contains("\"Hello: World\""));
        assert!(toon.contains("\"A|B\""));

        let back = parser.parse(&toon).unwrap();
        assert_eq!(back.get("message").unwrap(), "Hello: World");
        assert_eq!(back.get("note").unwrap(), "A|B");
    }

    #[test]
    fn test_jsonld_id_and_type() {
        let serializer = ToonSerializer::new();
        let parser = ToonParser::new();

        let json = r#"{
            "@id": "http://example.org/person/1",
            "@type": "Person",
            "name": "Alice"
        }"#;

        let toon = serializer.serialize_json(json).unwrap();

        assert!(toon.contains("@id:"));
        assert!(toon.contains("@type: Person"));
        assert!(toon.contains("name: Alice"));

        // Verify roundtrip
        let back = parser.parse(&toon).unwrap();
        assert_eq!(back.get("@id").unwrap(), "http://example.org/person/1");
        assert_eq!(back.get("@type").unwrap(), "Person");
        assert_eq!(back.get("name").unwrap(), "Alice");
    }

    #[test]
    fn test_jsonld_graph() {
        let serializer = ToonSerializer::new();
        let parser = ToonParser::new();

        let json = r#"{
            "@context": {
                "foaf": "http://xmlns.com/foaf/0.1/"
            },
            "@graph": [
                {"@id": "http://example.org/1", "@type": "Person", "foaf:name": "Alice"},
                {"@id": "http://example.org/2", "@type": "Person", "foaf:name": "Bob"}
            ]
        }"#;

        let toon = serializer.serialize_json(json).unwrap();

        // @graph should use tabular format
        assert!(toon.contains("@graph[2]"));
        assert!(toon.contains("@id"));
        assert!(toon.contains("@type"));

        // Verify roundtrip
        let back = parser.parse(&toon).unwrap();
        let graph = back.get("@graph").unwrap().as_array().unwrap();
        assert_eq!(graph.len(), 2);
        assert_eq!(graph[0].get("@type").unwrap(), "Person");
    }

    #[test]
    fn test_jsonld_type_array() {
        let serializer = ToonSerializer::new();
        let parser = ToonParser::new();

        // @type can be an array of types
        let json = r#"{
            "@id": "http://example.org/1",
            "@type": ["Person", "Agent"],
            "name": "Alice"
        }"#;

        let toon = serializer.serialize_json(json).unwrap();
        assert!(toon.contains("@type"));
        assert!(toon.contains("Person"));
        assert!(toon.contains("Agent"));

        let back = parser.parse(&toon).unwrap();
        let types = back.get("@type").unwrap().as_array().unwrap();
        assert_eq!(types.len(), 2);
    }

    #[test]
    fn test_jsonld_full_document() {
        let json = r#"{
            "@context": {
                "foaf": "http://xmlns.com/foaf/0.1/",
                "schema": "http://schema.org/"
            },
            "@id": "http://example.org/dataset",
            "@type": "schema:Dataset",
            "@graph": [
                {
                    "@id": "http://example.org/person/1",
                    "@type": "foaf:Person",
                    "foaf:name": "Alice",
                    "foaf:age": 30
                },
                {
                    "@id": "http://example.org/person/2",
                    "@type": "foaf:Person",
                    "foaf:name": "Bob",
                    "foaf:age": 25
                }
            ]
        }"#;

        let toon = jsonld_to_toonld(json).unwrap();

        // Context should come first
        assert!(toon.starts_with("@context:"));
        assert!(toon.contains("@id:"));
        assert!(toon.contains("http://example.org/dataset"));
        assert!(toon.contains("@type:"));
        assert!(toon.contains("@graph[2]"));

        // Roundtrip
        let back_json = toonld_to_jsonld(&toon).unwrap();
        let back: Value = serde_json::from_str(&back_json).unwrap();

        assert_eq!(back.get("@id").unwrap(), "http://example.org/dataset");
        let graph = back.get("@graph").unwrap().as_array().unwrap();
        assert_eq!(graph.len(), 2);
    }

    #[test]
    fn test_jsonld_value_with_language() {
        let json = r#"{
            "title": {"@value": "Bonjour", "@language": "fr"}
        }"#;

        let toon = jsonld_to_toonld(json).unwrap();

        // Value nodes now use standard TOON object syntax
        assert!(toon.contains("@value"));
        assert!(toon.contains("Bonjour"));
        assert!(toon.contains("@language"));
        assert!(toon.contains("fr"));
    }

    #[test]
    fn test_jsonld_value_with_type() {
        let json = r#"{
            "@context": {
                "xsd": "http://www.w3.org/2001/XMLSchema#"
            },
            "date": {"@value": "2024-01-15", "@type": "http://www.w3.org/2001/XMLSchema#date"}
        }"#;

        let toon = jsonld_to_toonld(json).unwrap();

        // Value nodes now use standard TOON object syntax
        assert!(toon.contains("@value"));
        assert!(toon.contains("2024-01-15"));
        assert!(toon.contains("@type"));
        assert!(toon.contains("xsd:date"));
    }

    #[test]
    fn test_jsonld_list() {
        let serializer = ToonSerializer::new();
        let parser = ToonParser::new();

        let json = r#"{
            "sequence": {"@list": ["first", "second", "third"]}
        }"#;

        let toon = serializer.serialize_json(json).unwrap();

        assert!(toon.contains("@list"));
        assert!(toon.contains("first"));

        let back = parser.parse(&toon).unwrap();
        let seq = back.get("sequence").unwrap().as_object().unwrap();
        let list = seq.get("@list").unwrap().as_array().unwrap();
        assert_eq!(list.len(), 3);
    }

    #[test]
    fn test_jsonld_set() {
        let serializer = ToonSerializer::new();
        let parser = ToonParser::new();

        let json = r#"{
            "tags": {"@set": ["rust", "wasm", "python"]}
        }"#;

        let toon = serializer.serialize_json(json).unwrap();

        assert!(toon.contains("@set"));

        let back = parser.parse(&toon).unwrap();
        let tags = back.get("tags").unwrap().as_object().unwrap();
        let set = tags.get("@set").unwrap().as_array().unwrap();
        assert_eq!(set.len(), 3);
    }

    #[test]
    fn test_jsonld_reverse() {
        let serializer = ToonSerializer::new();
        let parser = ToonParser::new();

        let json = r#"{
            "@id": "http://example.org/alice",
            "@reverse": {
                "foaf:knows": {"@id": "http://example.org/bob"}
            }
        }"#;

        let toon = serializer.serialize_json(json).unwrap();

        assert!(toon.contains("@reverse"));
        assert!(toon.contains("foaf:knows"));

        let back = parser.parse(&toon).unwrap();
        assert!(back.get("@reverse").is_some());
    }

    #[test]
    fn test_jsonld_base_and_vocab() {
        let serializer = ToonSerializer::new();

        let json = r#"{
            "@context": {
                "@base": "http://example.org/",
                "@vocab": "http://schema.org/"
            },
            "@id": "person/1",
            "name": "Alice"
        }"#;

        let toon = serializer.serialize_json(json).unwrap();

        assert!(toon.contains("@base"));
        assert!(toon.contains("@vocab"));
        assert!(toon.contains("http://example.org/"));
        assert!(toon.contains("http://schema.org/"));
    }

    // JSON-LD 1.1 keyword tests

    #[test]
    fn test_jsonld_version() {
        let serializer = ToonSerializer::new();
        let parser = ToonParser::new();

        let json = r#"{
            "@context": {
                "@version": 1.1,
                "name": "http://schema.org/name"
            },
            "name": "Alice"
        }"#;

        let toon = serializer.serialize_json(json).unwrap();

        assert!(toon.contains("@version"));
        assert!(toon.contains("1.1"));

        let back = parser.parse(&toon).unwrap();
        let ctx = back.get("@context").unwrap().as_object().unwrap();
        assert!(ctx.get("@version").is_some());
    }

    #[test]
    fn test_jsonld_direction() {
        let serializer = ToonSerializer::new();
        let parser = ToonParser::new();

        let json = r#"{
            "@context": {
                "@direction": "rtl"
            },
            "text": "Hello"
        }"#;

        let toon = serializer.serialize_json(json).unwrap();

        assert!(toon.contains("@direction"));
        assert!(toon.contains("rtl"));

        let back = parser.parse(&toon).unwrap();
        let ctx = back.get("@context").unwrap().as_object().unwrap();
        assert_eq!(ctx.get("@direction").unwrap(), "rtl");
    }

    #[test]
    fn test_jsonld_value_with_direction() {
        let json = r#"{
            "title": {"@value": "مرحبا", "@language": "ar", "@direction": "rtl"}
        }"#;

        let toon = jsonld_to_toonld(json).unwrap();

        // Value nodes now use standard TOON object syntax
        assert!(toon.contains("@value"));
        assert!(toon.contains("مرحبا"));
        assert!(toon.contains("@language"));
        assert!(toon.contains("ar"));
        assert!(toon.contains("@direction"));
        assert!(toon.contains("rtl"));
    }

    #[test]
    fn test_jsonld_container() {
        let serializer = ToonSerializer::new();
        let parser = ToonParser::new();

        let json = r#"{
            "@context": {
                "tags": {
                    "@id": "http://example.org/tags",
                    "@container": "@set"
                }
            },
            "tags": ["a", "b", "c"]
        }"#;

        let toon = serializer.serialize_json(json).unwrap();

        assert!(toon.contains("@container"));

        let back = parser.parse(&toon).unwrap();
        assert!(back.get("@context").is_some());
    }

    #[test]
    fn test_jsonld_container_array() {
        let serializer = ToonSerializer::new();
        let parser = ToonParser::new();

        let json = r#"{
            "@container": ["@index", "@set"]
        }"#;

        let toon = serializer.serialize_json(json).unwrap();

        assert!(toon.contains("@container"));

        let back = parser.parse(&toon).unwrap();
        let container = back.get("@container").unwrap().as_array().unwrap();
        assert_eq!(container.len(), 2);
    }

    #[test]
    fn test_jsonld_index() {
        let serializer = ToonSerializer::new();
        let parser = ToonParser::new();

        let json = r#"{
            "@id": "http://example.org/1",
            "@index": "chapter1",
            "title": "Introduction"
        }"#;

        let toon = serializer.serialize_json(json).unwrap();

        assert!(toon.contains("@index"));
        assert!(toon.contains("chapter1"));

        let back = parser.parse(&toon).unwrap();
        assert_eq!(back.get("@index").unwrap(), "chapter1");
    }

    #[test]
    fn test_jsonld_included() {
        let serializer = ToonSerializer::new();
        let parser = ToonParser::new();

        let json = r#"{
            "@id": "http://example.org/main",
            "name": "Main Resource",
            "@included": [
                {"@id": "http://example.org/ref1", "name": "Reference 1"},
                {"@id": "http://example.org/ref2", "name": "Reference 2"}
            ]
        }"#;

        let toon = serializer.serialize_json(json).unwrap();

        assert!(toon.contains("@included"));
        assert!(toon.contains("Reference 1"));
        assert!(toon.contains("Reference 2"));

        let back = parser.parse(&toon).unwrap();
        let included = back.get("@included").unwrap().as_array().unwrap();
        assert_eq!(included.len(), 2);
    }

    #[test]
    fn test_jsonld_nest() {
        let serializer = ToonSerializer::new();
        let parser = ToonParser::new();

        let json = r#"{
            "@id": "http://example.org/1",
            "@nest": {
                "nested_prop": "nested_value"
            }
        }"#;

        let toon = serializer.serialize_json(json).unwrap();

        assert!(toon.contains("@nest"));
        assert!(toon.contains("nested_prop"));
        assert!(toon.contains("nested_value"));

        let back = parser.parse(&toon).unwrap();
        let nest = back.get("@nest").unwrap().as_object().unwrap();
        assert_eq!(nest.get("nested_prop").unwrap(), "nested_value");
    }

    #[test]
    fn test_jsonld_prefix() {
        let serializer = ToonSerializer::new();
        let parser = ToonParser::new();

        let json = r#"{
            "@context": {
                "ex": {
                    "@id": "http://example.org/",
                    "@prefix": true
                }
            },
            "ex:name": "Test"
        }"#;

        let toon = serializer.serialize_json(json).unwrap();

        assert!(toon.contains("@prefix"));
        assert!(toon.contains("true"));

        let back = parser.parse(&toon).unwrap();
        assert!(back.get("@context").is_some());
    }

    #[test]
    fn test_jsonld_propagate() {
        let serializer = ToonSerializer::new();
        let parser = ToonParser::new();

        let json = r#"{
            "@context": {
                "@propagate": false,
                "name": "http://schema.org/name"
            },
            "name": "Alice"
        }"#;

        let toon = serializer.serialize_json(json).unwrap();

        assert!(toon.contains("@propagate"));
        assert!(toon.contains("false"));

        let back = parser.parse(&toon).unwrap();
        let ctx = back.get("@context").unwrap().as_object().unwrap();
        assert_eq!(ctx.get("@propagate").unwrap(), false);
    }

    #[test]
    fn test_jsonld_protected() {
        let serializer = ToonSerializer::new();
        let parser = ToonParser::new();

        let json = r#"{
            "@context": {
                "name": {
                    "@id": "http://schema.org/name",
                    "@protected": true
                }
            },
            "name": "Alice"
        }"#;

        let toon = serializer.serialize_json(json).unwrap();

        assert!(toon.contains("@protected"));
        assert!(toon.contains("true"));

        let _back = parser.parse(&toon).unwrap();
    }

    #[test]
    fn test_jsonld_import() {
        let serializer = ToonSerializer::new();
        let parser = ToonParser::new();

        let json = r#"{
            "@context": {
                "@import": "http://example.org/context.jsonld"
            },
            "name": "Alice"
        }"#;

        let toon = serializer.serialize_json(json).unwrap();

        assert!(toon.contains("@import"));
        assert!(toon.contains("http://example.org/context.jsonld"));

        let back = parser.parse(&toon).unwrap();
        let ctx = back.get("@context").unwrap().as_object().unwrap();
        assert!(ctx.get("@import").is_some());
    }

    #[test]
    fn test_jsonld_none() {
        let serializer = ToonSerializer::new();
        let parser = ToonParser::new();

        let json = r#"{
            "@none": {"name": "Default item"}
        }"#;

        let toon = serializer.serialize_json(json).unwrap();

        assert!(toon.contains("@none"));

        let back = parser.parse(&toon).unwrap();
        assert!(back.get("@none").is_some());
    }

    #[test]
    fn test_jsonld_json_literal() {
        let serializer = ToonSerializer::new();
        let parser = ToonParser::new();

        let json = r#"{
            "data": {
                "@value": {"nested": "object", "count": 42},
                "@type": "@json"
            }
        }"#;

        let toon = serializer.serialize_json(json).unwrap();

        assert!(toon.contains("data"));

        let back = parser.parse(&toon).unwrap();
        assert!(back.get("data").is_some());
    }

    #[test]
    fn test_jsonld_context_with_term_definitions() {
        let json = r#"{
            "@context": {
                "@version": 1.1,
                "name": {
                    "@id": "http://schema.org/name",
                    "@container": "@set",
                    "@protected": true
                },
                "knows": {
                    "@id": "http://xmlns.com/foaf/0.1/knows",
                    "@type": "@id"
                }
            },
            "name": ["Alice", "Alicia"],
            "knows": "http://example.org/bob"
        }"#;

        let toon = jsonld_to_toonld(json).unwrap();

        assert!(toon.contains("@context"));
        assert!(toon.contains("@version"));
        assert!(toon.contains("name"));
        assert!(toon.contains("knows"));

        let back_json = toonld_to_jsonld(&toon).unwrap();
        let back: Value = serde_json::from_str(&back_json).unwrap();
        assert!(back.get("@context").is_some());
    }

    #[test]
    fn test_jsonld_full_1_1_document() {
        let json = r#"{
            "@context": {
                "@version": 1.1,
                "@base": "http://example.org/",
                "@vocab": "http://schema.org/",
                "foaf": "http://xmlns.com/foaf/0.1/",
                "@direction": "ltr",
                "name": {
                    "@id": "foaf:name",
                    "@protected": true
                }
            },
            "@id": "person/1",
            "@type": "Person",
            "@index": "main",
            "name": "Alice",
            "@included": [
                {"@id": "person/2", "name": "Bob"}
            ]
        }"#;

        let toon = jsonld_to_toonld(json).unwrap();

        assert!(toon.contains("@context"));
        assert!(toon.contains("@version"));
        assert!(toon.contains("@base"));
        assert!(toon.contains("@vocab"));
        assert!(toon.contains("@direction"));
        assert!(toon.contains("@id"));
        assert!(toon.contains("@type"));
        assert!(toon.contains("@index"));
        assert!(toon.contains("@included"));
        assert!(toon.contains("Alice"));
        assert!(toon.contains("Bob"));

        let back_json = toonld_to_jsonld(&toon).unwrap();
        let back: Value = serde_json::from_str(&back_json).unwrap();
        assert!(back.get("@context").is_some());
        assert!(back.get("@included").is_some());
    }
}
