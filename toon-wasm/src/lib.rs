//! TOON-LD WASM Bindings
//!
//! WebAssembly bindings for TOON-LD serializer/parser using wasm-bindgen.

use wasm_bindgen::prelude::*;

/// Initialize panic hook for better error messages in browser console
#[wasm_bindgen(start)]
pub fn init() {
    #[cfg(feature = "console_error_panic_hook")]
    console_error_panic_hook::set_once();
}

/// Convert JSON-LD string to TOON-LD format
///
/// # Arguments
/// * `json` - A JSON or JSON-LD formatted string
///
/// # Returns
/// * TOON-LD formatted string on success
/// * Error message on failure
#[wasm_bindgen]
pub fn convert_jsonld_to_toonld(json: String) -> Result<String, JsValue> {
    toon_core::jsonld_to_toonld(&json).map_err(|e| JsValue::from_str(&e.to_string()))
}

/// Convert TOON-LD string to JSON-LD format
///
/// # Arguments
/// * `toon` - A TOON-LD formatted string
///
/// # Returns
/// * JSON-LD formatted string (pretty-printed) on success
/// * Error message on failure
#[wasm_bindgen]
pub fn convert_toonld_to_jsonld(toon: String) -> Result<String, JsValue> {
    toon_core::toonld_to_jsonld(&toon).map_err(|e| JsValue::from_str(&e.to_string()))
}

/// Validate a TOON-LD string
///
/// # Arguments
/// * `toon` - A TOON-LD formatted string
///
/// # Returns
/// * `true` if the string is valid TOON-LD
/// * `false` otherwise
#[wasm_bindgen]
pub fn validate_toonld(toon: String) -> bool {
    toon_core::toonld_to_jsonld(&toon).is_ok()
}

/// Validate a JSON-LD string
///
/// # Arguments
/// * `json` - A JSON or JSON-LD formatted string
///
/// # Returns
/// * `true` if the string is valid JSON
/// * `false` otherwise
#[wasm_bindgen]
pub fn validate_json(json: String) -> bool {
    serde_json::from_str::<serde_json::Value>(&json).is_ok()
}

/// Tests that use toon_core directly (work on all platforms)
#[cfg(test)]
mod tests {
    #[test]
    fn test_jsonld_to_toonld() {
        let json = r#"{"name": "Alice", "age": 30}"#;
        let result = toon_core::jsonld_to_toonld(json);
        assert!(result.is_ok());
        let toon = result.unwrap();
        assert!(toon.contains("name: Alice"));
        assert!(toon.contains("age: 30"));
    }

    #[test]
    fn test_toonld_to_jsonld() {
        let toon = "name: Alice\nage: 30";
        let result = toon_core::toonld_to_jsonld(toon);
        assert!(result.is_ok());
        let json = result.unwrap();
        assert!(json.contains("\"name\""));
        assert!(json.contains("\"Alice\""));
    }

    #[test]
    fn test_validate_toonld() {
        let valid = "name: Alice\nage: 30";
        assert!(toon_core::toonld_to_jsonld(valid).is_ok());
    }

    #[test]
    fn test_validate_json() {
        let valid = r#"{"name": "Alice"}"#;
        let invalid = r#"{"name": }"#;
        assert!(serde_json::from_str::<serde_json::Value>(valid).is_ok());
        assert!(serde_json::from_str::<serde_json::Value>(invalid).is_err());
    }

    #[test]
    fn test_tabular_array_conversion() {
        let json = r#"{
            "users": [
                {"id": 1, "name": "Alice"},
                {"id": 2, "name": "Bob"}
            ]
        }"#;
        let result = toon_core::jsonld_to_toonld(json);
        assert!(result.is_ok());
        let toon = result.unwrap();
        assert!(toon.contains("users[2]"));
    }

    #[test]
    fn test_invalid_json_error() {
        let invalid_json = r#"{"name": }"#;
        let result = toon_core::jsonld_to_toonld(invalid_json);
        assert!(result.is_err());
    }
}
