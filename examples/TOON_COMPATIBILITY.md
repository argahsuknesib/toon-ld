# TOON-LD Extension Relationship

## Overview

TOON-LD extends TOON in the same way that JSON-LD extends JSON. This document explains the extension relationship, compatibility guarantees, and implementation requirements.

## The JSON-LD Extension Model

JSON-LD successfully extends JSON by following these principles:

1. **Every valid JSON-LD document is also valid JSON**
2. **JSON parsers can process JSON-LD without modification**
3. **JSON-LD adds semantic meaning to certain keys** (those starting with `@`)
4. **No new JSON syntax is introduced**

Example:
```json
{
  "@context": {"foaf": "http://xmlns.com/foaf/0.1/"},
  "@id": "http://example.org/person/1",
  "foaf:name": "Alice"
}
```

A JSON parser sees this as a regular object with three string keys. A JSON-LD processor additionally interprets `@context` as namespace definitions and `@id` as a node identifier.

## TOON-LD Extension Model (Identical Pattern)

TOON-LD follows the exact same pattern:

1. **Every valid TOON-LD document is also valid TOON**
2. **TOON parsers can process TOON-LD without modification**
3. **TOON-LD adds semantic meaning to certain keys** (those starting with `@`)
4. **No new TOON syntax is introduced**

Example:
```
@context:
  foaf: http://xmlns.com/foaf/0.1/
@id: http://example.org/person/1
foaf:name: Alice
```

A TOON parser sees this as a regular object with three keys. A TOON-LD processor additionally interprets the `@-prefixed` keys according to JSON-LD semantics.

## Compatibility Guarantees

### Syntactic Compatibility

Every TOON-LD document MUST be parseable by a conformant TOON parser. This means:

- ✅ Use only TOON-defined syntax (objects, arrays, tabular format, primitives)
- ✅ Follow TOON's indentation, quoting, and escaping rules
- ✅ Use TOON's array syntax (`key[N]:` or `key[N]{fields}:`)
- ❌ NO special operators, suffixes, or shorthand notations

### Semantic Layer

TOON-LD processors add semantic interpretation on top of TOON's syntax:

- `@context` → namespace and vocabulary definitions
- `@id` → node identifier (URI reference)
- `@type` → node type(s)
- `@graph` → named graph container
- `@value`, `@language`, `@type` → value nodes with language tags or datatypes

A TOON parser treats these as regular keys. A TOON-LD processor interprets them according to JSON-LD 1.1 semantics.

## Value Nodes: The Critical Design Decision

### The Problem

JSON-LD represents language-tagged strings and typed literals using objects with special keys:

```json
{
  "title": {
    "@value": "Bonjour",
    "@language": "fr"
  },
  "date": {
    "@value": "2024-01-15",
    "@type": "xsd:date"
  }
}
```

Some RDF formats (like Turtle/N-Triples) use special syntax:
```turtle
"Bonjour"@fr
"2024-01-15"^^xsd:date
```

### The Temptation (and Why It's Wrong)

It might seem convenient to adopt Turtle-like syntax in TOON-LD:

```
# DON'T DO THIS - breaks TOON compatibility!
title: "Bonjour"@fr
date: "2024-01-15"^^xsd:date
```

**Problem**: This syntax is NOT valid TOON. A TOON parser would fail or misparse these values. This breaks the fundamental extension relationship.

### The Correct Solution

Use standard TOON syntax for value nodes:

**Single value (object notation):**
```
title:
  @value: Bonjour
  @language: fr
```

**Multiple values (tabular notation for token efficiency):**
```
titles[2]{@value,@language}:
  The Hobbit,en
  Der Hobbit,de
```

Both representations are valid TOON and can be parsed by any TOON parser. TOON-LD processors recognize the `@value`, `@language`, and `@type` keys and interpret them as JSON-LD value nodes.

## Token Efficiency Preserved

The tabular format maintains TOON-LD's token efficiency goals:

**JSON-LD (verbose):**
```json
{
  "titles": [
    {"@value": "The Hobbit", "@language": "en"},
    {"@value": "Der Hobbit", "@language": "de"},
    {"@value": "Le Hobbit", "@language": "fr"}
  ]
}
```
~180 characters, ~50 tokens

**TOON-LD (compact, TOON-compatible):**
```
titles[3]{@value,@language}:
  The Hobbit,en
  Der Hobbit,de
  Le Hobbit,fr
```
~70 characters, ~20 tokens (60% reduction)

