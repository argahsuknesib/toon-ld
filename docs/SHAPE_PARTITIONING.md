# Shape-Based Partitioning

## Overview

Shape-based partitioning is an automatic optimization feature in TOON-LD that reduces token usage and improves readability when serializing heterogeneous data. Instead of forcing all entities into a single sparse table with many null values, the serializer intelligently groups entities by their property structure (shape) and emits multiple dense blocks.

## Motivation

When processing heterogeneous RDF graphs or mixed-type data, the traditional "union schema" approach creates sparse tables:

```toon
@graph[3]{@id, @type, name, age, email, industry, founded}:
  ex:person1, Person, Alice, 30, alice@example.com, null, null
  ex:person2, Person, Bob, 25, bob@example.com, null, null
  ex:org1, Organization, ACME, null, null, Technology, 2000
```

**Problems:**
- **Token waste**: Empty delimiter sequences (`, null, null`) consume tokens without providing information
- **Poor readability**: Sparse tables obscure the actual data structure
- **Semantic confusion**: Unrelated properties appear in the same table

## How It Works

Shape-based partitioning automatically activates when:
1. An array contains objects (not primitives)
2. Sparsity exceeds 30% (calculated as `null_cells / total_cells`)

### Algorithm

1. **Signature Generation**: For each entity, generate a deterministic signature by sorting its keys alphabetically and joining with `|`
   - `{"age": 30, "name": "Alice"}` → `"age|name"`
   - `{"name": "Bob", "age": 25}` → `"age|name"` (same signature)

2. **Grouping**: Group entities with identical signatures

3. **Serialization**: Emit separate array blocks for each group, ordered by size (largest first)

### Output Format

With partitioning enabled:

```toon
@graph[2]{@id, @type, name, age, email}:
  ex:person1, Person, Alice, 30, alice@example.com
  ex:person2, Person, Bob, 25, bob@example.com

@graph[1]{@id, @type, name, industry, founded}:
  ex:org1, Organization, ACME, Technology, 2000
```

**Benefits:**
- No null delimiters needed
- Each block is dense and self-describing
- Related entities are grouped together
- Reduced token count for sparse data

## Configuration

### Enable (Default)

```rust
use toon_core::ToonSerializer;

let serializer = ToonSerializer::new(); // Partitioning enabled by default
```

### Disable

```rust
let serializer = ToonSerializer::new().with_shape_partitioning(false);
```

### Adjust Threshold

The sparsity threshold is defined as a constant in `serializer.rs`:

```rust
const SPARSITY_THRESHOLD: f64 = 0.30; // 30%
```

To change it, modify this constant and recompile.

## Parser Compatibility

The parser automatically handles multiple array blocks with the same key by **merging** them:

```toon
items[2]{a,b}:
  1, 2
  3, 4

items[1]{x,y}:
  5, 6
```

Parses to:

```json
{
  "items": [
    {"a": 1, "b": 2},
    {"a": 3, "b": 4},
    {"x": 5, "y": 6}
  ]
}
```

This ensures full round-trip compatibility.

## Examples

### Example 1: Heterogeneous RDF Graph

**Input:**
```json
{
  "@graph": [
    {"@id": "ex:1", "@type": "Person", "name": "Alice", "age": 30},
    {"@id": "ex:2", "@type": "Person", "name": "Bob", "age": 25},
    {"@id": "ex:3", "@type": "Org", "name": "ACME", "industry": "Tech"}
  ]
}
```

**Output (with partitioning):**
```toon
@graph[2]{@id,@type,age,name}:
  ex:1, Person, 30, Alice
  ex:2, Person, 25, Bob

@graph[1]{@id,@type,industry,name}:
  ex:3, Org, Tech, ACME
```

**Sparsity calculation:**
- Total cells: 3 rows × 5 fields = 15
- Null cells: 2×1 + 1×1 = 3
- Sparsity: 3/15 = 20%

Since 20% < 30%, this specific example wouldn't trigger partitioning. To exceed the threshold, you need more field diversity.

### Example 2: Highly Diverse Entities

**Input:**
```json
{
  "items": [
    {"@id": "item:1", "name": "Widget", "price": 10.99},
    {"@id": "item:2", "name": "Tool", "weight": 5.5, "material": "steel"},
    {"@id": "item:3", "name": "Device", "voltage": 120, "power": 1000}
  ]
}
```

**Output (with partitioning):**
```toon
items[1]{@id,name,price}:
  item:1, Widget, 10.99

items[1]{@id,material,name,weight}:
  item:2, Tool, steel, 5.5

items[1]{@id,name,power,voltage}:
  item:3, Device, 1000, 120
```

**Sparsity calculation:**
- Total cells: 3 rows × 6 unique fields = 18
- Null cells: 12 (each row missing 4 fields)
- Sparsity: 12/18 = 66.7%

Partitioning activated! Each entity becomes its own dense block.

### Example 3: Low Sparsity (No Partitioning)

**Input:**
```json
{
  "users": [
    {"id": 1, "name": "Alice", "email": "alice@example.com"},
    {"id": 2, "name": "Bob", "email": "bob@example.com"},
    {"id": 3, "name": "Carol", "email": "carol@example.com"}
  ]
}
```

