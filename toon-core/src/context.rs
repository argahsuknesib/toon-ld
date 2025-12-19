//! JSON-LD Context handling for TOON-LD.
//!
//! This module provides the `JsonLdContext` struct for managing JSON-LD context
//! information, including prefix definitions, base IRI, vocabulary, and other
//! context-related settings.

use serde_json::Value;
use std::collections::HashMap;

/// JSON-LD context for prefix expansion/compaction.
///
/// The context stores prefix mappings and other JSON-LD context settings
/// that are used during serialization and parsing to compact and expand URIs.
///
/// # Example
///
/// ```
/// use toon_core::JsonLdContext;
/// use serde_json::json;
///
/// let value = json!({
///     "@context": {
///         "foaf": "http://xmlns.com/foaf/0.1/",
///         "@base": "http://example.org/"
///     }
/// });
///
/// let ctx = JsonLdContext::from_value(&value);
/// assert_eq!(ctx.compact_uri("http://xmlns.com/foaf/0.1/name"), "foaf:name");
/// ```
#[derive(Debug, Clone, Default)]
pub struct JsonLdContext {
    /// Prefix to URI mappings (e.g., "foaf" -> "http://xmlns.com/foaf/0.1/")
    prefixes: HashMap<String, String>,
    /// Base IRI for relative URI resolution
    base: Option<String>,
    /// Default vocabulary for terms without explicit mapping
    vocab: Option<String>,
    /// JSON-LD version (1.0 or 1.1)
    version: Option<f64>,
    /// Default text direction (ltr or rtl)
    direction: Option<String>,
    /// Context propagation flag
    propagate: Option<bool>,
    /// External context to import
    import: Option<String>,
}

impl JsonLdContext {
    /// Create a new empty context.
    ///
    /// # Example
    ///
    /// ```
    /// use toon_core::JsonLdContext;
    ///
    /// let ctx = JsonLdContext::new();
    /// assert!(ctx.base().is_none());
    /// assert!(ctx.vocab().is_none());
    /// ```
    pub fn new() -> Self {
        Self::default()
    }

    /// Extract context from a JSON-LD document.
    ///
    /// This parses the `@context` key from the given JSON value and extracts
    /// all prefix definitions and context settings.
    ///
    /// # Arguments
    ///
    /// * `value` - A JSON value potentially containing an `@context` key
    ///
    /// # Example
    ///
    /// ```
    /// use toon_core::JsonLdContext;
    /// use serde_json::json;
    ///
    /// let value = json!({
    ///     "@context": {
    ///         "schema": "http://schema.org/",
    ///         "@vocab": "http://example.org/vocab/"
    ///     },
    ///     "schema:name": "Test"
    /// });
    ///
    /// let ctx = JsonLdContext::from_value(&value);
    /// assert_eq!(ctx.vocab(), Some("http://example.org/vocab/"));
    /// ```
    pub fn from_value(value: &Value) -> Self {
        let mut ctx = Self::new();
        if let Some(context) = value.get("@context") {
            ctx.parse_context(context);
        }
        ctx
    }

    /// Parse a context value (can be object, array, or string).
    fn parse_context(&mut self, context: &Value) {
        match context {
            Value::Object(map) => {
                for (key, val) in map {
                    self.parse_context_entry(key, val);
                }
            }
            Value::Array(arr) => {
                // Multiple contexts - parse each one
                for item in arr {
                    self.parse_context(item);
                }
            }
            Value::String(uri) => {
                // External context reference - store as import
                // In a full implementation, this would trigger fetching
                self.import = Some(uri.clone());
            }
            _ => {}
        }
    }

    /// Parse a single context entry (key-value pair).
    fn parse_context_entry(&mut self, key: &str, val: &Value) {
        match key {
            "@base" => {
                if let Value::String(uri) = val {
                    self.base = Some(uri.clone());
                }
            }
            "@vocab" => {
                if let Value::String(uri) = val {
                    self.vocab = Some(uri.clone());
                }
            }
            "@version" => {
                if let Value::Number(n) = val {
                    self.version = n.as_f64();
                }
            }
            "@direction" => {
                if let Value::String(dir) = val {
                    self.direction = Some(dir.clone());
                }
            }
            "@propagate" => {
                if let Value::Bool(p) = val {
                    self.propagate = Some(*p);
                }
            }
            "@import" => {
                if let Value::String(uri) = val {
                    self.import = Some(uri.clone());
                }
            }
            _ => {
                // Handle term definitions (can be string or object)
                self.parse_term_definition(key, val);
            }
        }
    }

