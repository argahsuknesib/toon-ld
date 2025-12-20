# toon-ld

[![Crates.io](https://img.shields.io/crates/v/toon-ld)](https://crates.io/crates/toon-ld)
[![Documentation](https://docs.rs/toon-ld/badge.svg)](https://docs.rs/toon-ld)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

**Token-Oriented Object Notation for Linked Data** — A compact RDF serialization format that achieves 40-60% token reduction compared to JSON-LD, making it ideal for LLM applications and bandwidth-constrained environments.

TOON-LD extends TOON in the same way that JSON-LD extends JSON: **every valid TOON-LD document is also a valid TOON document**. Base TOON parsers can process TOON-LD without modification, while TOON-LD processors interpret `@-prefixed` keys according to JSON-LD semantics.

## Features

- **40-60% Token Reduction**: Fewer tokens means lower LLM costs and more data in context windows
- **Full JSON-LD Compatibility**: Round-trip conversion without data loss
- **Tabular Arrays**: Serialize arrays of objects as CSV-like rows with shared headers
- **All JSON-LD 1.1 Keywords**: Complete support for `@context`, `@graph`, `@id`, `@type`, value nodes, etc.
- **High Performance**: Optimized serialization with automatic tabular array detection
- **Comprehensive Error Messages**: Detailed errors with line numbers and context

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
toon-ld = "0.1"
```

## Quick Start

```rust
use toon_ld::{jsonld_to_toon, toon_to_jsonld};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Convert JSON-LD to TOON-LD
    let json_ld = r#"{
        "@context": {"foaf": "http://xmlns.com/foaf/0.1/"},
        "foaf:name": "Alice",
        "foaf:age": 30
    }"#;
    
    let toon = jsonld_to_toon(json_ld)?;
    println!("TOON-LD:\n{}", toon);
    // Output:
    // @context:
    //   foaf: http://xmlns.com/foaf/0.1/
    // foaf:name: Alice
    // foaf:age: 30
    
    // Convert back to JSON-LD
    let back_to_json = toon_to_jsonld(&toon)?;
    
    Ok(())
}
```

## Tabular Arrays - The Key Feature

TOON-LD's most powerful feature is efficient serialization of arrays of objects:

**JSON-LD (repetitive keys):**
```json
{
  "@context": {"foaf": "http://xmlns.com/foaf/0.1/"},
  "@graph": [
    {"@id": "ex:1", "@type": "foaf:Person", "foaf:name": "Alice", "foaf:age": 30},
    {"@id": "ex:2", "@type": "foaf:Person", "foaf:name": "Bob", "foaf:age": 25},
    {"@id": "ex:3", "@type": "foaf:Person", "foaf:name": "Carol", "foaf:age": 28}
  ]
}
```

**TOON-LD (shared headers):**
```
@context:
  foaf: http://xmlns.com/foaf/0.1/
@graph[3]{@id,@type,foaf:age,foaf:name}:
  ex:1, foaf:Person, 30, Alice
  ex:2, foaf:Person, 25, Bob
  ex:3, foaf:Person, 28, Carol
```

Notice how object keys appear once in the header instead of repeating for each object.

### Code Example

```rust
use toon_ld::jsonld_to_toon;

let json_ld = r#"{
    "@context": {"foaf": "http://xmlns.com/foaf/0.1/"},
    "@graph": [
        {"@id": "ex:1", "foaf:name": "Alice", "foaf:age": 30},
        {"@id": "ex:2", "foaf:name": "Bob", "foaf:age": 25}
    ]
}"#;

let toon = jsonld_to_toon(json_ld)?;
// Automatically serializes as tabular format when beneficial
```

## Value Nodes

Language tags and datatypes use standard TOON object or tabular syntax:

```rust
use toon_ld::jsonld_to_toon;

let json_ld = r#"{
    "@context": {
        "dc": "http://purl.org/dc/terms/",
        "xsd": "http://www.w3.org/2001/XMLSchema#"
    },
    "dc:title": [
        {"@value": "Bonjour", "@language": "fr"},
        {"@value": "Hello", "@language": "en"}
    ],
    "dc:date": {"@value": "2024-01-15", "@type": "xsd:date"}
}"#;

