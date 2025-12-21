# toon-core

> **DEPRECATED: Please use [`toon-ld`](https://crates.io/crates/toon-ld) instead.**
>
> This crate is no longer maintained. All functionality has been moved to the `toon-ld` crate, which provides a better user-facing API and naming scheme.

Core serialization and parsing logic for the TOON-LD (Token-Oriented Object Notation for Linked Data) format.

This crate provides the fundamental conversion algorithms between JSON-LD and TOON-LD formats, achieving 40-60% token reduction while maintaining full semantic compatibility with JSON-LD.

## Features

- **JSON-LD to TOON-LD conversion** with automatic tabular array optimization
- **TOON-LD to JSON-LD parsing** with full round-trip compatibility
- **Context handling** for URI compaction and expansion
- **Value node support** for language tags and datatypes
- **Optimized parsing** with efficient data structures
- **Comprehensive error handling** with detailed error messages

## Usage

Add this to your `Cargo.toml`:

```toml
[dependencies]
toon-core = "0.1"
```

### Basic Example

```rust
use toon_core::{jsonld_to_toon, toon_to_jsonld};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Convert JSON-LD to TOON-LD
    let json_ld = r#"{
        "@context": {"foaf": "http://xmlns.com/foaf/0.1/"},
        "foaf:name": "Alice"
    }"#;
    
    let toon = jsonld_to_toon(json_ld)?;
    println!("TOON-LD:\n{}", toon);
    
    // Convert back to JSON-LD
    let back_to_json = toon_to_jsonld(&toon)?;
    println!("JSON-LD:\n{}", back_to_json);
    
    Ok(())
}
```

### Tabular Arrays

TOON-LD's key feature is efficient serialization of arrays of objects:

```rust
use toon_core::jsonld_to_toon;

let json_ld = r#"{
    "@context": {"foaf": "http://xmlns.com/foaf/0.1/"},
    "@graph": [
        {"@id": "ex:1", "foaf:name": "Alice", "foaf:age": 30},
        {"@id": "ex:2", "foaf:name": "Bob", "foaf:age": 25}
    ]
}"#;

let toon = jsonld_to_toon(json_ld)?;
// Output uses tabular format:
// @graph[2]{@id,foaf:age,foaf:name}:
//   ex:1, 30, Alice
//   ex:2, 25, Bob
```

## API Reference

### Main Functions

- `jsonld_to_toon(json: &str) -> Result<String, ToonError>` - Convert JSON-LD to TOON-LD
- `toon_to_jsonld(toon: &str) -> Result<String, ToonError>` - Convert TOON-LD to JSON-LD
- `parse_toon(toon: &str) -> Result<Value, ToonError>` - Parse TOON-LD to serde_json::Value
- `serialize_to_toon(value: &Value) -> Result<String, ToonError>` - Serialize Value to TOON-LD

### Error Handling

All functions return `Result<T, ToonError>` where `ToonError` provides detailed error messages including line numbers and context.

## Performance

TOON-LD achieves:
- **40-60% token reduction** compared to JSON-LD
- **Efficient parsing** with structured error handling
- **Optimized serialization** with automatic tabular array detection

## Specification

For the complete TOON-LD specification, see the [main repository](https://github.com/argahsuknesib/toon-ld).

## Related Crates

- `toon-cli` - Command-line tool for TOON-LD conversion and benchmarking

## License

MIT License - See [LICENSE](../LICENSE) for details.