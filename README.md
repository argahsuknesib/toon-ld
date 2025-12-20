# TOON-LD

[![npm](https://img.shields.io/npm/v/toon-ld)](https://www.npmjs.com/package/toon-ld)
[![PyPI](https://img.shields.io/pypi/v/toon-ld)](https://pypi.org/project/toon-ld/)
[![Crates.io](https://img.shields.io/crates/v/toon-ld)](https://crates.io/crates/toon-ld)

**Token-Oriented Object Notation for Linked Data** — A compact RDF serialization format that achieves 40-60% token reduction compared to JSON-LD, making it ideal for LLM applications and bandwidth-constrained environments.

TOON-LD extends TOON in the same way that JSON-LD extends JSON: **every valid TOON-LD document is also a valid TOON document**. Base TOON parsers can process TOON-LD without modification, while TOON-LD processors interpret `@-prefixed` keys according to JSON-LD semantics.

## Why TOON-LD?

TOON-LD combines the semantic expressiveness of RDF/JSON-LD with radical token efficiency through tabular arrays. By eliminating repetitive keys and using CSV-like rows for uniform data, TOON-LD fits more information into LLM context windows while maintaining human readability.

## Features

- **Pure TOON Extension**: Every TOON-LD document is valid TOON (like JSON-LD extends JSON)
- **Tabular Arrays**: Serialize arrays of objects as CSV-like rows with shared headers
- **40-60% Token Reduction**: Fewer tokens means lower costs and more data in context
- **Full JSON-LD Compatibility**: Round-trip conversion without data loss
- **All JSON-LD 1.1 Keywords**: Complete support for `@context`, `@graph`, `@id`, `@type`, value nodes, etc.
- **Cross-Platform**: Rust, WebAssembly (npm), and Python (PyPI) implementations
- **High Performance**: Optimized serialization with automatic tabular array detection

## Benchmarks

Real-world token savings across different dataset sizes:

| Records | JSON-LD Size | TOON-LD Size | Size Saved | Tokens Saved |
|---------|--------------|--------------|------------|--------------|
| 10      | 2,249 B      | 1,425 B      | **36.6%**  | **51.6%**    |
| 100     | 20,208 B     | 11,375 B     | **43.7%**  | **57.8%**    |
| 1,000   | 202,497 B    | 113,565 B    | **43.9%**  | **58.5%**    |
| 10,000  | 2,052,356 B  | 1,162,425 B  | **43.4%**  | **58.6%**    |

**Key takeaway**: Token savings scale well and are especially valuable for LLM context windows.

## Quick Example

**JSON-LD:**
```json
{
  "@context": {
    "foaf": "http://xmlns.com/foaf/0.1/"
  },
  "@graph": [
    {"@id": "ex:1", "@type": "foaf:Person", "foaf:name": "Alice", "foaf:age": 30},
    {"@id": "ex:2", "@type": "foaf:Person", "foaf:name": "Bob", "foaf:age": 25}
  ]
}
```

**TOON-LD:**
```
@context:
  foaf: http://xmlns.com/foaf/0.1/
@graph[2]{@id,@type,foaf:age,foaf:name}:
  ex:1, foaf:Person, 30, Alice
  ex:2, foaf:Person, 25, Bob
```

Notice how object keys appear once in the header instead of repeating for each object.

## How TOON-LD Extends TOON

Just as JSON-LD extends JSON by adding semantic meaning to certain key names (those starting with `@`), TOON-LD extends TOON the same way:

- **No new syntax**: TOON-LD uses only standard TOON syntax (objects, arrays, tabular format)
- **Semantic interpretation**: Keys like `@context`, `@id`, `@type` have special JSON-LD meaning
- **Full compatibility**: Any TOON parser can parse TOON-LD documents
- **Value nodes**: Language tags and datatypes use tabular format for efficiency

Example value node with language tag:
```
title[2]{@value,@language}:
  The Hobbit,en
  Der Hobbit,de
```

This is standard TOON tabular syntax that base TOON parsers handle natively, while TOON-LD processors interpret it as JSON-LD value nodes.

## Installation

### Rust
```toml
[dependencies]
toon-ld = "0.2"
```

### CLI
```bash
cargo install toon-cli
```

### Python
```bash
pip install toon-ld
```

### JavaScript/TypeScript
```bash
npm install toon-ld
```

## Quick Start

### CLI
```bash
# Convert JSON-LD to TOON-LD
toon-ld convert -i data.jsonld -o data.toon

# Convert back to JSON-LD
toon-ld convert -i data.toon -o data.jsonld

# Run benchmark
toon-ld benchmark --max-records 10000
```

### Rust
```rust
use toon_ld::{jsonld_to_toonld, toonld_to_jsonld};

let json_ld = r#"{"@context": {"foaf": "http://xmlns.com/foaf/0.1/"}, "foaf:name": "Alice"}"#;
let toon = jsonld_to_toonld(json_ld)?;
let back = toonld_to_jsonld(&toon)?;
```

### Python
```python
import toon_ld

json_ld = '{"@context": {"foaf": "http://xmlns.com/foaf/0.1/"}, "foaf:name": "Alice"}'
toon_str = toon_ld.convert_jsonld_to_toonldldld(json_ld)
json_str = toon_ld.convert_toonld_to_jsonld(toon_str)
```

### JavaScript
```javascript
import { convert_jsonld_to_toonldldld, convert_toonld_to_jsonld } from 'toon-ld';

const jsonLd = '{"@context": {"foaf": "http://xmlns.com/foaf/0.1/"}, "foaf:name": "Alice"}';
const toon = convert_jsonld_to_toonldldld(jsonLd);
const json = convert_toonld_to_jsonld(toon);
```
 

## Key Concepts

### Tabular Arrays
Arrays of objects share a header with field names, followed by CSV-like rows:
```
@context:
  foaf: http://xmlns.com/foaf/0.1/
  vcard: http://www.w3.org/2006/vcard/ns#
foaf:knows[3]{foaf:name,foaf:age,vcard:locality}:
  Alice, 30, null
  Bob, null, Portland
  Carol, 28, Seattle
```

### Value Nodes
Language tags and datatypes use standard TOON object or tabular syntax:
```
@context:
  dc: http://purl.org/dc/terms/
  schema: http://schema.org/
  xsd: http://www.w3.org/2001/XMLSchema#
dc:title:
  @value: Bonjour
  @language: fr
schema:datePublished:
  @value: "2024-01-15"
  @type: xsd:date
```

Or using tabular format for multiple values:
```
dc:titles[2]{@value,@language}:
  Bonjour,fr
  Hello,en
```

### Context Support
Automatic URI compaction using `@context`:
```
@context:
  foaf: http://xmlns.com/foaf/0.1/
foaf:name: Alice
```

## Project Structure

- `toon-core/` - Core Rust implementation
- `toon-cli/` - Command-line tool
- `toon-wasm/` - WebAssembly bindings (npm)
- `toon-py/` - Python bindings (PyPI)

## Building from Source

```bash
# Build all workspace members
cargo build --release

# Run tests
cargo test --workspace

# Build WASM package
cd toon-wasm && wasm-pack build --target web

# Build Python wheel
cd toon-py && maturin build --release
```

## License

MIT License - See [LICENSE](LICENSE) for details.
