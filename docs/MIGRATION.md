# Migration Guide: v0.1.x to v0.2.0

This guide helps you migrate from TOON-LD v0.1.x to v0.2.0, which includes breaking API changes to improve naming consistency.

## Breaking Changes

All function names have been updated to use `toonld` instead of `toon` to accurately reflect that we're converting to/from **TOON-LD** format (not generic TOON).

## Rust (crates.io)

### Package Name Change

**Old (deprecated):**
```toml
[dependencies]
toon-core = "0.1"
```

**New (recommended):**
```toml
[dependencies]
toon-ld = "0.2"
```

The `toon-core` crate is deprecated. Please use `toon-ld` instead.

### Function Name Changes

**Old:**
```rust
use toon_core::{jsonld_to_toon, toon_to_jsonld};

let toon = jsonld_to_toon(json_ld)?;
let json = toon_to_jsonld(&toon)?;
```

**New:**
```rust
use toon_ld::{jsonld_to_toonld, toonld_to_jsonld};

let toon = jsonld_to_toonld(json_ld)?;
let json = toonld_to_jsonld(&toon)?;
```

### Migration Steps

1. Update your `Cargo.toml`:
   ```diff
   [dependencies]
   - toon-core = "0.1"
   + toon-ld = "0.2"
   ```

2. Update imports:
   ```diff
   - use toon_core::{jsonld_to_toon, toon_to_jsonld};
   + use toon_ld::{jsonld_to_toonld, toonld_to_jsonld};
   ```

3. Update function calls:
   ```diff
   - let toon = jsonld_to_toon(json_ld)?;
   + let toon = jsonld_to_toonld(json_ld)?;
   
   - let json = toon_to_jsonld(&toon)?;
   + let json = toonld_to_jsonld(&toon)?;
   ```

## Python (PyPI)

### Function Name Changes

| Old (v0.1.x) | New (v0.2.0) |
|--------------|--------------|
| `convert_jsonld_to_toon()` | `convert_jsonld_to_toonld()` |
| `convert_toon_to_jsonld()` | `convert_toonld_to_jsonld()` |
| `validate_toon()` | `validate_toonld()` |
| `parse_toon()` | `parse_toonld()` |
| `serialize_to_toon()` | `serialize_to_toonld()` |

### Migration Example

**Old:**
```python
import toon_ld

toon_str = toon_ld.convert_jsonld_to_toon(json_ld)
json_str = toon_ld.convert_toon_to_jsonld(toon_str)
```

**New:**
```python
import toon_ld

toon_str = toon_ld.convert_jsonld_to_toonld(json_ld)
json_str = toon_ld.convert_toonld_to_jsonld(toon_str)
```

### Migration Steps

1. Update package version:
   ```bash
   pip install --upgrade toon-ld
   ```

2. Find and replace in your codebase:
   ```bash
   # macOS/Linux
   sed -i 's/convert_jsonld_to_toon/convert_jsonld_to_toonld/g' *.py
   sed -i 's/convert_toon_to_jsonld/convert_toonld_to_jsonld/g' *.py
   sed -i 's/validate_toon(/validate_toonld(/g' *.py
   sed -i 's/parse_toon(/parse_toonld(/g' *.py
   sed -i 's/serialize_to_toon(/serialize_to_toonld(/g' *.py
   ```

## JavaScript/TypeScript (npm)

### Function Name Changes

| Old (v0.1.x) | New (v0.2.0) |
|--------------|--------------|
| `convert_jsonld_to_toon()` | `convert_jsonld_to_toonld()` |
| `convert_toon_to_jsonld()` | `convert_toonld_to_jsonld()` |
| `validate_toon()` | `validate_toonld()` |

### Migration Example

**Old:**
```javascript
import { convert_jsonld_to_toon, convert_toon_to_jsonld } from 'toon-ld';

const toon = convert_jsonld_to_toon(jsonLd);
const json = convert_toon_to_jsonld(toon);
```

