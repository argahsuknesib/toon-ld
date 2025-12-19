# TOON-LD Specification

**Version:** 3.0  
**Status:** Draft  
**Last Updated:** 2025-01-15

## Abstract

TOON-LD (Token-Oriented Object Notation for Linked Data) is a text serialization format designed to minimize token count while preserving the semantic richness of JSON-LD. It achieves 40-60% token reduction compared to equivalent JSON-LD representations, making it ideal for Large Language Model (LLM) context windows and bandwidth-constrained applications.

## Table of Contents

1. [Introduction](#1-introduction)
2. [Design Goals](#2-design-goals)
3. [Lexical Structure](#3-lexical-structure)
4. [Grammar](#4-grammar)
5. [Data Types](#5-data-types)
6. [Objects](#6-objects)
7. [Arrays](#7-arrays)
8. [Tabular Arrays](#8-tabular-arrays)
9. [JSON-LD Keywords](#9-json-ld-keywords)
10. [Context and URI Handling](#10-context-and-uri-handling)
11. [Quoting and Escaping](#11-quoting-and-escaping)
12. [Whitespace and Indentation](#12-whitespace-and-indentation)
13. [Parsing Algorithm](#13-parsing-algorithm)
14. [Serialization Algorithm](#14-serialization-algorithm)
15. [Conformance](#15-conformance)
16. [Examples](#16-examples)
17. [Security Considerations](#17-security-considerations)
18. [IANA Considerations](#18-iana-considerations)

---

## 1. Introduction

TOON-LD extends the TOON (Token-Oriented Object Notation) format to support Linked Data semantics. It provides a human-readable, token-efficient alternative to JSON-LD that maintains full round-trip compatibility.

### 1.1 Relationship to JSON-LD

TOON-LD is designed as a serialization format for JSON-LD data. Any valid JSON-LD document can be converted to TOON-LD and back without loss of information. TOON-LD preserves:

- All JSON-LD keywords (`@context`, `@id`, `@type`, `@graph`, etc.)
- Prefix definitions and URI compaction
- Nested structures and arrays
- Value nodes with language tags and datatype annotations

### 1.2 Notation

The key words "MUST", "MUST NOT", "REQUIRED", "SHALL", "SHALL NOT", "SHOULD", "SHOULD NOT", "RECOMMENDED", "MAY", and "OPTIONAL" in this document are to be interpreted as described in RFC 2119.

---

## 2. Design Goals

1. **Token Efficiency**: Minimize the number of tokens when processed by LLM tokenizers
2. **Human Readability**: Maintain readability without sacrificing compactness
3. **Round-Trip Fidelity**: Guarantee lossless conversion to/from JSON-LD
4. **Streaming Support**: Enable incremental parsing for large documents
5. **Simplicity**: Keep the grammar simple enough for straightforward implementation

---

## 3. Lexical Structure

### 3.1 Character Set

TOON-LD documents MUST be encoded in UTF-8.

### 3.2 Line Termination

Lines are terminated by:
- Line Feed (U+000A) - LF
- Carriage Return followed by Line Feed (U+000D U+000A) - CRLF

Implementations MUST accept both forms and SHOULD produce LF when serializing.

### 3.3 Comments

TOON-LD does not support comments. All content is significant.

### 3.4 Reserved Characters

The following characters have special meaning:

| Character | Name | Usage |
|-----------|------|-------|
| `:` | Colon | Key-value separator |
| `,` | Comma | Value separator in arrays/rows |
| `[` `]` | Brackets | Array length notation |
| `{` `}` | Braces | Tabular field list |
| `@` | At sign | JSON-LD keyword prefix |
| `"` | Double quote | String delimiter |
| `^` | Caret | Datatype annotation (`^^`) |
| `\|` | Pipe | Reserved for future use |

---

## 4. Grammar

### 4.1 EBNF Grammar

```ebnf
document        = object | array ;

object          = { property } ;

property        = key ":" value NEWLINE
                | key ":" NEWLINE indented-value
                | array-property ;

key             = jsonld-keyword | prefixed-name | string ;

jsonld-keyword  = "@" identifier ;

prefixed-name   = identifier [ ":" identifier ] ;

identifier      = ( ALPHA | "_" ) { ALPHA | DIGIT | "_" } ;

value           = primitive | inline-array | value-node ;

primitive       = null | boolean | number | string ;

null            = "null" ;

boolean         = "true" | "false" ;

number          = [ "-" ] digits [ "." digits ] [ exponent ] ;

digits          = DIGIT { DIGIT } ;

exponent        = ( "e" | "E" ) [ "+" | "-" ] digits ;

string          = unquoted-string | quoted-string ;

unquoted-string = safe-char { safe-char } ;

quoted-string   = '"' { string-char } '"' ;

safe-char       = ? any character except ':', ',', '|', '"', leading/trailing whitespace ? ;

string-char     = ? any character except unescaped '"' ? | escape-sequence ;

escape-sequence = "\" ( '"' | "\" | "/" | "b" | "f" | "n" | "r" | "t" | unicode-escape ) ;

unicode-escape  = "u" HEX HEX HEX HEX ;

inline-array    = "[" length "]:" [ value-list ] ;

value-list      = value { "," value } ;

array-property  = tabular-array | primitive-array ;

tabular-array   = key "[" length "]{" field-list "}:" NEWLINE { row } ;

primitive-array = key "[" length "]:" [ value-list ] NEWLINE
                | key "[" length "]:" NEWLINE { indented-value } ;

field-list      = field { "," field } ;

field           = key ;

row             = INDENT value-list NEWLINE ;

length          = digits ;

indented-value  = INDENT ( object | value NEWLINE ) ;

value-node      = quoted-string "@" language-tag
                | quoted-string "^^" datatype ;

language-tag    = identifier [ "-" identifier ] ;

datatype        = prefixed-name | iri ;

iri             = "<" { iri-char } ">" ;

INDENT          = "  " ;  (* exactly 2 spaces per nesting level *)

NEWLINE         = LF | CRLF ;

ALPHA           = ? [A-Za-z] ? ;

DIGIT           = ? [0-9] ? ;

HEX             = ? [0-9A-Fa-f] ? ;
```

---

## 5. Data Types

### 5.1 Null

The literal `null` represents the absence of a value.

```
key: null
```

### 5.2 Boolean

Boolean values are represented as `true` or `false` (case-sensitive).

```
enabled: true
disabled: false
```

### 5.3 Numbers

Numbers follow JSON number syntax:

- Integer: `42`, `-17`
- Floating point: `3.14`, `-0.5`
- Exponential: `1.23e10`, `5E-3`

```
count: 42
temperature: -17.5
avogadro: 6.022e23
```

### 5.4 Strings

Strings can be unquoted or quoted.

**Unquoted strings** are used when the value:
- Does not contain `:`, `,`, `|`, or `"`
- Does not start or end with whitespace
- Is not a reserved word (`null`, `true`, `false`)
- Is not parseable as a number

```
name: Alice
city: New York
```

**Quoted strings** MUST be used when:
- The value contains reserved characters (`:`, `,`, `|`)
- The value starts or ends with whitespace
- The value is empty
- The value could be confused with a reserved word or number

```
message: "Hello, World!"
path: "C:\Users\name"
empty: ""
literal_null: "null"
```

---

## 6. Objects

Objects are represented as a sequence of key-value pairs, one per line.

### 6.1 Simple Objects

```
name: Alice
age: 30
email: alice@example.org
```

Equivalent JSON:
```json
{
  "name": "Alice",
  "age": 30,
  "email": "alice@example.org"
}
```

### 6.2 Nested Objects

Nested objects are indented by 2 spaces per level:

```
person:
  name: Alice
  address:
    street: 123 Main St
    city: Springfield
```

Equivalent JSON:
```json
{
  "person": {
    "name": "Alice",
    "address": {
      "street": "123 Main St",
      "city": "Springfield"
    }
  }
}
```

---

## 7. Arrays

### 7.1 Primitive Arrays

Arrays of primitive values use the `key[N]:` syntax where `N` is the array length.

**Inline format** (for short arrays):
```
tags[3]: red, green, blue
```

**Multi-line format** (for long arrays or readability):
```
numbers[5]:
  1
  2
  3
  4
  5
```

### 7.2 Empty Arrays

```
items[0]:
```

Or equivalently:
```
items: []
```

### 7.3 Mixed Arrays

Arrays containing mixed types or nested structures use list notation:

```
mixed[3]:
  - 42
  - hello
  -
    nested: object
```

---

## 8. Tabular Arrays

Tabular arrays are TOON-LD's primary mechanism for token reduction. When an array contains objects, they are serialized as a table with a header row and CSV-like data rows.

### 8.1 Syntax

```
key[N]{field1,field2,field3}:
  val1, val2, val3
  val4, val5, val6
```

Where:
- `key` is the property name
- `N` is the number of rows (array length)
- `{field1,field2,field3}` lists the column headers
- Each subsequent indented line is a data row

### 8.2 Field Union Behavior

When objects in an array have different keys, TOON-LD uses the **union of all keys** as the header. Missing values are represented as `null`.

Input JSON:
```json
{
  "people": [
    {"name": "Alice", "age": 30},
    {"name": "Bob", "city": "NYC"},
    {"name": "Carol", "age": 25, "city": "LA"}
  ]
}
```

TOON-LD output:
```
people[3]{name,age,city}:
  Alice, 30, null
  Bob, null, NYC
  Carol, 25, LA
```

### 8.3 Field Ordering

Fields in the header MUST be ordered as follows:

1. JSON-LD keywords first, in this order: `@id`, `@type`, then others alphabetically
2. Non-keyword fields in alphabetical order

### 8.4 Nested Values in Tabular Rows

When a cell contains an object or array, it MUST be serialized as JSON and quoted:

```
items[2]{name,metadata}:
  Widget, "{""color"":""red""}"
  Gadget, "{""color"":""blue"",""size"":""large""}"
```

### 8.5 Quoting in Tabular Rows

Values in tabular rows MUST be quoted if they:
- Contain commas
- Contain double quotes (escape as `""`)
- Start or end with whitespace

```
messages[2]{id,text}:
  1, "Hello, World!"
  2, "She said ""Hi"""
```

---

## 9. JSON-LD Keywords

TOON-LD supports all JSON-LD 1.1 keywords. Keywords retain their `@` prefix.

### 9.1 Core Keywords

| Keyword | Description | TOON-LD Handling |
|---------|-------------|------------------|
| `@context` | Namespace definitions | Serialized first, nested key-value pairs |
| `@id` | Node identifier | Simple value or URI |
| `@type` | Node type(s) | Simple value or array |
| `@graph` | Named graph | Tabular array format |
| `@value` | Explicit value | Compact notation (see 9.3) |
| `@language` | Language tag | Combined with `@value` |
| `@list` | Ordered collection | Array with `@list` prefix |
| `@set` | Unordered collection | Array with `@set` prefix |
| `@reverse` | Reverse properties | Nested object |
| `@base` | Base IRI | Simple value |
| `@vocab` | Default vocabulary | Simple value |

### 9.2 Additional Keywords (JSON-LD 1.1)

| Keyword | Description | TOON-LD Handling |
|---------|-------------|------------------|
| `@container` | Container type | Simple value or array |
| `@direction` | Text direction | Simple value (`ltr` or `rtl`) |
| `@import` | Import external context | URI value |
| `@included` | Included nodes | Array of nodes |
| `@index` | Index property | Simple value |
| `@json` | JSON literal | Quoted JSON string |
| `@nest` | Nested properties | Simple value |
| `@none` | Default index value | Literal `@none` |
| `@prefix` | Prefix flag | Boolean |
| `@propagate` | Context propagation | Boolean |
| `@protected` | Protected term | Boolean |
| `@version` | JSON-LD version | Number (1.1) |

### 9.3 Value Nodes

Value nodes (`@value` with optional `@language` or `@type`) use compact notation:

**Language-tagged strings:**
```
title:
  "Bonjour"@fr
```

Equivalent JSON-LD:
```json
{
  "title": {
    "@value": "Bonjour",
    "@language": "fr"
  }
}
```

**Typed literals:**
```
birthDate:
  "1990-05-15"^^xsd:date
```

Equivalent JSON-LD:
```json
{
  "birthDate": {
    "@value": "1990-05-15",
    "@type": "xsd:date"
  }
}
```

### 9.4 Keyword Ordering

When serializing, keywords MUST appear in this order:
1. `@context`
2. `@base`
3. `@vocab`
4. `@id`
5. `@type`
6. `@graph`
7. `@reverse`
8. All other keywords alphabetically
9. Non-keyword properties alphabetically

---

## 10. Context and URI Handling

### 10.1 Context Serialization

The `@context` object is serialized with nested key-value pairs:

```
@context:
  foaf: http://xmlns.com/foaf/0.1/
  schema: http://schema.org/
  @base: http://example.org/
  @vocab: http://example.org/vocab/
```

### 10.2 URI Compaction

When a `@context` is present, full URIs SHOULD be compacted using defined prefixes:

Input:
```json
{
  "@context": {"foaf": "http://xmlns.com/foaf/0.1/"},
  "http://xmlns.com/foaf/0.1/name": "Alice"
}
```

Output:
```
@context:
  foaf: http://xmlns.com/foaf/0.1/
foaf:name: Alice
```

### 10.3 URI Expansion

When parsing, prefixed URIs SHOULD be expanded if a context is available.

---

## 11. Quoting and Escaping

### 11.1 When to Quote

A string value MUST be quoted if it:

1. Contains any of: `:` `,` `|` `"`
2. Starts or ends with whitespace
3. Is empty
4. Equals `null`, `true`, or `false`
5. Could be parsed as a number

### 11.2 Escape Sequences

Within quoted strings, the following escape sequences are recognized:

| Sequence | Character |
|----------|-----------|
| `\"` | Double quote (U+0022) |
| `\\` | Backslash (U+005C) |
| `\/` | Forward slash (U+002F) |
| `\b` | Backspace (U+0008) |
| `\f` | Form feed (U+000C) |
| `\n` | Newline (U+000A) |
| `\r` | Carriage return (U+000D) |
| `\t` | Tab (U+0009) |
| `\uXXXX` | Unicode code point |

### 11.3 CSV Cell Escaping

In tabular array rows:
- Values containing commas MUST be quoted
- Embedded double quotes MUST be escaped by doubling (`""`)

```
data[2]{name,note}:
  Alice, "Said ""Hello"""
  Bob, "Uses comma, here"
```

---

## 12. Whitespace and Indentation

### 12.1 Indentation

- Indentation MUST use exactly **2 spaces** per nesting level
- Tabs MUST NOT be used for indentation
- Mixed tabs and spaces MUST NOT be used

### 12.2 Trailing Whitespace

- Lines SHOULD NOT have trailing whitespace
- Parsers MUST tolerate trailing whitespace

### 12.3 Blank Lines

- Blank lines between properties are allowed but not required
- Parsers MUST skip blank lines
- Serializers SHOULD NOT emit blank lines

---

## 13. Parsing Algorithm

### 13.1 Parser States

The parser operates in one of two modes:

1. **Indented Mode**: Default mode for parsing key-value pairs and nested objects
2. **CSV Mode**: Activated when a tabular array header is encountered

### 13.2 Indented Mode Parsing

```
1. Read a line
2. Calculate indentation level (number of leading spaces / 2)
3. If line matches tabular header pattern `key[N]{fields}:`:
   a. Switch to CSV mode
   b. Parse N rows as CSV data
   c. Return to indented mode
4. If line matches primitive array pattern `key[N]: values`:
   a. Parse inline values as array
5. If line matches key-value pattern `key: value`:
   a. If value is empty, parse nested content at deeper indent
   b. Otherwise, parse value as primitive
6. Handle indentation changes for object nesting
```

### 13.3 CSV Mode Parsing

```
1. Read the header to extract field names
2. For each of N rows:
   a. Read line and trim indentation
   b. Parse comma-separated values
   c. Handle quoted values (CSV escaping rules)
   d. Map values to field names
   e. Create object from field-value pairs
3. Return array of objects
4. Return to indented mode
```

### 13.4 Value Parsing

```
1. If value is "null": return null
2. If value is "true": return true
3. If value is "false": return false
4. If value matches number pattern: return number
5. If value starts with '"': parse quoted string
6. If value contains '@' (language tag): parse value node
7. If value contains '^^' (datatype): parse typed literal
8. Otherwise: return unquoted string
```

---

## 14. Serialization Algorithm

### 14.1 Top-Level Serialization

```
1. If value is object:
   a. Extract @context if present, build prefix map
   b. Serialize @context first
   c. Serialize remaining properties in keyword order
2. If value is array:
   a. If array of uniform objects: use tabular format
   b. If array of primitives: use inline/multiline format
   c. Otherwise: use mixed array format
3. If value is primitive: serialize directly
```

### 14.2 Object Serialization

```
1. Sort keys by keyword order (see 9.4)
2. For each key-value pair:
   a. Compact key using context prefixes
   b. If value is object: serialize nested with increased indent
   c. If value is array: choose appropriate array format
   d. If value is primitive: serialize inline
```

### 14.3 Tabular Array Serialization

```
1. Collect union of all keys from all objects
2. Sort fields (JSON-LD keywords first, then alphabetically)
3. Emit header: key[N]{field1,field2,...}:
4. For each object:
   a. For each field in header order:
      - Get value or null if missing
      - Convert to CSV cell (quote if needed)
   b. Emit row with proper indentation
```

---

## 15. Conformance

### 15.1 Conformance Levels

**Level 1 - Basic:**
- Parse and serialize primitive values
- Parse and serialize simple objects
- Parse and serialize primitive arrays

**Level 2 - Standard:**
- All Level 1 features
- Parse and serialize tabular arrays
- Handle union-of-keys for non-uniform objects
- Support all JSON-LD core keywords

**Level 3 - Full:**
- All Level 2 features
- Support all JSON-LD 1.1 keywords
- URI compaction/expansion with @context
- Value node compact notation

### 15.2 Round-Trip Requirements

A conforming implementation MUST satisfy:

```
parse(serialize(json_ld)) ≡ json_ld
serialize(parse(toon_ld)) ≡ toon_ld (modulo whitespace normalization)
```

Where `≡` indicates semantic equivalence (identical data, possibly different formatting).

---

## 16. Examples

### 16.1 Simple Document

**JSON-LD:**
```json
{
  "@context": {
    "foaf": "http://xmlns.com/foaf/0.1/"
  },
  "@id": "http://example.org/alice",
  "@type": "foaf:Person",
  "foaf:name": "Alice",
  "foaf:age": 30
}
```

**TOON-LD:**
```
@context:
  foaf: http://xmlns.com/foaf/0.1/
@id: "http://example.org/alice"
@type: foaf:Person
foaf:age: 30
foaf:name: Alice
```

### 16.2 Graph with Tabular Data

**JSON-LD:**
```json
{
  "@context": {
    "foaf": "http://xmlns.com/foaf/0.1/",
    "schema": "http://schema.org/"
  },
  "@graph": [
    {"@id": "ex:1", "@type": "foaf:Person", "foaf:name": "Alice", "schema:age": 30},
    {"@id": "ex:2", "@type": "foaf:Person", "foaf:name": "Bob", "schema:age": 25},
    {"@id": "ex:3", "@type": "foaf:Person", "foaf:name": "Carol", "schema:age": 35}
  ]
}
```

**TOON-LD:**
```
@context:
  foaf: http://xmlns.com/foaf/0.1/
  schema: http://schema.org/
@graph[3]{@id,@type,foaf:name,schema:age}:
  ex:1, foaf:Person, Alice, 30
  ex:2, foaf:Person, Bob, 25
  ex:3, foaf:Person, Carol, 35
```

### 16.3 Value Nodes

**JSON-LD:**
```json
{
  "@context": {
    "xsd": "http://www.w3.org/2001/XMLSchema#",
    "schema": "http://schema.org/"
  },
  "schema:name": [
    {"@value": "The Hobbit", "@language": "en"},
    {"@value": "Der Hobbit", "@language": "de"}
  ],
  "schema:datePublished": {
    "@value": "1937-09-21",
    "@type": "xsd:date"
  }
}
```

**TOON-LD:**
```
@context:
  xsd: http://www.w3.org/2001/XMLSchema#
  schema: http://schema.org/
schema:name[2]:
  "The Hobbit"@en
  "Der Hobbit"@de
schema:datePublished:
  "1937-09-21"^^xsd:date
```

### 16.4 Reverse Properties

**JSON-LD:**
```json
{
  "@context": {"schema": "http://schema.org/"},
  "@id": "http://example.org/book/1",
  "@type": "schema:Book",
  "schema:name": "The Great Novel",
  "@reverse": {
    "schema:about": [
      {"@id": "http://example.org/review/1"},
      {"@id": "http://example.org/review/2"}
    ]
  }
}
```

**TOON-LD:**
```
@context:
  schema: http://schema.org/
@id: "http://example.org/book/1"
@type: schema:Book
schema:name: The Great Novel
@reverse:
  schema:about[2]: "http://example.org/review/1", "http://example.org/review/2"
```

---

## 17. Security Considerations

### 17.1 Input Validation

Implementations MUST:
- Validate UTF-8 encoding
- Reject malformed escape sequences
- Limit recursion depth to prevent stack overflow
- Limit document size to prevent memory exhaustion

### 17.2 URI Handling

When expanding or compacting URIs:
- Validate URI syntax
- Be cautious of URI schemes that may trigger actions (e.g., `javascript:`)
- Consider restricting allowed URI schemes

### 17.3 Resource Limits

Implementations SHOULD allow configuration of:
- Maximum document size
- Maximum nesting depth
- Maximum number of fields in tabular arrays
- Maximum line length

---

## 18. IANA Considerations

### 18.1 Media Type

The proposed media type for TOON-LD is:

```
Type name: application
Subtype name: toon-ld
Required parameters: none
Optional parameters: charset (default: utf-8)
Encoding considerations: UTF-8
Security considerations: See Section 17
```

### 18.2 File Extension

The recommended file extension is `.toon` or `.toonld`.

---

## Appendix A: Comparison with JSON-LD

| Aspect | JSON-LD | TOON-LD |
|--------|---------|---------|
| Delimiters | `{}`, `[]`, `:`, `,` | `:`, `,` (minimal) |
| Key quoting | Always quoted | Never quoted |
| String quoting | Always quoted | Only when necessary |
| Repetition | Keys repeated per object | Keys in header only |
| Whitespace | Insignificant | Significant (indentation) |
| Typical token reduction | Baseline | 40-60% fewer tokens |

## Appendix B: Token Savings Analysis

For an array of N objects with K keys and average value length V:

**JSON-LD tokens (approximate):**
```
2 + N × (2 + K × (2 + 1 + 1)) = 2 + N × (2 + 4K) = 2 + 2N + 4NK
```

**TOON-LD tokens (approximate):**
```
1 + K + N × K = 1 + K + NK
```

**Savings ratio:**
```
(2 + 2N + 4NK - 1 - K - NK) / (2 + 2N + 4NK)
= (1 + 2N + 3NK - K) / (2 + 2N + 4NK)
≈ 3NK / 4NK = 75% for large N and K
```

In practice, savings of 40-60% are typical due to value token overhead.

---

## Appendix C: Reference Implementation

The reference implementation is available at:

- **Repository:** https://github.com/[org]/toon-ld
- **Rust crate:** `toon-core`
- **npm package:** `toon-ld`
- **PyPI package:** `toon-ld`

---

## Appendix D: Changelog

### Version 3.0 (Current)
- Added union-of-keys behavior for tabular arrays
- Added support for all JSON-LD 1.1 keywords
- Added compact value node notation (`@lang`, `^^type`)
- Formalized quoting and escaping rules
- Added streaming parsing considerations

### Version 2.0
- Added JSON-LD keyword support
- Added URI compaction/expansion
- Added tabular array format

### Version 1.0
- Initial specification
- Basic TOON format with indentation-based nesting

---

*End of Specification*