# TOON-LD

**Token-Oriented Object Notation for Linked Data** — A compact RDF serialization format that achieves 40-60% token reduction compared to JSON-LD, making it ideal for LLM applications and bandwidth-constrained environments.

## Why TOON-LD?

TOON-LD combines the semantic expressiveness of RDF/JSON-LD with radical token efficiency through tabular arrays. By eliminating repetitive keys and using CSV-like rows for uniform data, TOON-LD fits more information into LLM context windows while maintaining human readability.

## Features

- **Tabular Arrays**: Serialize arrays of objects as CSV-like rows with shared headers
- **40-60% Token Reduction**: Fewer tokens means lower costs and more data in context
- **Full JSON-LD Compatibility**: Round-trip conversion without data loss
- **All JSON-LD 1.1 Keywords**: Complete support for `@context`, `@graph`, `@id`, `@type`, value nodes, etc.
- **Cross-Platform**: Rust, WebAssembly (npm), and Python (PyPI) implementations

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

## Installation

### Rust
```toml
[dependencies]
toon-core = "0.1"
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
use toon_core::{jsonld_to_toon, toon_to_jsonld};

let json_ld = r#"{"@context": {"foaf": "http://xmlns.com/foaf/0.1/"}, "foaf:name": "Alice"}"#;
let toon = jsonld_to_toon(json_ld)?;
let back = toon_to_jsonld(&toon)?;
```

### Python
```python
import toon_ld

toon_str = toon_ld.convert_jsonld_to_toon('{"name": "Alice"}')
json_str = toon_ld.convert_toon_to_jsonld(toon_str)
```

### JavaScript
```javascript
import { convert_jsonld_to_toon, convert_toon_to_jsonld } from 'toon-ld';

const toon = convert_jsonld_to_toon('{"name": "Alice"}');
const json = convert_toon_to_jsonld(toon);
```

## Documentation

- **[Full Specification](SPECIFICATION.md)** - Complete grammar, algorithms, and conformance requirements
- **[W3C-Style Spec (Bikeshed)](https://kushbisen.github.io/toon-ld/)** - Rendered specification with examples
- **[Contributing Guide](CONTRIBUTING.md)** - How to contribute to the project

## Key Concepts

### Tabular Arrays
Arrays of objects share a header with field names, followed by CSV-like rows:
```
people[3]{name,age,city}:
  Alice, 30, null
  Bob, null, Portland
  Carol, 28, Seattle
```

### Value Nodes
Compact notation for language tags and datatypes:
```
title: "Bonjour"@fr
date: "2024-01-15"^^xsd:date
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
- `spec/` - Bikeshed specification source

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

## Citation

If you use TOON-LD in your research, please cite:
```bibtex
@software{toon-ld,
  title = {TOON-LD: Token-Oriented Object Notation for Linked Data},
  author = {Bisen, Kush},
  year = {2025},
  url = {https://github.com/argahsuknesib/toon-ld}
}
```
