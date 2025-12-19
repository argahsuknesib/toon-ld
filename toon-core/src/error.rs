//! Error types for TOON-LD operations.

use thiserror::Error;

/// Errors that can occur during TOON-LD operations
#[derive(Error, Debug)]
pub enum ToonError {
    /// Error parsing JSON input
    #[error("JSON parsing error: {0}")]
    JsonError(#[from] serde_json::Error),

    /// Error parsing TOON-LD input
    #[error("TOON parsing error at line {line}: {message}")]
    ParseError {
        /// Line number where the error occurred (1-based)
        line: usize,
        /// Description of the parsing error
        message: String,
    },

    /// Error during serialization
    #[error("Serialization error: {0}")]
    SerializeError(String),

    /// Invalid tabular array structure
    #[error("Invalid tabular array: {0}")]
    InvalidTabularArray(String),
}

impl ToonError {
    /// Create a new parse error at the given line
    pub fn parse_error(line: usize, message: impl Into<String>) -> Self {
        ToonError::ParseError {
            line,
            message: message.into(),
        }
    }

    /// Create a new serialization error
    pub fn serialize_error(message: impl Into<String>) -> Self {
        ToonError::SerializeError(message.into())
    }

    /// Create a new invalid tabular array error
    pub fn invalid_tabular_array(message: impl Into<String>) -> Self {
        ToonError::InvalidTabularArray(message.into())
    }
}

/// Result type for TOON-LD operations
pub type Result<T> = std::result::Result<T, ToonError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_error_display() {
        let err = ToonError::parse_error(42, "unexpected token");
        assert_eq!(
            err.to_string(),
            "TOON parsing error at line 42: unexpected token"
        );
    }

    #[test]
    fn test_serialize_error_display() {
        let err = ToonError::serialize_error("invalid value");
        assert_eq!(err.to_string(), "Serialization error: invalid value");
    }

    #[test]
    fn test_invalid_tabular_array_display() {
        let err = ToonError::invalid_tabular_array("mismatched field count");
        assert_eq!(
            err.to_string(),
            "Invalid tabular array: mismatched field count"
        );
    }

    #[test]
    fn test_json_error_conversion() {
        let json_err = serde_json::from_str::<serde_json::Value>("invalid json").unwrap_err();
        let toon_err: ToonError = json_err.into();
        assert!(toon_err.to_string().starts_with("JSON parsing error:"));
    }
}