The tabular format eliminates repeated keys (`@value`, `@language`) while remaining syntactically compatible with base TOON.

## Implementation Requirements

### For TOON-LD Serializers (Encoders)

1. **MUST** produce output that is valid TOON
2. **MUST NOT** use any syntax not defined in the TOON specification
3. **MUST** represent value nodes using standard TOON objects or tabular arrays
4. **SHOULD** use tabular format for arrays of value nodes (token efficiency)
5. **MUST** follow TOON's keyword ordering when available

### For TOON-LD Parsers (Decoders)

1. **MUST** accept all valid TOON input
2. **SHOULD** recognize and interpret `@-prefixed` keys according to JSON-LD semantics
3. **MUST** parse value nodes (objects with `@value` key) according to JSON-LD rules
4. **MAY** perform URI expansion/compaction using `@context`
5. **MUST** preserve semantic equivalence during round-trips

### Testing Compatibility

Implementers MUST verify compatibility by:

1. **Roundtrip test**: `JSON-LD → TOON-LD → JSON-LD` preserves semantics
2. **TOON compatibility test**: Feed TOON-LD output to a reference TOON parser
3. **Structure validation**: Verify TOON parser produces expected object/array structure

Example test:
```rust
// 1. Convert JSON-LD to TOON-LD
let json_ld = r#"{"@id": "ex:1", "name": "Alice"}"#;
let toon_ld = jsonld_to_toonld(json_ld);

// 2. Parse with base TOON parser (must succeed)
let toon_structure = toon_parse(toon_ld);
assert!(toon_structure.is_ok());

// 3. Verify structure
assert_eq!(toon_structure["@id"], "ex:1");
assert_eq!(toon_structure["name"], "Alice");

// 4. Convert back to JSON-LD (must preserve semantics)
let json_ld_2 = toonld_to_jsonld(toon_ld);
assert_semantically_equivalent(json_ld, json_ld_2);
```

## Benefits of This Approach

1. **True Extension**: TOON-LD is a proper superset of TOON's syntax
2. **Incremental Adoption**: Existing TOON tools can process TOON-LD data
3. **Clear Separation**: Syntax (TOON) vs. Semantics (JSON-LD)
4. **Future-Proof**: No conflicts with future TOON syntax additions
5. **Token Efficiency**: Tabular format achieves 40-60% token reduction vs JSON-LD

## Examples

### Basic Graph

**JSON-LD:**
```json
{
  "@context": {"foaf": "http://xmlns.com/foaf/0.1/"},
  "@graph": [
    {"@id": "ex:1", "@type": "foaf:Person", "foaf:name": "Alice"},
    {"@id": "ex:2", "@type": "foaf:Person", "foaf:name": "Bob"}
  ]
}
```

**TOON-LD (TOON-compatible):**
```
@context:
  foaf: http://xmlns.com/foaf/0.1/
@graph[2]{@id,@type,foaf:name}:
  ex:1,foaf:Person,Alice
  ex:2,foaf:Person,Bob
```

### Value Nodes with Language Tags

**JSON-LD:**
```json
{
  "@context": {"dc": "http://purl.org/dc/terms/"},
  "dc:title": [
    {"@value": "The Hobbit", "@language": "en"},
    {"@value": "Der Hobbit", "@language": "de"}
  ]
}
```

**TOON-LD (TOON-compatible):**
```
@context:
  dc: http://purl.org/dc/terms/
dc:title[2]{@value,@language}:
  The Hobbit,en
  Der Hobbit,de
```

### Typed Literals

**JSON-LD:**
```json
{
  "@context": {"xsd": "http://www.w3.org/2001/XMLSchema#"},
  "birthDate": {
    "@value": "1990-05-15",
    "@type": "xsd:date"
  }
}
```

**TOON-LD (TOON-compatible):**
```
@context:
  xsd: http://www.w3.org/2001/XMLSchema#
birthDate:
  @value: "1990-05-15"
  @type: xsd:date
```

## Summary

TOON-LD successfully extends TOON by:

1. **Using only TOON syntax** - no new operators, suffixes, or special notations
2. **Adding semantic interpretation** - `@-prefixed` keys have JSON-LD meaning
3. **Leveraging tabular format** - achieving token efficiency while maintaining compatibility
4. **Following the JSON-LD model** - every TOON-LD document is valid TOON

This approach ensures that TOON-LD can be processed by existing TOON tools while providing full JSON-LD semantic expressiveness for Linked Data applications.