    /// Parse a term definition (prefix or expanded term).
    fn parse_term_definition(&mut self, key: &str, val: &Value) {
        match val {
            Value::String(uri) => {
                // Simple prefix definition: "foaf": "http://xmlns.com/foaf/0.1/"
                self.prefixes.insert(key.to_string(), uri.clone());
            }
            Value::Object(term_def) => {
                // Expanded term definition with @id
                if let Some(Value::String(id)) = term_def.get("@id") {
                    self.prefixes.insert(key.to_string(), id.clone());
                }
            }
            _ => {}
        }
    }

    /// Get the base IRI if defined.
    ///
    /// # Returns
    ///
    /// The base IRI as a string slice, or `None` if not set.
    #[inline]
    pub fn base(&self) -> Option<&str> {
        self.base.as_deref()
    }

    /// Get the default vocabulary if defined.
    ///
    /// # Returns
    ///
    /// The vocabulary IRI as a string slice, or `None` if not set.
    #[inline]
    pub fn vocab(&self) -> Option<&str> {
        self.vocab.as_deref()
    }

    /// Get the JSON-LD version if defined.
    ///
    /// # Returns
    ///
    /// The version number (typically 1.0 or 1.1), or `None` if not set.
    #[inline]
    pub fn version(&self) -> Option<f64> {
        self.version
    }

    /// Get the default text direction if defined.
    ///
    /// # Returns
    ///
    /// The direction ("ltr" or "rtl"), or `None` if not set.
    #[inline]
    pub fn direction(&self) -> Option<&str> {
        self.direction.as_deref()
    }

    /// Get the propagate flag if defined.
    ///
    /// # Returns
    ///
    /// The propagate flag value, or `None` if not set.
    #[inline]
    pub fn propagate(&self) -> Option<bool> {
        self.propagate
    }

    /// Get the import URI if defined.
    ///
    /// # Returns
    ///
    /// The import URI as a string slice, or `None` if not set.
    #[inline]
    pub fn import(&self) -> Option<&str> {
        self.import.as_deref()
    }

    /// Get the prefix mappings.
    ///
    /// # Returns
    ///
    /// A reference to the prefix-to-URI mapping.
    #[inline]
    pub fn prefixes(&self) -> &HashMap<String, String> {
        &self.prefixes
    }

    /// Add a prefix mapping.
    ///
    /// # Arguments
    ///
    /// * `prefix` - The prefix (e.g., "foaf")
    /// * `uri` - The base URI (e.g., "http://xmlns.com/foaf/0.1/")
    pub fn add_prefix(&mut self, prefix: impl Into<String>, uri: impl Into<String>) {
        self.prefixes.insert(prefix.into(), uri.into());
    }

    /// Set the base IRI.
    ///
    /// # Arguments
    ///
    /// * `base` - The base IRI
    pub fn set_base(&mut self, base: impl Into<String>) {
        self.base = Some(base.into());
    }

    /// Set the default vocabulary.
    ///
    /// # Arguments
    ///
    /// * `vocab` - The vocabulary IRI
    pub fn set_vocab(&mut self, vocab: impl Into<String>) {
        self.vocab = Some(vocab.into());
    }