**Output:**
```toon
users[3]{email,id,name}:
  alice@example.com, 1, Alice
  bob@example.com, 2, Bob
  carol@example.com, 3, Carol
```

All entities share the same shape → 0% sparsity → single dense table (no partitioning).

## Performance Characteristics

### When Partitioning Helps

- **High field diversity**: Entities have mostly different properties
- **Large datasets**: More entities mean more token savings
- **Heterogeneous graphs**: Mixed entity types with distinct schemas

### When Partitioning Hurts

- **Low sparsity**: Most entities share most fields (adds header overhead)
- **Small entity groups**: Overhead of multiple headers exceeds savings
- **Very large shared key sets**: Repeating common keys in multiple headers

### Token Cost Analysis

**Union Schema:**
- Cost = `header + (rows × columns × avg_delimiter_size)`
- High cost when `null_count` is large

**Partitioned Schema:**
- Cost = `Σ(partition_header + partition_rows × partition_columns × avg_delimiter_size)`
- Low cost when partitions have dense, non-overlapping fields

**Break-even point:** ~30% sparsity threshold balances both approaches.

## Implementation Details

### Key Functions

- `calculate_sparsity(&self, arr: &[Value], fields: &[String]) -> f64`
  - Computes ratio of null cells to total cells

- `entity_signature(&self, obj: &Map<String, Value>) -> String`
  - Generates deterministic signature from sorted keys

- `partition_by_shape<'a>(&self, arr: &'a [Value]) -> Vec<(String, Vec<String>, Vec<&'a Value>)>`
  - Groups entities by signature, returns sorted partitions

- `serialize_partitioned_array(&self, key: &str, arr: &[Value], depth: usize, output: &mut String)`
  - Emits multiple array blocks for each shape group

### Parser Changes

The parser was updated to **accumulate** rather than **overwrite** when encountering duplicate array keys:

```rust
// Check if this key already exists - if so, we'll append to it
let should_merge = obj.contains_key(&key.to_string());

if should_merge {
    if let Some(Value::Array(existing)) = obj.get(&key.to_string()) {
        current_array = existing.clone();
        current_array.reserve(count);
    }
}
```

This ensures multiple `@graph[N]{...}:` blocks merge into a single JSON array.

## Testing

Run the comprehensive test suite:

```bash
cargo test -p toon-core --lib
```

Key tests:
- `test_shape_partitioning_disabled` - Verifies override works
- `test_shape_partitioning_low_sparsity` - No partitioning for dense data
- `test_shape_partitioning_high_sparsity` - Partitioning for sparse data
- `test_shape_partitioning_heterogeneous_graph` - Real-world RDF scenario
- `test_shape_partitioning_roundtrip` - Parser correctly merges blocks
- `test_calculate_sparsity` - Sparsity calculation accuracy
- `test_entity_signature` - Signature consistency

Run the example:

```bash
cargo run --package toon-core --example shape_partitioning
```

## Future Enhancements

### Potential Improvements

1. **Adaptive Threshold**: Calculate optimal threshold per-dataset based on actual token costs
2. **Configurable Threshold**: Expose as serializer parameter rather than compile-time constant
3. **Minimum Partition Size**: Don't partition if groups are too small (e.g., < 2 entities)
4. **Hybrid Mode**: Use union schema for small overlap, partition for high divergence
5. **Metrics**: Log sparsity and partitioning decisions for observability

### Non-Goals

- **Perfect Optimization**: Not trying to find globally optimal partitioning (NP-hard)
- **Schema Inference**: Not inferring types or generating formal schemas
- **Columnar Storage**: Staying with row-oriented tabular format

## FAQ

### Q: Does this break backwards compatibility?

**A:** No. The parser automatically merges multiple array blocks. Old parsers would only see the last block, but the format itself is valid TOON-LD.

### Q: Can I force partitioning even with low sparsity?

**A:** Not directly. You'd need to lower `SPARSITY_THRESHOLD` and recompile. Future versions may expose this as a parameter.

### Q: What happens if all entities have unique shapes?

**A:** Each entity becomes its own block. This is optimal for maximum sparsity (saves the most tokens).

### Q: Does this work with nested objects?

**A:** Partitioning only applies to arrays of objects at the top level of serialization or as property values. Nested structures within objects are serialized normally.

### Q: How does this interact with JSON-LD context?

**A:** Keys are compacted using the context before comparison, so `foaf:name` and `http://xmlns.com/foaf/0.1/name` are treated as the same property.

### Q: What about ordering?

**A:** Partitions are emitted largest-first for readability. Within each partition, entities maintain their original order.

## References

- Original Issue: [GitHub Issue describing the motivation]
- TOON-LD Specification: `../spec/index.html`
- Implementation: `../toon-core/src/serializer.rs` (lines 754-865)
- Parser Support: `../toon-core/src/parser.rs` (lines 242-265)

## Changelog

- **v0.2.0**: Initial implementation of shape-based partitioning
  - Added `with_shape_partitioning()` configuration method
  - Parser support for merging multiple array blocks
  - Default threshold: 30% sparsity
  - Comprehensive test coverage
