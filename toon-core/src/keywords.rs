//! JSON-LD keyword constants for TOON-LD.
//!
//! This module defines all JSON-LD 1.0 and 1.1 keywords used in TOON-LD serialization
//! and parsing. Keywords are organized into core keywords (JSON-LD 1.0) and extended
//! keywords (JSON-LD 1.1).

// =============================================================================
// JSON-LD Keywords (Core - JSON-LD 1.0)
// =============================================================================

/// The `@context` keyword defines the vocabulary mapping for terms
pub const JSONLD_CONTEXT: &str = "@context";

/// The `@id` keyword identifies a node with an IRI
pub const JSONLD_ID: &str = "@id";

/// The `@type` keyword specifies the type of a node or value
pub const JSONLD_TYPE: &str = "@type";

/// The `@graph` keyword contains a named graph
pub const JSONLD_GRAPH: &str = "@graph";

/// The `@value` keyword specifies the value of a typed or language-tagged literal
pub const JSONLD_VALUE: &str = "@value";

/// The `@language` keyword specifies the language of a string value
pub const JSONLD_LANGUAGE: &str = "@language";

/// The `@list` keyword specifies an ordered list
pub const JSONLD_LIST: &str = "@list";

/// The `@set` keyword specifies an unordered set
pub const JSONLD_SET: &str = "@set";

/// The `@reverse` keyword specifies reverse properties
pub const JSONLD_REVERSE: &str = "@reverse";

/// The `@base` keyword sets the base IRI for relative IRIs
pub const JSONLD_BASE: &str = "@base";

/// The `@vocab` keyword sets the default vocabulary for terms
pub const JSONLD_VOCAB: &str = "@vocab";

// =============================================================================
// JSON-LD 1.1 Keywords (Extended)
// =============================================================================

/// The `@container` keyword specifies how values are organized
pub const JSONLD_CONTAINER: &str = "@container";

/// The `@direction` keyword specifies text direction (ltr/rtl)
pub const JSONLD_DIRECTION: &str = "@direction";

/// The `@import` keyword imports an external context
pub const JSONLD_IMPORT: &str = "@import";

/// The `@included` keyword specifies included nodes
pub const JSONLD_INCLUDED: &str = "@included";

/// The `@index` keyword specifies an index value for a node
pub const JSONLD_INDEX: &str = "@index";

/// The `@json` keyword marks a JSON literal value
pub const JSONLD_JSON: &str = "@json";

/// The `@nest` keyword specifies nested properties
pub const JSONLD_NEST: &str = "@nest";

/// The `@none` keyword represents the default index value
pub const JSONLD_NONE: &str = "@none";

/// The `@prefix` keyword indicates a term is a prefix
pub const JSONLD_PREFIX: &str = "@prefix";

/// The `@propagate` keyword controls context propagation
pub const JSONLD_PROPAGATE: &str = "@propagate";

/// The `@protected` keyword prevents term redefinition
pub const JSONLD_PROTECTED: &str = "@protected";

/// The `@version` keyword specifies the JSON-LD version
pub const JSONLD_VERSION: &str = "@version";

// =============================================================================
// TOON-LD Shorthand Keywords
// =============================================================================
// These are the same as JSON-LD keywords but defined separately for clarity
// in the TOON-LD serialization format. We preserve the @ prefix for clarity.

/// TOON-LD shorthand for `@id`
pub const TOON_ID: &str = "@id";

/// TOON-LD shorthand for `@type`
pub const TOON_TYPE: &str = "@type";

/// TOON-LD shorthand for `@graph`
pub const TOON_GRAPH: &str = "@graph";

/// TOON-LD shorthand for `@value`
pub const TOON_VALUE: &str = "@value";

/// TOON-LD shorthand for `@language`
pub const TOON_LANGUAGE: &str = "@language";

/// TOON-LD shorthand for `@list`
pub const TOON_LIST: &str = "@list";

/// TOON-LD shorthand for `@set`
pub const TOON_SET: &str = "@set";

/// TOON-LD shorthand for `@reverse`
pub const TOON_REVERSE: &str = "@reverse";

/// TOON-LD shorthand for `@base`
pub const TOON_BASE: &str = "@base";

/// TOON-LD shorthand for `@vocab`
pub const TOON_VOCAB: &str = "@vocab";

/// TOON-LD shorthand for `@container`
pub const TOON_CONTAINER: &str = "@container";

/// TOON-LD shorthand for `@direction`
pub const TOON_DIRECTION: &str = "@direction";

/// TOON-LD shorthand for `@import`
pub const TOON_IMPORT: &str = "@import";

/// TOON-LD shorthand for `@included`
pub const TOON_INCLUDED: &str = "@included";

/// TOON-LD shorthand for `@index`
pub const TOON_INDEX: &str = "@index";

/// TOON-LD shorthand for `@json`
pub const TOON_JSON: &str = "@json";

/// TOON-LD shorthand for `@nest`
pub const TOON_NEST: &str = "@nest";

/// TOON-LD shorthand for `@none`
pub const TOON_NONE: &str = "@none";

/// TOON-LD shorthand for `@prefix`
pub const TOON_PREFIX: &str = "@prefix";

/// TOON-LD shorthand for `@propagate`
pub const TOON_PROPAGATE: &str = "@propagate";

/// TOON-LD shorthand for `@protected`
pub const TOON_PROTECTED: &str = "@protected";

