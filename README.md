# TOON-LD

High-performance serializer/parser for TOON-LD (Token-Oriented Object Notation for Linked Data) implemented in Rust with bindings for WebAssembly (npm) and Python (PyPI) for easy developer integration.

## Overview

TOON-LD extends TOON (Token-Oriented Object Notation) to handle Linked Data (RDF). It combines the token-saving "Tabular Arrays" of TOON with the `@context` expansion of JSON-LD and the ability to represent complex RDF structures while minimizing token usage which is critical for LLM applications. Moreover, TOON-LD maintains human readability and ease of parsing. LLMs can benefit from the reduced token count when processing multiple queries as TOON-LD will fit more data into their context windows, leading to better performance and lower costs. Moreover, the Linked Data paradigm allows LLMs to interlink (via a tool call) and reference external knowledge bases seamlessly if they need to augment their knowledge. Thus, the ontological expressiveness of RDF combined with the token efficiency of TOON makes TOON-LD a powerful format for LLM-centric applications.

## Features

- **Tabular Arrays**: Efficiently serialize arrays of uniform objects as CSV-like rows
- **Primitive Arrays**: Compact representation for simple value lists
- **JSON-LD Context Support**: Automatic URI prefix compaction/expansion
- **Cross-Platform**: Native Rust, WebAssembly, and Python bindings

## Specification (v3.0)

### Tabular Arrays

If an array contains objects, serialize using the union of all keys:

```
key[N]{field1,field2}:
  val1, val2
  val3, val4
```

Where `N` is the array length. If an object is missing a field, `null` is inserted to maintain CSV alignment:

```json
{
  "@context": {
    "schema": "http://schema.org/"
  },
  "schema:items": [
    {"schema:name": "Alice", "schema:age": 30},
    {"schema:name": "Bob", "schema:email": "bob@example.com"}
  ]
}
```

Becomes:

```
@context:
  schema: http://schema.org/
schema:items[2]{schema:age,schema:email,schema:name}:
  30, null, Alice
  null, "bob@example.com", Bob
```

### Primitive Arrays

Serialize as `key[N]: val1, val2` (inline) or indented lines (if long).

### Formatting Rules

- **Indentation**: 2 spaces for nesting
- **No braces/brackets**: Except in headers (`key[N]{...}:`)
- **Quoting**: Strings are unquoted unless they contain `,`, `:`, `|`, or start/end with whitespace

## Benchmarks

Size and token savings comparison across different dataset scales:

| Records | JSON-LD | TOON-LD | Size Saved | Tokens Saved |
|---------|---------|---------|------------|--------------|
| 10 | 2,249 B | 1,425 B | **36.6%** | **51.6%** |
| 100 | 20,208 B | 11,375 B | **43.7%** | **57.8%** |
| 1,000 | 202,497 B | 113,565 B | **43.9%** | **58.5%** |
| 10,000 | 2,052,356 B | 1,162,425 B | **43.4%** | **58.6%** |

Key findings:
- **Size savings scale well**: 37-44% reduction across all dataset sizes
- **Token savings are even better**: 52-59% fewer tokens
- **Tabular arrays shine**: The `@graph[N]{fields}:` format eliminates repetitive keys

The token reduction is especially valuable for LLM context windows where every token counts.

## Installation

### Rust (Cargo)

```toml
[dependencies]
toon-core = "0.1"
```

### CLI Tool

```bash
cargo install toon-cli
```

### Python (PyPI)

```bash
pip install toon-ld
```

### JavaScript/TypeScript (npm)

```bash
npm install toon-ld
```

## Usage

### Rust

```rust
use toon_core::{jsonld_to_toon, toon_to_jsonld};

// Convert JSON-LD to TOON-LD
let json_ld = r#"{
    "@context": {
        "foaf": "http://xmlns.com/foaf/0.1/"
    },
    "http://xmlns.com/foaf/0.1/name": "Alice",
    "http://xmlns.com/foaf/0.1/age": 30
}"#;
let toon = jsonld_to_toon(json_ld).unwrap();
// URIs are compacted using @context: foaf:name, foaf:age
// 
// Convert TOON-LD back to JSON-LD
let back = toon_to_jsonld(&toon).unwrap();
```

### Python

```python
import toon_ld

# Convert JSON-LD to TOON-LD
json_str = '''
{
  "@context": {
    "foaf": "http://xmlns.com/foaf/0.1/",
    "schema": "http://schema.org/"
  },
  "http://xmlns.com/foaf/0.1/name": "Alice",
  "http://schema.org/age": 30
}
'''
toon_str = toon_ld.convert_jsonld_to_toon(json_str)
# Output uses compact URIs: foaf:name, schema:age
# 
# Convert TOON-LD to JSON-LD
json_back = toon_ld.convert_toon_to_jsonld(toon_str)

# Work with Python dicts directly
data = {
    "@context": {"foaf": "http://xmlns.com/foaf/0.1/"},
    "users": [
        {"foaf:name": "Alice", "foaf:age": 30},
        {"foaf:name": "Bob", "foaf:age": 25}
    ]
}
toon_str = toon_ld.serialize_to_toon(data)

parsed = toon_ld.parse_toon(toon_str)
```

### JavaScript/TypeScript