let toon = jsonld_to_toon(json_ld)?;
// Tabular format for language tags:
// dc:title[2]{@value,@language}:
//   Bonjour,fr
//   Hello,en
```

## API Reference

### Main Functions

- `jsonld_to_toon(json: &str) -> Result<String, ToonError>` - Convert JSON-LD to TOON-LD
- `toon_to_jsonld(toon: &str) -> Result<String, ToonError>` - Convert TOON-LD to JSON-LD
- `parse_toon(toon: &str) -> Result<Value, ToonError>` - Parse TOON-LD to serde_json::Value
- `serialize_to_toon(value: &Value) -> Result<String, ToonError>` - Serialize Value to TOON-LD with automatic tabular optimization

### Error Handling

All functions return `Result<T, ToonError>` where `ToonError` provides detailed error messages including line numbers and context.

```rust
use toon_ld::toon_to_jsonld;

let result = toon_to_jsonld("invalid: [unclosed");
match result {
    Ok(json) => println!("Success: {}", json),
    Err(e) => eprintln!("Error: {}", e), // Detailed error with line number
}
```

## Performance & Benchmarks

Real-world token savings across different dataset sizes:

| Records | JSON-LD Size | TOON-LD Size | Size Saved | Tokens Saved |
|---------|--------------|--------------|------------|--------------|
| 10      | 2,249 B      | 1,425 B      | **36.6%**  | **51.6%**    |
| 100     | 20,208 B     | 11,375 B     | **43.7%**  | **57.8%**    |
| 1,000   | 202,497 B    | 113,565 B    | **43.9%**  | **58.5%**    |
| 10,000  | 2,052,356 B  | 1,162,425 B  | **43.4%**  | **58.6%**    |

Token savings scale well and are especially valuable for LLM context windows.

## Cross-Platform Support

TOON-LD is also available for:
- **Python**: `pip install toon-ld` - [PyPI](https://pypi.org/project/toon-ld/)
- **JavaScript/TypeScript**: `npm install toon-ld` - [npm](https://www.npmjs.com/package/toon-ld)
- **CLI**: `cargo install toon-cli` - Command-line conversion tool

## Documentation

- **[Full Specification](https://github.com/argahsuknesib/toon-ld)** - Complete TOON-LD specification
- **[API Documentation](https://docs.rs/toon-ld)** - Detailed API reference
- **[GitHub Repository](https://github.com/argahsuknesib/toon-ld)** - Source code and examples

## Examples

### Context Support

```rust
let json_ld = r#"{
    "@context": {
        "foaf": "http://xmlns.com/foaf/0.1/",
        "schema": "http://schema.org/"
    },
    "foaf:name": "Alice",
    "schema:email": "alice@example.com"
}"#;

let toon = jsonld_to_toon(json_ld)?;
// Automatically handles context and URI compaction
```

### Nested Objects

```rust
let json_ld = r#"{
    "@context": {"vcard": "http://www.w3.org/2006/vcard/ns#"},
    "vcard:name": "Alice",
    "vcard:address": {
        "vcard:street": "123 Main St",
        "vcard:city": "Portland"
    }
}"#;

let toon = jsonld_to_toon(json_ld)?;
// Preserves nesting with proper indentation
```

### Mixed Data Types

```rust
let json_ld = r#"{
    "name": "Alice",
    "age": 30,
    "active": true,
    "score": 98.5,
    "tags": ["rust", "json-ld", "rdf"]
}"#;

let toon = jsonld_to_toon(json_ld)?;
// Handles all JSON data types correctly
```

## Related Crates

- `toon-core` - Core serialization/parsing logic (this crate re-exports from toon-core)
- `toon-cli` - Command-line tool for TOON-LD conversion and benchmarking

## License

MIT License - See [LICENSE](https://github.com/argahsuknesib/toon-ld/blob/main/LICENSE) for details.

## Contributing

Contributions are welcome! Please see the [Contributing Guide](https://github.com/argahsuknesib/toon-ld/blob/main/CONTRIBUTING.md) for details.

## Citation

If you use TOON-LD in your research, please cite:

```bibtex
@software{toon-ld,
  title = {TOON-LD: Token-Oriented Object Notation for Linked Data},
  author = {Bisen, Kushagra Singh},
  year = {2025},
  url = {https://github.com/argahsuknesib/toon-ld}
}
```
