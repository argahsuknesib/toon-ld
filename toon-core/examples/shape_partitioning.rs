//! Example demonstrating shape-based partitioning for heterogeneous data
//!
//! This example shows how TOON-LD automatically partitions arrays with high
//! sparsity (>30% null values) into separate dense blocks grouped by entity shape.
//!
//! Run with: cargo run --example shape_partitioning

use serde_json::json;
use toon_core::ToonSerializer;

fn main() {
    println!("=== Shape-Based Partitioning Example ===\n");

    // Example 1: Heterogeneous RDF Graph (High Sparsity)
    println!("Example 1: Heterogeneous @graph with Persons and Organizations\n");

    let heterogeneous_graph = json!({
        "@context": {
            "foaf": "http://xmlns.com/foaf/0.1/",
            "org": "http://www.w3.org/ns/org#"
        },
        "@graph": [
            {
                "@id": "ex:person1",
                "@type": "foaf:Person",
                "foaf:name": "Alice",
                "foaf:age": 30,
                "foaf:email": "alice@example.com"
            },
            {
                "@id": "ex:person2",
                "@type": "foaf:Person",
                "foaf:name": "Bob",
                "foaf:age": 25,
                "foaf:email": "bob@example.com"
            },
            {
                "@id": "ex:org1",
                "@type": "org:Organization",
                "org:name": "ACME Corp",
                "org:industry": "Technology",
                "org:founded": 2000,
                "org:employees": 500
            },
            {
                "@id": "ex:org2",
                "@type": "org:Organization",
                "org:name": "XYZ Inc",
                "org:industry": "Finance",
                "org:founded": 1995,
                "org:employees": 300
            }
        ]
    });

    let serializer = ToonSerializer::new().with_shape_partitioning(true);
    let toon = serializer.serialize(&heterogeneous_graph).unwrap();

    println!("Input JSON-LD:");
    println!(
        "{}\n",
        serde_json::to_string_pretty(&heterogeneous_graph).unwrap()
    );

    println!("Output TOON-LD (with shape partitioning):");
    println!("{}\n", toon);

    println!("Notice: The @graph is split into 2 dense blocks:");
    println!("  - Block 1: 2 Person entities (shared shape)");
    println!("  - Block 2: 2 Organization entities (shared shape)");
    println!("  - No null values needed!\n");

    // Example 2: Same data WITHOUT partitioning (Union Schema)
    println!("=== For comparison: Same data WITHOUT partitioning ===\n");

    let serializer_no_partition = ToonSerializer::new().with_shape_partitioning(false);
    let toon_no_partition = serializer_no_partition
        .serialize(&heterogeneous_graph)
        .unwrap();

    println!("Output TOON-LD (union schema - partitioning disabled):");
    println!("{}\n", toon_no_partition);

    println!("Notice: Single @graph block with ALL fields and many null values.\n");

    // Example 3: Low Sparsity - No Partitioning Needed
    println!("=== Example 2: Low Sparsity (Homogeneous Data) ===\n");

    let homogeneous_data = json!({
        "users": [
            {"id": 1, "name": "Alice", "email": "alice@example.com"},
            {"id": 2, "name": "Bob", "email": "bob@example.com"},
            {"id": 3, "name": "Carol", "email": "carol@example.com"}
        ]
    });

    let toon_homogeneous = serializer.serialize(&homogeneous_data).unwrap();

    println!("Input JSON:");
    println!(
        "{}\n",
        serde_json::to_string_pretty(&homogeneous_data).unwrap()
    );

    println!("Output TOON-LD:");
    println!("{}\n", toon_homogeneous);

    println!("Notice: All entities have the same shape (low sparsity),");
    println!("so they remain in a single dense table - no partitioning needed.\n");

    // Example 4: Mixed Sparsity
    println!("=== Example 3: Mixed Sparsity (Automatic Decision) ===\n");

    let mixed_data = json!({
        "items": [
            {"@id": "item:1", "name": "Widget", "price": 10.99, "color": "blue"},
            {"@id": "item:2", "name": "Gadget", "price": 24.99, "color": "red"},
            {"@id": "item:3", "name": "Tool", "weight": 5.5, "material": "steel", "length": 30},
            {"@id": "item:4", "name": "Device", "voltage": 120, "power": 1000, "frequency": 60}
        ]
    });

    let toon_mixed = serializer.serialize(&mixed_data).unwrap();

    println!("Input JSON:");
    println!("{}\n", serde_json::to_string_pretty(&mixed_data).unwrap());

    println!("Output TOON-LD:");
    println!("{}\n", toon_mixed);

    println!("Notice: Items 1-2 have similar shape (price/color),");
    println!("while items 3-4 have completely different properties.");
    println!("High sparsity triggers automatic partitioning into 4 blocks.\n");

    // Summary
    println!("=== Summary ===\n");
    println!("Shape-based partitioning automatically activates when:");
    println!("  • Sparsity > 30% (null cells / total cells)");
    println!("  • Array contains objects with different property sets");
    println!("\nBenefits:");
    println!("  ✓ Reduces token count by eliminating null delimiters");
    println!("  ✓ Improves readability by grouping similar entities");
    println!("  ✓ Makes entity structure more apparent");
    println!("  ✓ Backwards compatible - parser merges blocks automatically");
    println!("\nConfiguration:");
    println!("  • Enabled by default: ToonSerializer::new()");
    println!("  • Disable: ToonSerializer::new().with_shape_partitioning(false)");
    println!("  • Sparsity threshold: 30% (SPARSITY_THRESHOLD constant)");
}
