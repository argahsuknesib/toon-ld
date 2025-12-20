# toon-ld

High-performance Python bindings for TOON-LD (Token-Oriented Object Notation for Linked Data).

## Overview

TOON-LD extends TOON (Token-Oriented Object Notation) to handle Linked Data (RDF). It combines the token-saving "Tabular Arrays" of TOON with the `@context` expansion of JSON-LD.

This package provides Python bindings for the Rust implementation, offering excellent performance for serialization and parsing operations.

## Installation

```bash
pip install toon-ld
```

## Usage

### Converting Between Formats

```python
import toon_ld

# Convert JSON-LD to TOON-LD
json_str = '{"name": "Alice", "age": 30}'
toon_str = toon_ld.convert_jsonld_to_toonld(json_str)
print(toon_str)
# Output:
# age: 30
# name: Alice

# Convert TOON-LD back to JSON-LD
json_back = toon_ld.convert_toonld_to_jsonld(toon_str)
print(json_back)
```

### Working with Python Objects

```python
import toon_ld

# Serialize Python dict to TOON-LD
data = {
    "users": [
        {"id": 1, "name": "Alice"},
        {"id": 2, "name": "Bob"}
    ]
}
toon_str = toon_ld.serialize_to_toon(data)
print(toon_str)
# Output:
# users[2]{id,name}:
#   1, Alice
#   2, Bob

# Parse TOON-LD to Python dict
parsed = toon_ld.parse_toon(toon_str)
print(parsed["users"][0]["name"])  # Alice
```

### Validation

```python
import toon_ld

# Validate TOON-LD
if toon_ld.validate_toon("name: Alice"):
    print("Valid TOON-LD!")

# Validate JSON
if toon_ld.validate_json('{"name": "Alice"}'):
    print("Valid JSON!")
```

## API Reference

### `convert_jsonld_to_toonld(json_str: str) -> str`

Convert a JSON-LD string to TOON-LD format.

**Args:**
- `json_str`: A JSON or JSON-LD formatted string

**Returns:** TOON-LD formatted string

**Raises:** `ValueError` if the input is not valid JSON

### `convert_toonld_to_jsonld(toon_str: str) -> str`

Convert a TOON-LD string to JSON-LD format.

**Args:**
- `toon_str`: A TOON-LD formatted string

**Returns:** JSON-LD formatted string (pretty-printed)

**Raises:** `ValueError` if the input is not valid TOON-LD

### `parse_toon(toon_str: str) -> dict`

Parse a TOON-LD string to a Python dictionary.

**Args:**
- `toon_str`: A TOON-LD formatted string

**Returns:** Python dictionary representing the parsed data

**Raises:** `ValueError` if the input is not valid TOON-LD

### `serialize_to_toon(data: Any) -> str`

Serialize a Python dictionary to TOON-LD format.

**Args:**
- `data`: A Python dictionary or JSON-serializable object

**Returns:** TOON-LD formatted string

**Raises:** `ValueError` if the input cannot be serialized

### `validate_toon(toon_str: str) -> bool`

Validate a TOON-LD string.

**Args:**
- `toon_str`: A TOON-LD formatted string

**Returns:** `True` if valid, `False` otherwise

### `validate_json(json_str: str) -> bool`

Validate a JSON string.

**Args:**
- `json_str`: A JSON formatted string

**Returns:** `True` if valid, `False` otherwise

## TOON-LD Format

### Tabular Arrays

Arrays of objects with identical keys are serialized compactly:

```
users[2]{id,name}:
  1, Alice
  2, Bob
```

### Primitive Arrays

Simple value arrays use inline format:

```
tags[3]: rust, python, wasm
```

### Quoting Rules

Strings are unquoted unless they contain special characters (`,`, `:`, `|`) or start/end with whitespace:

```
message: Hello World
note: "Hello, World"
```

## License

MIT