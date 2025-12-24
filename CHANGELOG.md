# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- Additional JSON-LD 1.1 keyword support (planned)
- Streaming parser support (planned)

## [0.2.2] - 2025-01-24

### BREAKING CHANGES

Global API Rename: All supported languages (Rust, Python, JavaScript/WASM) now use a consistent, concise API: `encode`, `decode`, `parse`, and `stringify`.

#### JavaScript/WASM (toon-ld)
- `convert_jsonld_to_toonld` → `encode`
- `convert_toonld_to_jsonld` → `decode`
- `parse_toonld` → `parse`
- `serialize_to_toonld` → `stringify`
- `validate_toonld` → `validateToonld`
- `validate_json` → `validateJson`

#### Python (toon-ld)
- `convert_jsonld_to_toonld` → `encode`
- `convert_toonld_to_jsonld` → `decode`
- `parse_toonld` → `parse`
- `serialize_to_toonld` → `stringify`

#### Rust (toon-core / toon-ld)
- `jsonld_to_toonld` → `encode`
- `toonld_to_jsonld` → `decode`

### Rationale
This change standardizes the API across all platforms, using the standard compression verbs (`encode`/`decode`) for the string-to-string format conversion, and standard Object verbs (`parse`/`stringify`) for Object-to-String conversion. This aligns with standard idioms in Python (e.g. `codecs`), Rust (e.g. `encoding`), and JavaScript (e.g. `JSON`).

## [0.2.1] - 2025-01-22

### Changed
- Removed all emojis from codebase for cleaner professional appearance
- Updated benchmarks and added new visualizations in README
- Improved documentation structure (moved detailed docs to `docs/` folder)

## [0.2.0] - 2025-01-19

### BREAKING CHANGES

All function names have been updated to use `toonld` instead of `toon` to accurately reflect that we're converting to/from **TOON-LD** format, not generic TOON.

#### Changed Function Names

**Rust (toon-core, toon-ld):**
- `jsonld_to_toon()` → `jsonld_to_toonld()`
- `toon_to_jsonld()` → `toonld_to_jsonld()`

**Python (toon-py):**
- `convert_jsonld_to_toon()` → `convert_jsonld_to_toonld()`
- `convert_toon_to_jsonld()` → `convert_toonld_to_jsonld()`
- `validate_toon()` → `validate_toonld()`
- `parse_toon()` → `parse_toonld()`
- `serialize_to_toon()` → `serialize_to_toonld()`

**JavaScript/TypeScript (toon-wasm):**
- `convert_jsonld_to_toon()` → `convert_jsonld_to_toonld()`
- `convert_toon_to_jsonld()` → `convert_toonld_to_jsonld()`
- `validate_toon()` → `validate_toonld()`

#### Package Changes

**Rust:**
- `toon-core` crate is now **deprecated** - use `toon-ld` instead
- `toon-ld` v0.2.0 is the new recommended package
- Both packages updated to use consistent naming

**Python:**
- Updated to v0.2.0 with new function names
- Package name remains `toon-ld` on PyPI

**npm:**
- Updated to v0.2.0 with new function names
- Package name remains `toon-ld` on npm

#### Documentation Updates

- Corrected misleading performance claims (removed "zero-copy parsing" - not currently implemented)
- Updated all README files across packages
- Added deprecation notice to `toon-core`
- Updated code examples in all documentation

#### Added
- **docs/MIGRATION.md** - Comprehensive migration guide from v0.1.x to v0.2.0
- Migration examples for Rust, Python, and JavaScript/TypeScript
- Automated migration scripts for all platforms

#### Why This Change?

The previous naming was misleading:
- **TOON** is the base format (like JSON)
- **TOON-LD** extends TOON with Linked Data semantics (like JSON-LD extends JSON)

The old function names (`jsonld_to_toon`) suggested we were converting to generic TOON, but we're actually converting to TOON-LD. The new names (`jsonld_to_toonld`) accurately reflect this distinction.

## [0.1.0] - 2025-01-15

### Added

#### Core Library (toon-core)
- Initial implementation of TOON-LD serializer and parser
- Tabular array support with union-of-keys behavior for non-uniform objects
- Primitive array serialization (inline and multi-line formats)
- JSON-LD context parsing and URI compaction/expansion
- Support for core JSON-LD keywords:
  - `@context` - Namespace prefix definitions
  - `@base` - Base IRI for relative references
  - `@vocab` - Default vocabulary IRI
  - `@id` - Node identifier
  - `@type` - Node type (single value and arrays)
  - `@graph` - Named graph with tabular serialization
  - `@value` - Explicit value nodes
  - `@language` - Language tags for value nodes
  - `@list` - Ordered collections
  - `@set` - Unordered collections
  - `@reverse` - Reverse properties

- Smart quoting (only quote strings containing special characters)
- Comprehensive test suite (20+ unit tests)

#### CLI Tool (toon-cli)
- `convert` command for JSON-LD to TOON-LD conversion (bidirectional)
- `validate` command for syntax validation
- `stats` command for size and token statistics
- `benchmark` command for performance testing at various scales
- Support for stdin/stdout and file I/O
- Auto-detection of input format

#### WebAssembly Bindings (toon-wasm)
- `convert_jsonld_to_toon()` function
- `convert_toon_to_jsonld()` function
- Compatible with web and Node.js targets

#### Python Bindings (toon-py)
- `convert_jsonld_to_toon()` function
- `convert_toon_to_jsonld()` function
- `serialize_to_toon()` for Python dict input
- `parse_toon()` for parsing to Python dict
- PyO3-based bindings with maturin build system

#### Documentation
- Comprehensive README with examples
- SPECIFICATION.md formal format specification (v3.0)
- CONTRIBUTING.md guidelines
- MIT License

#### CI/CD
- GitHub Actions workflow for testing (multi-OS, multi-Rust-version)
- Linting workflow (rustfmt, clippy)
- WASM build verification
- Python binding build verification
- Code coverage reporting
- Security audit
- Release workflow for crates.io, npm, and PyPI publishing

### Performance
- Benchmark results show 40-60% token reduction vs JSON-LD
- Size savings of 37-44% across dataset scales (10 to 100k records)
- Token savings of 52-59% (significant for LLM context windows)

## [0.0.1] - 2025-01-01

### Added
- Initial project scaffolding
- Basic workspace structure

---

## Comparison

### Token Savings by Dataset Size

| Records | JSON-LD Size | TOON-LD Size | Size Saved | Tokens Saved |
|---------|--------------|--------------|------------|--------------|
| 10      | 2,249 B      | 1,425 B      | 36.6%      | 51.6%        |
| 100     | 20,208 B     | 11,375 B     | 43.7%      | 57.8%        |
| 1,000   | 202,497 B    | 113,565 B    | 43.9%      | 58.5%        |
| 10,000  | 2,052,356 B  | 1,162,425 B  | 43.4%      | 58.6%        |

---

[Unreleased]: https://github.com/[org]/toon-ld/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/[org]/toon-ld/compare/v0.0.1...v0.1.0
[0.0.1]: https://github.com/[org]/toon-ld/releases/tag/v0.0.1