```javascript
import { convert_jsonld_to_toon, convert_toon_to_jsonld } from 'toon-ld';

// Convert JSON-LD to TOON-LD
const jsonLd = JSON.stringify({
  "@context": {
    "foaf": "http://xmlns.com/foaf/0.1/"
  },
  "http://xmlns.com/foaf/0.1/name": "Alice",
  "http://xmlns.com/foaf/0.1/knows": [
    {"http://xmlns.com/foaf/0.1/name": "Bob"},
    {"http://xmlns.com/foaf/0.1/name": "Carol"}
  ]
});
const toon = convert_jsonld_to_toon(jsonLd);

// Convert TOON-LD to JSON-LD
const jsonBack = convert_toon_to_jsonld(toon);
```

## CLI Usage

```bash
# Convert JSON-LD to TOON-LD
toon-ld convert -i input.jsonld -o output.toon

# Convert TOON-LD to JSON-LD
toon-ld convert -i input.toon -o output.jsonld

# Auto-detect format and convert (stdin/stdout)
cat data.jsonld | toon-ld convert --to toon

# Validate a file
toon-ld validate -i data.toon --verbose

# Show size/token statistics
toon-ld stats -i data.jsonld --tokens

# Run size/token savings benchmark (tests 10, 100, 1000, 10000 records)
toon-ld benchmark --max-records 10000
```

## Example

### Input (JSON-LD)

```json
{
  "@context": {
    "foaf": "http://xmlns.com/foaf/0.1/"
  },
  "@id": "http://example.org/dataset",
  "@type": "Dataset",
  "@graph": [
    {"@id": "http://example.org/1", "@type": "foaf:Person", "foaf:name": "Alice", "foaf:age": 30},
    {"@id": "http://example.org/2", "@type": "foaf:Person", "foaf:name": "Bob", "foaf:age": 25}
  ]
}
```

### Output (TOON-LD)

```
@context:
  foaf: http://xmlns.com/foaf/0.1/
@id: "http://example.org/dataset"
@type: Dataset
@graph[2]{@id,@type,foaf:age,foaf:name}:
  "http://example.org/1", "foaf:Person", 30, Alice
  "http://example.org/2", "foaf:Person", 25, Bob
```

## JSON-LD Keyword Support

TOON-LD supports all JSON-LD 1.1 keywords.

### Core Keywords

| Keyword | Description | TOON-LD Format |
|---------|-------------|----------------|
| `@context` | Prefix definitions | Serialized first, nested key-value pairs |
| `@base` | Base IRI for relative references | `@base: <uri>` |
| `@vocab` | Default vocabulary | `@vocab: <uri>` |
| `@id` | Resource identifier | `@id: <uri>` |
| `@type` | Type declaration | `@type: <type>` or `@type[N]: ...` for arrays |
| `@graph` | Named graph | `@graph[N]{fields}: ...` tabular format |
| `@value` | Explicit value node | `"value"@lang` or `"value"^^type` |
| `@language` | Language tag | Combined with @value as `"text"@en` |
| `@list` | Ordered list | `@list[N]: ...` |
| `@set` | Explicit set | `@set[N]: ...` |
| `@reverse` | Reverse properties | `@reverse:` with nested properties |

### JSON-LD 1.1 Extended Keywords

| Keyword | Description | TOON-LD Format |
|---------|-------------|----------------|
| `@version` | JSON-LD version | `@version: 1.1` |
| `@direction` | Text direction | `@direction: rtl` or `"text"@en:rtl` |
| `@container` | Container type | `@container: @set` or `@container[N]: ...` |
| `@index` | Index property | `@index: <value>` |
| `@included` | Included nodes | `@included[N]{fields}: ...` tabular format |
| `@nest` | Nested properties | `@nest:` with nested object |
| `@prefix` | Prefix flag | `@prefix: true` |
| `@propagate` | Context propagation | `@propagate: false` |
| `@protected` | Protected term | `@protected: true` |
| `@import` | Import external context | `@import: <uri>` |
| `@json` | JSON literal type | `"value"^^@json` |
| `@none` | Default index value | `@none:` with nested object |

### Value Node Examples

Language-tagged strings:
```
title:
  "Bonjour"@fr
```

Typed literals:
```
date:
  "2024-01-15"^^xsd:date
```

## Project Structure

```
toon-ld/
├── Cargo.toml              # Workspace configuration
├── toon-core/              # Pure Rust implementation
│   ├── Cargo.toml
│   └── src/lib.rs
├── toon-cli/               # Command-line tool
│   ├── Cargo.toml
│   └── src/main.rs
├── toon-wasm/              # WebAssembly bindings
│   ├── Cargo.toml
│   └── src/lib.rs
├── toon-py/                # Python bindings (PyO3)
│   ├── Cargo.toml
│   ├── pyproject.toml
│   └── src/lib.rs
└── examples/               # Sample files
    ├── sample.jsonld
    └── sample.toon
```

## Building

### Build All

```bash
cargo build --release
```

### Build WASM

```bash
cd toon-wasm
wasm-pack build --target web
```

### Build Python Wheel

```bash
cd toon-py
maturin build --release
```

## Testing

```bash
cargo test --workspace
```

## License

The library is licensed with MIT License. See [LICENSE](LICENSE) for details.