    /// Compact a full URI using known prefixes.
    ///
    /// If the URI matches a known prefix, it will be compacted to the
    /// prefixed form (e.g., "http://xmlns.com/foaf/0.1/name" -> "foaf:name").
    ///
    /// # Arguments
    ///
    /// * `uri` - The URI to compact
    ///
    /// # Returns
    ///
    /// The compacted URI if a matching prefix is found, otherwise the original URI.
    ///
    /// # Example
    ///
    /// ```
    /// use toon_core::JsonLdContext;
    ///
    /// let mut ctx = JsonLdContext::new();
    /// ctx.add_prefix("foaf", "http://xmlns.com/foaf/0.1/");
    ///
    /// assert_eq!(ctx.compact_uri("http://xmlns.com/foaf/0.1/name"), "foaf:name");
    /// assert_eq!(ctx.compact_uri("http://other.org/term"), "http://other.org/term");
    /// ```
    pub fn compact_uri(&self, uri: &str) -> String {
        // Check each prefix for a match
        for (prefix, base_uri) in &self.prefixes {
            if uri.starts_with(base_uri) {
                let local = &uri[base_uri.len()..];
                return format!("{}:{}", prefix, local);
            }
        }
        uri.to_string()
    }

    /// Expand a prefixed URI to full form.
    ///
    /// If the URI is in prefixed form and the prefix is known, it will be
    /// expanded to the full URI.
    ///
    /// # Arguments
    ///
    /// * `prefixed` - The prefixed URI to expand (e.g., "foaf:name")
    ///
    /// # Returns
    ///
    /// The expanded URI if the prefix is known, otherwise the original value.
    ///
    /// # Example
    ///
    /// ```
    /// use toon_core::JsonLdContext;
    ///
    /// let mut ctx = JsonLdContext::new();
    /// ctx.add_prefix("foaf", "http://xmlns.com/foaf/0.1/");
    ///
    /// assert_eq!(ctx.expand_uri("foaf:name"), "http://xmlns.com/foaf/0.1/name");
    /// assert_eq!(ctx.expand_uri("unknown:term"), "unknown:term");
    /// ```
    pub fn expand_uri(&self, prefixed: &str) -> String {
        if let Some((prefix, local)) = prefixed.split_once(':') {
            // Don't expand URIs that look like full URLs
            if prefix.contains('/') || prefix.is_empty() {
                return prefixed.to_string();
            }
            if let Some(base_uri) = self.prefixes.get(prefix) {
                return format!("{}{}", base_uri, local);
            }
        }
        prefixed.to_string()
    }

    /// Check if the context has any prefix definitions.
    ///
    /// # Returns
    ///
    /// `true` if at least one prefix is defined, `false` otherwise.
    #[inline]
    pub fn has_prefixes(&self) -> bool {
        !self.prefixes.is_empty()
    }