/// TOON-LD shorthand for `@version`
pub const TOON_VERSION: &str = "@version";

// =============================================================================
// Keyword Ordering
// =============================================================================

/// Returns the serialization order priority for a JSON-LD keyword.
///
/// Lower values appear first in the output. This ensures consistent ordering
/// with `@context` first, followed by metadata keywords, then content keywords.
///
/// # Arguments
///
/// * `keyword` - The JSON-LD keyword to get the order for
///
/// # Returns
///
/// An integer representing the sort order (lower = earlier)
pub fn keyword_order(keyword: &str) -> i32 {
    match keyword {
        // Context and metadata come first
        JSONLD_CONTEXT => 0,
        JSONLD_VERSION => 1,
        JSONLD_BASE => 2,
        JSONLD_VOCAB => 3,
        JSONLD_IMPORT => 4,
        JSONLD_PROPAGATE => 5,
        JSONLD_DIRECTION => 6,

        // Node identification
        JSONLD_ID => 10,
        JSONLD_TYPE => 11,
        JSONLD_INDEX => 12,

        // Content keywords
        JSONLD_GRAPH => 13,
        JSONLD_INCLUDED => 14,
        JSONLD_REVERSE => 15,
        JSONLD_NEST => 16,

        // Container and type keywords
        JSONLD_CONTAINER => 20,
        JSONLD_JSON => 21,
        JSONLD_NONE => 22,
        JSONLD_PREFIX => 23,
        JSONLD_PROTECTED => 24,

        // Value keywords (typically handled specially)
        JSONLD_VALUE => 30,
        JSONLD_LANGUAGE => 31,
        JSONLD_LIST => 32,
        JSONLD_SET => 33,

        // Non-keywords come last
        _ => 100,
    }
}

/// Checks if a string is a JSON-LD keyword (starts with @)
///
/// # Arguments
///
/// * `s` - The string to check
///
/// # Returns
///
/// `true` if the string starts with `@`, `false` otherwise
#[inline]
pub fn is_keyword(s: &str) -> bool {
    s.starts_with('@')
}

/// Returns the TOON-LD display key for a JSON-LD keyword.
///
/// For most keywords, this is the same as the input. This function
/// provides a central place to handle any keyword transformations
/// if needed in the future.
///
/// # Arguments
///
/// * `keyword` - The JSON-LD keyword
///
/// # Returns
///
/// The corresponding TOON-LD display key, or `None` if not a known keyword.
pub fn get_toon_keyword(keyword: &str) -> Option<&'static str> {
    match keyword {
        JSONLD_CONTEXT => Some(JSONLD_CONTEXT),
        JSONLD_ID => Some(TOON_ID),
        JSONLD_TYPE => Some(TOON_TYPE),
        JSONLD_GRAPH => Some(TOON_GRAPH),
        JSONLD_VALUE => Some(TOON_VALUE),
        JSONLD_LANGUAGE => Some(TOON_LANGUAGE),
        JSONLD_LIST => Some(TOON_LIST),
        JSONLD_SET => Some(TOON_SET),
        JSONLD_REVERSE => Some(TOON_REVERSE),
        JSONLD_BASE => Some(TOON_BASE),
        JSONLD_VOCAB => Some(TOON_VOCAB),
        JSONLD_CONTAINER => Some(TOON_CONTAINER),
        JSONLD_DIRECTION => Some(TOON_DIRECTION),
        JSONLD_IMPORT => Some(TOON_IMPORT),
        JSONLD_INCLUDED => Some(TOON_INCLUDED),
        JSONLD_INDEX => Some(TOON_INDEX),
        JSONLD_JSON => Some(TOON_JSON),
        JSONLD_NEST => Some(TOON_NEST),
        JSONLD_NONE => Some(TOON_NONE),
        JSONLD_PREFIX => Some(TOON_PREFIX),
        JSONLD_PROPAGATE => Some(TOON_PROPAGATE),
        JSONLD_PROTECTED => Some(TOON_PROTECTED),
        JSONLD_VERSION => Some(TOON_VERSION),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_keyword_order() {
        assert!(keyword_order(JSONLD_CONTEXT) < keyword_order(JSONLD_ID));
        assert!(keyword_order(JSONLD_ID) < keyword_order(JSONLD_GRAPH));
        assert!(keyword_order(JSONLD_GRAPH) < keyword_order("custom_property"));
    }

    #[test]
    fn test_is_keyword() {
        assert!(is_keyword("@context"));
        assert!(is_keyword("@id"));
        assert!(is_keyword("@type"));
        assert!(is_keyword("@custom"));
        assert!(!is_keyword("name"));
        assert!(!is_keyword("foaf:name"));
        assert!(!is_keyword(""));
    }

    #[test]
    fn test_get_toon_keyword() {
        assert_eq!(get_toon_keyword(JSONLD_CONTEXT), Some("@context"));
        assert_eq!(get_toon_keyword(JSONLD_ID), Some("@id"));
        assert_eq!(get_toon_keyword(JSONLD_TYPE), Some("@type"));
        assert_eq!(get_toon_keyword(JSONLD_VERSION), Some("@version"));
    }

    #[test]
    fn test_unknown_keyword_returns_none() {
        // Unknown keywords return None
        assert_eq!(get_toon_keyword("@unknown"), None);
        assert_eq!(get_toon_keyword("name"), None);
    }
}
