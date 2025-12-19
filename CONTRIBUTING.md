# Contributing to TOON-LD

Thank you for your interest in contributing to TOON-LD. This document provides guidelines and instructions for contributing.

## Table of Contents

- [Code of Conduct](#code-of-conduct)
- [Getting Started](#getting-started)
- [Development Setup](#development-setup)
- [Making Changes](#making-changes)
- [Pull Request Process](#pull-request-process)
- [Coding Standards](#coding-standards)
- [Testing Guidelines](#testing-guidelines)
- [Documentation](#documentation)
- [Release Process](#release-process)

## Code of Conduct

This project follows the [Rust Code of Conduct](https://www.rust-lang.org/policies/code-of-conduct). Please be respectful and constructive in all interactions.

## Getting Started

### Prerequisites

- Rust 1.70.0 or later
- Python 3.9+ (for Python bindings)
- Node.js 18+ (for WASM package testing)
- wasm-pack (for building WASM)
- maturin (for building Python wheels)

### Repository Structure

```
toon-ld/
├── toon-core/      # Core Rust library
├── toon-cli/       # Command-line interface
├── toon-wasm/      # WebAssembly bindings
├── toon-py/        # Python bindings (PyO3)
├── examples/       # Example files
└── SPECIFICATION.md # Format specification
```

## Development Setup

1. **Clone the repository:**
   ```bash
   git clone https://github.com/[org]/toon-ld.git
   cd toon-ld
   ```

2. **Build all crates:**
   ```bash
   cargo build --workspace
   ```

3. **Run tests:**
   ```bash
   cargo test --workspace
   ```

4. **Build WASM (optional):**
   ```bash
   cd toon-wasm
   wasm-pack build --target web
   ```

5. **Build Python wheel (optional):**
   ```bash
   cd toon-py
   pip install maturin
   maturin develop
   ```

## Making Changes

### Branch Naming

Use descriptive branch names:
- `feature/description` - New features
- `fix/description` - Bug fixes
- `docs/description` - Documentation updates
- `refactor/description` - Code refactoring
- `ci/description` - CI/CD changes

### Commit Messages

Follow conventional commit format:

```
type(scope): brief description

Longer description if needed.

Fixes #123
```

Types: `feat`, `fix`, `docs`, `style`, `refactor`, `test`, `ci`, `chore`

Scopes: `core`, `cli`, `wasm`, `py`, `spec`

Examples:
```
feat(core): add support for @nest keyword
fix(cli): handle empty input files gracefully
docs(spec): clarify quoting rules for tabular arrays
```

## Pull Request Process

1. **Create a feature branch** from `main`

2. **Make your changes** following the coding standards

3. **Add tests** for any new functionality

4. **Run the full test suite:**
   ```bash
   cargo test --workspace
   cargo clippy --workspace -- -D warnings
   cargo fmt --check
   ```

5. **Update documentation** if needed

6. **Submit a pull request** with a clear description

7. **Address review feedback** promptly

### PR Requirements

- All CI checks must pass
- At least one approval from a maintainer
- No unresolved conversations
- Up-to-date with the target branch

## Coding Standards

### Rust

- Follow the [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/)
- Run `cargo fmt` before committing
- Run `cargo clippy` and address all warnings
- Document all public APIs with doc comments
- Use meaningful variable and function names

### Error Handling

- Use the `ToonError` enum for all errors
- Provide helpful error messages with context
- Include line numbers in parse errors

### Performance

- Avoid unnecessary allocations in hot paths
- Use `&str` over `String` where possible
- Consider using `Cow<str>` for flexible ownership
- Profile before optimizing

## Testing Guidelines

### Unit Tests

Add unit tests in the same file as the code being tested:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_feature_name() {
        // Arrange
        let input = r#"{"test": "value"}"#;
        
        // Act
        let result = some_function(input);
        
        // Assert
        assert!(result.is_ok());
    }
}
```

### Test Naming

Use descriptive test names that explain what is being tested:

```rust
#[test]
fn test_tabular_array_with_missing_fields_inserts_null() { ... }

#[test]
fn test_quoted_string_with_comma_preserves_value() { ... }
```

### Integration Tests

For larger tests that span multiple modules, add files to `tests/`:

```
toon-core/tests/
├── roundtrip.rs
├── jsonld_keywords.rs
└── edge_cases.rs
```

### Test Coverage

Aim for high test coverage, especially for:
- All JSON-LD keywords
- Edge cases in parsing and serialization
- Error conditions
- Round-trip fidelity

## Documentation

### Code Documentation

- All public functions, structs, and modules must have doc comments
- Include examples in doc comments where helpful
- Use `# Examples` and `# Errors` sections in doc comments

```rust
/// Serializes a JSON value to TOON-LD format.
///
/// # Arguments
///
/// * `value` - The JSON value to serialize
///
/// # Returns
///
/// A `Result` containing the TOON-LD string or a `ToonError`
///
/// # Examples
///
/// ```
/// use toon_core::ToonSerializer;
/// use serde_json::json;
///
/// let serializer = ToonSerializer::new();
/// let value = json!({"name": "Alice"});
/// let toon = serializer.serialize(&value)?;
/// assert!(toon.contains("name: Alice"));
/// # Ok::<(), toon_core::ToonError>(())
/// ```
pub fn serialize(&self, value: &Value) -> Result<String> {
    // ...
}
```

### README Updates

Update the README when:
- Adding new features
- Changing CLI commands
- Adding new JSON-LD keyword support
- Changing installation instructions

### Specification Updates

If your changes affect the format itself:
1. Update `SPECIFICATION.md`
2. Include the change in PR description
3. Consider backward compatibility implications

## Release Process

Releases are managed by maintainers. The process:

1. Update version in all `Cargo.toml` files
2. Update `CHANGELOG.md`
3. Create a git tag: `git tag v0.x.x`
4. Push the tag: `git push origin v0.x.x`
5. CI will automatically publish to crates.io, npm, and PyPI

### Version Numbering

We follow [Semantic Versioning](https://semver.org/):
- MAJOR: Breaking changes to the format or API
- MINOR: New features, backward compatible
- PATCH: Bug fixes, backward compatible

## Getting Help

- Open an issue for bugs or feature requests
- Use discussions for questions
- Check existing issues before creating new ones

## Recognition

Contributors will be recognized in:
- The CHANGELOG for their contributions
- The GitHub contributors page
- Release notes when applicable

Thank you for contributing to TOON-LD.