    /// Check if the context is empty (no settings defined).
    ///
    /// # Returns
    ///
    /// `true` if no context settings are defined, `false` otherwise.
    pub fn is_empty(&self) -> bool {
        self.prefixes.is_empty()
            && self.base.is_none()
            && self.vocab.is_none()
            && self.version.is_none()
            && self.direction.is_none()
            && self.propagate.is_none()
            && self.import.is_none()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_new_context_is_empty() {
        let ctx = JsonLdContext::new();
        assert!(ctx.is_empty());
        assert!(!ctx.has_prefixes());
    }

    #[test]
    fn test_from_value_simple_context() {
        let value = json!({
            "@context": {
                "foaf": "http://xmlns.com/foaf/0.1/",
                "schema": "http://schema.org/"
            }
        });

        let ctx = JsonLdContext::from_value(&value);
        assert!(ctx.has_prefixes());
        assert_eq!(ctx.prefixes().len(), 2);
    }

    #[test]
    fn test_from_value_with_base_and_vocab() {
        let value = json!({
            "@context": {
                "@base": "http://example.org/",
                "@vocab": "http://schema.org/"
            }
        });

        let ctx = JsonLdContext::from_value(&value);
        assert_eq!(ctx.base(), Some("http://example.org/"));
        assert_eq!(ctx.vocab(), Some("http://schema.org/"));
    }

    #[test]
    fn test_from_value_with_version() {
        let value = json!({
            "@context": {
                "@version": 1.1,
                "name": "http://schema.org/name"
            }
        });

        let ctx = JsonLdContext::from_value(&value);
        assert_eq!(ctx.version(), Some(1.1));
    }

    #[test]
    fn test_from_value_with_direction() {
        let value = json!({
            "@context": {
                "@direction": "rtl"
            }
        });

        let ctx = JsonLdContext::from_value(&value);
        assert_eq!(ctx.direction(), Some("rtl"));
    }

    #[test]
    fn test_from_value_with_propagate() {
        let value = json!({
            "@context": {
                "@propagate": false
            }
        });

        let ctx = JsonLdContext::from_value(&value);
        assert_eq!(ctx.propagate(), Some(false));
    }

    #[test]
    fn test_from_value_with_import() {
        let value = json!({
            "@context": {
                "@import": "http://example.org/context.jsonld"
            }
        });

        let ctx = JsonLdContext::from_value(&value);
        assert_eq!(ctx.import(), Some("http://example.org/context.jsonld"));
    }

    #[test]
    fn test_from_value_with_term_definition_object() {
        let value = json!({
            "@context": {
                "name": {
                    "@id": "http://schema.org/name",
                    "@type": "@id"
                }
            }
        });

        let ctx = JsonLdContext::from_value(&value);
        assert_eq!(
            ctx.prefixes().get("name"),
            Some(&"http://schema.org/name".to_string())
        );
    }

    #[test]
    fn test_from_value_array_context() {
        let value = json!({
            "@context": [
                {"foaf": "http://xmlns.com/foaf/0.1/"},
                {"schema": "http://schema.org/"}
            ]
        });

        let ctx = JsonLdContext::from_value(&value);
        assert_eq!(ctx.prefixes().len(), 2);
        assert!(ctx.prefixes().contains_key("foaf"));
        assert!(ctx.prefixes().contains_key("schema"));
    }

    #[test]
    fn test_compact_uri() {
        let mut ctx = JsonLdContext::new();
        ctx.add_prefix("foaf", "http://xmlns.com/foaf/0.1/");
        ctx.add_prefix("schema", "http://schema.org/");

        assert_eq!(
            ctx.compact_uri("http://xmlns.com/foaf/0.1/name"),
            "foaf:name"
        );
        assert_eq!(ctx.compact_uri("http://schema.org/Person"), "schema:Person");
        assert_eq!(
            ctx.compact_uri("http://unknown.org/term"),
            "http://unknown.org/term"
        );
    }

    #[test]
    fn test_expand_uri() {
        let mut ctx = JsonLdContext::new();
        ctx.add_prefix("foaf", "http://xmlns.com/foaf/0.1/");

        assert_eq!(
            ctx.expand_uri("foaf:name"),
            "http://xmlns.com/foaf/0.1/name"
        );
        assert_eq!(ctx.expand_uri("unknown:term"), "unknown:term");
        // Full URIs should not be expanded
        assert_eq!(
            ctx.expand_uri("http://example.org/term"),
            "http://example.org/term"
        );
    }

    #[test]
    fn test_add_prefix() {
        let mut ctx = JsonLdContext::new();
        assert!(!ctx.has_prefixes());

        ctx.add_prefix("ex", "http://example.org/");
        assert!(ctx.has_prefixes());
        assert_eq!(
            ctx.prefixes().get("ex"),
            Some(&"http://example.org/".to_string())
        );
    }

    #[test]
    fn test_set_base() {
        let mut ctx = JsonLdContext::new();
        assert!(ctx.base().is_none());

        ctx.set_base("http://example.org/");
        assert_eq!(ctx.base(), Some("http://example.org/"));
    }

    #[test]
    fn test_set_vocab() {
        let mut ctx = JsonLdContext::new();
        assert!(ctx.vocab().is_none());

        ctx.set_vocab("http://schema.org/");
        assert_eq!(ctx.vocab(), Some("http://schema.org/"));
    }

    #[test]
    fn test_is_empty() {
        let mut ctx = JsonLdContext::new();
        assert!(ctx.is_empty());

        ctx.add_prefix("ex", "http://example.org/");
        assert!(!ctx.is_empty());
    }

    #[test]
    fn test_no_context_key() {
        let value = json!({
            "name": "Alice",
            "age": 30
        });

        let ctx = JsonLdContext::from_value(&value);
        assert!(ctx.is_empty());
    }

    #[test]
    fn test_string_context_reference() {
        let value = json!({
            "@context": "http://schema.org/"
        });

        let ctx = JsonLdContext::from_value(&value);
        assert_eq!(ctx.import(), Some("http://schema.org/"));
    }
}
