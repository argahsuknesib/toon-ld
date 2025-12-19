//! # TOON-LD
//!
//! **Token-Oriented Object Notation for Linked Data** — A compact RDF serialization format
//! that achieves 40-60% token reduction compared to JSON-LD.
//!
//! This crate provides high-level conversion functions between JSON-LD and TOON-LD formats.
//! TOON-LD extends TOON in the same way that JSON-LD extends JSON: every valid TOON-LD
//! document is also a valid TOON document.
//!
//! ## Quick Start
//!
//! ```rust
//! use toon_ld::{jsonld_to_toon, toon_to_jsonld};
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! // Convert JSON-LD to TOON-LD
//! let json_ld = r#"{
//!     "@context": {"foaf": "http://xmlns.com/foaf/0.1/"},
//!     "foaf:name": "Alice"
//! }"#;
//!
//! let toon = jsonld_to_toon(json_ld)?;
//! println!("TOON-LD:\n{}", toon);
//!
//! // Convert back to JSON-LD
//! let back_to_json = toon_to_jsonld(&toon)?;
//! # Ok(())
//! # }
//! ```
//!
//! ## Tabular Arrays
//!
//! TOON-LD's key feature is efficient serialization of arrays of objects:
//!
//! ```rust
//! use toon_ld::jsonld_to_toon;
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! let json_ld = r#"{
//!     "@context": {"foaf": "http://xmlns.com/foaf/0.1/"},
//!     "@graph": [
//!         {"@id": "ex:1", "foaf:name": "Alice", "foaf:age": 30},
//!         {"@id": "ex:2", "foaf:name": "Bob", "foaf:age": 25}
//!     ]
//! }"#;
//!
//! let toon = jsonld_to_toon(json_ld)?;
//! // Output uses tabular format with shared headers:
//! // @graph[2]{@id,foaf:age,foaf:name}:
//! //   ex:1, 30, Alice
//! //   ex:2, 25, Bob
//! # Ok(())
//! # }
//! ```
//!
//! ## Features
//!
//! - **40-60% token reduction** compared to JSON-LD
//! - **Full JSON-LD compatibility** with round-trip conversion
//! - **Tabular arrays** for efficient serialization of uniform data
//! - **Context support** for URI compaction
//! - **Value nodes** with language tags and datatypes
//! - **Zero-copy parsing** where possible

// Re-export everything from toon-core
pub use toon_core::*;

// Re-export commonly used types from dependencies
pub use serde_json::Value;