**New:**
```javascript
import { convert_jsonld_to_toonld, convert_toonld_to_jsonld } from 'toon-ld';

const toon = convert_jsonld_to_toonld(jsonLd);
const json = convert_toonld_to_jsonld(toon);
```

### TypeScript Users

TypeScript definitions have been updated automatically. Your IDE will show errors for the old function names, making migration straightforward.

### Migration Steps

1. Update package version:
   ```bash
   npm install toon-ld@0.2.0
   ```

2. Update imports:
   ```diff
   - import { convert_jsonld_to_toon, convert_toon_to_jsonld } from 'toon-ld';
   + import { convert_jsonld_to_toonld, convert_toonld_to_jsonld } from 'toon-ld';
   ```

3. Update function calls:
   ```diff
   - const toon = convert_jsonld_to_toon(jsonLd);
   + const toon = convert_jsonld_to_toonld(jsonLd);
   
   - const json = convert_toon_to_jsonld(toon);
   + const json = convert_toonld_to_jsonld(toon);
   ```

## CLI

The CLI tool remains the same - no breaking changes:

```bash
# Still works the same way
toon-ld convert -i data.jsonld -o data.toon
toon-ld convert -i data.toon -o data.jsonld
```

## Why This Change?

The previous naming was misleading:
- **TOON** is the base format (like JSON)
- **TOON-LD** extends TOON with Linked Data semantics (like JSON-LD extends JSON)

The old function names (`jsonld_to_toon`) suggested we were converting to generic TOON, but we're actually converting to TOON-LD. The new names (`jsonld_to_toonld`) accurately reflect this.

## Automated Migration

### Rust Projects

```bash
# Find all occurrences
rg "jsonld_to_toon|toon_to_jsonld" --type rust

# Replace with sed (macOS)
find . -name "*.rs" -exec sed -i '' 's/jsonld_to_toon/jsonld_to_toonld/g' {} \;
find . -name "*.rs" -exec sed -i '' 's/toon_to_jsonld/toonld_to_jsonld/g' {} \;
```

### Python Projects

```bash
# Find all occurrences
grep -r "convert_jsonld_to_toon\|convert_toon_to_jsonld" .

# Replace with sed (macOS)
find . -name "*.py" -exec sed -i '' 's/convert_jsonld_to_toon/convert_jsonld_to_toonld/g' {} \;
find . -name "*.py" -exec sed -i '' 's/convert_toon_to_jsonld/convert_toonld_to_jsonld/g' {} \;
find . -name "*.py" -exec sed -i '' 's/validate_toon(/validate_toonld(/g' {} \;
find . -name "*.py" -exec sed -i '' 's/parse_toon(/parse_toonld(/g' {} \;
find . -name "*.py" -exec sed -i '' 's/serialize_to_toon(/serialize_to_toonld(/g' {} \;
```

### JavaScript/TypeScript Projects

```bash
# Find all occurrences
grep -r "convert_jsonld_to_toon\|convert_toon_to_jsonld" .

# Replace with sed (macOS)
find . \( -name "*.js" -o -name "*.ts" -o -name "*.jsx" -o -name "*.tsx" \) \
  -exec sed -i '' 's/convert_jsonld_to_toon/convert_jsonld_to_toonld/g' {} \;

find . \( -name "*.js" -o -name "*.ts" -o -name "*.jsx" -o -name "*.tsx" \) \
  -exec sed -i '' 's/convert_toon_to_jsonld/convert_toonld_to_jsonld/g' {} \;

find . \( -name "*.js" -o -name "*.ts" -o -name "*.jsx" -o -name "*.tsx" \) \
  -exec sed -i '' 's/validate_toon(/validate_toonld(/g' {} \;
```

## Support

If you encounter any issues during migration:

1. **GitHub Issues**: https://github.com/argahsuknesib/toon-ld/issues
2. **Documentation**: https://github.com/argahsuknesib/toon-ld#readme

## Changelog

See [CHANGELOG.md](CHANGELOG.md) for a complete list of changes in v0.2.0.