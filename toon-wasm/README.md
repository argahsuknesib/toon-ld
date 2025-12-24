# toon-ld

[![npm](https://img.shields.io/npm/v/toon-ld)](https://www.npmjs.com/package/toon-ld)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

**Token-Oriented Object Notation for Linked Data** — A compact RDF serialization format that achieves 40-60% token reduction compared to JSON-LD, making it ideal for LLM applications and bandwidth-constrained environments.

TOON-LD extends TOON in the same way that JSON-LD extends JSON: **every valid TOON-LD document is also a valid TOON document**. Base TOON parsers can process TOON-LD without modification, while TOON-LD processors interpret `@-prefixed` keys according to JSON-LD semantics.

## Features

- **40-60% Token Reduction**: Fewer tokens means lower LLM costs and more data in context windows
- **Full JSON-LD Compatibility**: Round-trip conversion without data loss
- **Tabular Arrays**: Serialize arrays of objects as CSV-like rows with shared headers
- **All JSON-LD 1.1 Keywords**: Complete support for `@context`, `@graph`, `@id`, `@type`, value nodes, etc.
- **WebAssembly Performance**: Compiled from Rust for high-performance parsing and serialization
- **TypeScript Support**: Fully typed API with excellent IDE support
- **Browser & Node.js**: Works in both environments

## Installation

```bash
npm install toon-ld
```

Or with yarn:

```bash
yarn add toon-ld
```

## Quick Start

```javascript
import { encode, decode, parse, stringify } from 'toon-ld';

// 1. String Conversion
// Convert JSON-LD to TOON-LD
const jsonLd = JSON.stringify({
  "@context": {"foaf": "http://xmlns.com/foaf/0.1/"},
  "foaf:name": "Alice",
  "foaf:age": 30
});

const toon = encode(jsonLd);
console.log(toon);
// Output:
// @context:
//   foaf: http://xmlns.com/foaf/0.1/
// foaf:name: Alice
// foaf:age: 30

// Convert back to JSON-LD
const backToJson = decode(toon);
const parsed = JSON.parse(backToJson);
console.log(parsed);

// 2. Object Helper Functions
// Parse directly to JS Object
const data = parse(toon);
console.log(data['foaf:name']); // "Alice"

// Serialize JS Object directly to TOON-LD
const toonStr = stringify({
  "@context": {"schema": "http://schema.org/"},
  "schema:name": "Bob"
});
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

## API Reference

### `encode(json: string): string`

Convert (encode) a JSON-LD string to TOON-LD format.

**Parameters:**
- `json` - A JSON or JSON-LD formatted string

**Returns:**
- TOON-LD formatted string

**Throws:**
- Error with message if the input is invalid JSON

### `decode(toon: string): string`

Convert (decode) a TOON-LD string to JSON-LD format.

**Parameters:**
- `toon` - A TOON-LD formatted string

**Returns:**
- JSON-LD formatted string (pretty-printed)

**Throws:**
- Error with message if the input is invalid TOON-LD

### `parse(toon: string): any`

Parse a TOON-LD string directly into a JavaScript Object.

**Parameters:**
- `toon` - A TOON-LD formatted string

**Returns:**
- JavaScript Object representing the data

### `stringify(data: any): string`

Stringify a JavaScript Object directly into a TOON-LD string.

**Parameters:**
- `data` - A JavaScript Object

**Returns:**
- TOON-LD formatted string

### `validateJson(json: string): boolean`

Validate a JSON-LD string.

**Parameters:**
- `json` - A JSON or JSON-LD formatted string

**Returns:**
- `true` if the string is valid JSON, `false` otherwise

### `validateToonld(toon: string): boolean`

Validate a TOON-LD string.

**Parameters:**
- `toon` - A TOON-LD formatted string

**Returns:**
- `true` if the string is valid TOON-LD, `false` otherwise

### `init(): void`

Initialize panic hook for better error messages in the browser console. This is optional but recommended for development.

**Example:**
```javascript
import { init } from 'toon-ld';

init(); // Call once at app startup
```

## TypeScript Support

This package includes TypeScript type definitions out of the box:

```typescript
import { 
  encode, 
  decode,
  validateJson,
  validateToonld 
} from 'toon-ld';

const jsonLd: string = '{"name": "Alice"}';
const toon: string = encode(jsonLd);
const isValid: boolean = validateToonld(toon);
```

## Usage Examples

### With Express.js

```javascript
import express from 'express';
import { encode, decode } from 'toon-ld';

const app = express();
app.use(express.text({ type: 'text/toon' }));

app.post('/convert/to-toon', (req, res) => {
  try {
    const toon = encode(req.body);
    res.type('text/toon').send(toon);
  } catch (error) {
    res.status(400).json({ error: error.message });
  }
});

app.post('/convert/to-jsonld', (req, res) => {
  try {
    const jsonLd = decode(req.body);
    res.json(JSON.parse(jsonLd));
  } catch (error) {
    res.status(400).json({ error: error.message });
  }
});
```

### In the Browser

```html
<!DOCTYPE html>
<html>
<head>
  <title>TOON-LD Converter</title>
</head>
<body>
  <script type="module">
    import { encode, init } from 'https://unpkg.com/toon-ld';
    
    init(); // Better error messages
    
    const jsonLd = JSON.stringify({
      "@context": {"schema": "http://schema.org/"},
      "schema:name": "Example"
    });
    
    const toon = encode(jsonLd);
    console.log(toon);
  </script>
</body>
</html>
```

### Value Nodes with Language Tags

```javascript
const jsonLd = JSON.stringify({
  "@context": {
    "dc": "http://purl.org/dc/terms/"
  },
  "dc:title": [
    {"@value": "Bonjour", "@language": "fr"},
    {"@value": "Hello", "@language": "en"}
  ]
});

const toon = encode(jsonLd);
console.log(toon);
// Output:
// @context:
//   dc: http://purl.org/dc/terms/
// dc:title[2]{@value,@language}:
//   Bonjour,fr
//   Hello,en
```

### Error Handling

```javascript
import { decode } from 'toon-ld';

try {
  const result = decode("invalid: [unclosed");
} catch (error) {
  console.error("Conversion failed:", error.message);
  // Error message includes line numbers and helpful context
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
- **Rust**: `cargo add toon-ld` - [crates.io](https://crates.io/crates/toon-ld)
- **Python**: `pip install toon-ld` - [PyPI](https://pypi.org/project/toon-ld/)
- **CLI**: `cargo install toon-cli` - Command-line conversion tool

## Documentation

- **[Full Specification](https://github.com/argahsuknesib/toon-ld)** - Complete TOON-LD specification
- **[GitHub Repository](https://github.com/argahsuknesib/toon-ld)** - Source code and examples
- **[API Documentation](https://github.com/argahsuknesib/toon-ld#readme)** - Detailed guides

## Browser Compatibility

This package uses WebAssembly and requires:
- Chrome/Edge 57+
- Firefox 52+
- Safari 11+
- Node.js 12+

## Contributing

Contributions are welcome! Please see the [Contributing Guide](https://github.com/argahsuknesib/toon-ld/blob/main/CONTRIBUTING.md) for details.

## License

MIT License - See [LICENSE](https://github.com/argahsuknesib/toon-ld/blob/main/LICENSE) for details.

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
