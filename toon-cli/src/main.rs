//! TOON-LD Command Line Interface
//!
//! A command-line tool for converting between JSON-LD and TOON-LD formats.

use anyhow::{Context, Result};
use clap::{Parser, Subcommand, ValueEnum};
use std::fs;
use std::io::{self, Read, Write};
use std::path::PathBuf;
use std::time::Instant;

#[derive(Parser)]
#[command(name = "toon-ld")]
#[command(
    author,
    version,
    about = "TOON-LD: Token-Oriented Object Notation for Linked Data"
)]
#[command(
    long_about = "A high-performance serializer/parser for TOON-LD format.\n\n\
    TOON-LD extends TOON to handle Linked Data (RDF), combining token-saving \
    Tabular Arrays with JSON-LD @context expansion."
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Convert between JSON-LD and TOON-LD formats
    Convert {
        /// Input file (use - for stdin)
        #[arg(short, long)]
        input: Option<PathBuf>,

        /// Output file (use - for stdout, default)
        #[arg(short, long)]
        output: Option<PathBuf>,

        /// Input format (auto-detected if not specified)
        #[arg(short, long, value_enum)]
        from: Option<Format>,

        /// Output format (required if input format is auto-detected)
        #[arg(short, long, value_enum)]
        to: Option<Format>,

        /// Pretty print JSON output
        #[arg(long, default_value = "true")]
        pretty: bool,
    },

    /// Validate a TOON-LD or JSON-LD file
    Validate {
        /// Input file (use - for stdin)
        #[arg(short, long)]
        input: Option<PathBuf>,

        /// Input format (auto-detected if not specified)
        #[arg(short, long, value_enum)]
        format: Option<Format>,

        /// Show detailed parsing information
        #[arg(short, long)]
        verbose: bool,
    },

    /// Show statistics and comparison between formats
    Stats {
        /// Input file (use - for stdin)
        #[arg(short, long)]
        input: Option<PathBuf>,

        /// Input format (auto-detected if not specified)
        #[arg(short, long, value_enum)]
        format: Option<Format>,

        /// Show token count estimate (approximation based on whitespace/punctuation)
        #[arg(long)]
        tokens: bool,
    },

    /// Run size/token savings benchmark with scaling record counts
    Benchmark {
        /// Base JSON-LD template file (optional, uses built-in template if not provided)
        #[arg(short, long)]
        input: Option<PathBuf>,

        /// Maximum number of records to test (will test 10, 100, 1000, up to this value)
        #[arg(short, long, default_value = "10000")]
        max_records: usize,
    },
}

#[derive(Copy, Clone, PartialEq, Eq, ValueEnum)]
enum Format {
    /// JSON-LD format
    #[value(name = "jsonld", alias = "json")]
    JsonLd,
    /// TOON-LD format
    #[value(name = "toon", alias = "toonld")]
    ToonLd,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Convert {
            input,
            output,
            from,
            to,
            pretty,
        } => cmd_convert(input, output, from, to, pretty),
        Commands::Validate {
            input,
            format,
            verbose,
        } => cmd_validate(input, format, verbose),
        Commands::Stats {
            input,
            format,
            tokens,
        } => cmd_stats(input, format, tokens),
        Commands::Benchmark { input, max_records } => cmd_benchmark(input, max_records),
    }
}

fn read_input(path: Option<PathBuf>) -> Result<String> {
    match path {
        Some(p) if p.to_string_lossy() != "-" => {
            fs::read_to_string(&p).with_context(|| format!("Failed to read file: {}", p.display()))
        }
        _ => {
            let mut buffer = String::new();
            io::stdin()
                .read_to_string(&mut buffer)
                .context("Failed to read from stdin")?;
            Ok(buffer)
        }
    }
}

fn write_output(path: Option<PathBuf>, content: &str) -> Result<()> {
    match path {
        Some(p) if p.to_string_lossy() != "-" => {
            fs::write(&p, content).with_context(|| format!("Failed to write file: {}", p.display()))
        }
        _ => io::stdout()
            .write_all(content.as_bytes())
            .context("Failed to write to stdout"),
    }
}

fn detect_format(content: &str) -> Format {
    let trimmed = content.trim();
    // JSON-LD starts with { or [
    if trimmed.starts_with('{') || trimmed.starts_with('[') {
        Format::JsonLd
    } else {
        Format::ToonLd
    }
}

fn cmd_convert(
    input: Option<PathBuf>,
    output: Option<PathBuf>,
    from: Option<Format>,
    to: Option<Format>,
    pretty: bool,
) -> Result<()> {
    let content = read_input(input)?;
    let source_format = from.unwrap_or_else(|| detect_format(&content));

    let target_format = to.unwrap_or(match source_format {
        Format::JsonLd => Format::ToonLd,
        Format::ToonLd => Format::JsonLd,
    });

    let result = match (source_format, target_format) {
        (Format::JsonLd, Format::ToonLd) => {
            toon_core::jsonld_to_toon(&content).context("Failed to convert JSON-LD to TOON-LD")?
        }
        (Format::ToonLd, Format::JsonLd) => {
            let json = toon_core::toon_to_jsonld(&content)
                .context("Failed to convert TOON-LD to JSON-LD")?;
            if pretty {
                let value: serde_json::Value = serde_json::from_str(&json)?;
                serde_json::to_string_pretty(&value)?
            } else {
                json
            }
        }
        (Format::JsonLd, Format::JsonLd) => {
            // Just validate and optionally pretty-print
            let value: serde_json::Value =
                serde_json::from_str(&content).context("Invalid JSON")?;
            if pretty {
                serde_json::to_string_pretty(&value)?
            } else {
                serde_json::to_string(&value)?
            }
        }
        (Format::ToonLd, Format::ToonLd) => {
            // Validate by parsing and re-serializing
            let json = toon_core::toon_to_jsonld(&content).context("Failed to parse TOON-LD")?;
            toon_core::jsonld_to_toon(&json).context("Failed to re-serialize TOON-LD")?
        }
    };

    write_output(output, &result)?;
    Ok(())
}

fn cmd_validate(input: Option<PathBuf>, format: Option<Format>, verbose: bool) -> Result<()> {
    let content = read_input(input)?;
    let source_format = format.unwrap_or_else(|| detect_format(&content));

    let start = Instant::now();
    let result = match source_format {
        Format::JsonLd => {
            let value: serde_json::Value =
                serde_json::from_str(&content).context("Invalid JSON")?;
            if verbose {
                describe_json_structure(&value, 0);
            }
            Ok(())
        }
        Format::ToonLd => {
            let parser = toon_core::ToonParser::new();
            let value = parser.parse(&content).context("Invalid TOON-LD")?;
            if verbose {
                describe_json_structure(&value, 0);
            }
            Ok(())
        }
    };
    let elapsed = start.elapsed();

    match result {
        Ok(()) => {
            eprintln!(
                "Valid {} (parsed in {:.3}ms)",
                match source_format {
                    Format::JsonLd => "JSON-LD",
                    Format::ToonLd => "TOON-LD",
                },
                elapsed.as_secs_f64() * 1000.0
            );
            Ok(())
        }
        Err(e) => Err(e),
    }
}

fn describe_json_structure(value: &serde_json::Value, depth: usize) {
    let indent = "  ".repeat(depth);
    match value {
        serde_json::Value::Null => eprintln!("{}null", indent),
        serde_json::Value::Bool(b) => eprintln!("{}bool: {}", indent, b),
        serde_json::Value::Number(n) => eprintln!("{}number: {}", indent, n),
        serde_json::Value::String(s) => {
            if s.len() > 50 {
                eprintln!("{}string: \"{}...\" ({} chars)", indent, &s[..47], s.len())
            } else {
                eprintln!("{}string: \"{}\"", indent, s)
            }
        }
        serde_json::Value::Array(arr) => {
            eprintln!("{}array[{}]:", indent, arr.len());
            if arr.len() <= 3 {
                for item in arr {
                    describe_json_structure(item, depth + 1);
                }
            } else {
                describe_json_structure(&arr[0], depth + 1);
                eprintln!("{}  ... ({} more items)", indent, arr.len() - 1);
            }
        }
        serde_json::Value::Object(obj) => {
            eprintln!("{}object ({} keys):", indent, obj.len());
            for (key, val) in obj.iter().take(10) {
                eprint!("{}  {}: ", indent, key);
                match val {
                    serde_json::Value::Object(o) => eprintln!("object ({} keys)", o.len()),
                    serde_json::Value::Array(a) => eprintln!("array[{}]", a.len()),
                    serde_json::Value::String(s) if s.len() > 30 => {
                        eprintln!("\"{}...\"", &s[..27])
                    }
                    _ => eprintln!("{}", val),
                }
            }
            if obj.len() > 10 {
                eprintln!("{}  ... ({} more keys)", indent, obj.len() - 10);
            }
        }
    }
}

fn cmd_stats(input: Option<PathBuf>, format: Option<Format>, show_tokens: bool) -> Result<()> {
    let content = read_input(input)?;
    let source_format = format.unwrap_or_else(|| detect_format(&content));

    // Get both representations
    let (jsonld, toonld) = match source_format {
        Format::JsonLd => {
            let toon =
                toon_core::jsonld_to_toon(&content).context("Failed to convert to TOON-LD")?;
            // Normalize JSON for fair comparison
            let value: serde_json::Value = serde_json::from_str(&content)?;
            let normalized = serde_json::to_string(&value)?;
            (normalized, toon)
        }
        Format::ToonLd => {
            let json =
                toon_core::toon_to_jsonld(&content).context("Failed to convert to JSON-LD")?;
            let value: serde_json::Value = serde_json::from_str(&json)?;
            let normalized = serde_json::to_string(&value)?;
            (normalized, content)
        }
    };

    let jsonld_bytes = jsonld.len();
    let toonld_bytes = toonld.len();
    let savings_bytes = jsonld_bytes as i64 - toonld_bytes as i64;
    let savings_percent = (savings_bytes as f64 / jsonld_bytes as f64) * 100.0;

    println!("=== Size Comparison ===");
    println!("JSON-LD:  {:>10} bytes", jsonld_bytes);
    println!("TOON-LD:  {:>10} bytes", toonld_bytes);
    println!(
        "Savings:  {:>10} bytes ({:+.1}%)",
        savings_bytes, savings_percent
    );
    println!();

    if show_tokens {
        let jsonld_tokens = estimate_tokens(&jsonld);
        let toonld_tokens = estimate_tokens(&toonld);
        let token_savings = jsonld_tokens as i64 - toonld_tokens as i64;
        let token_savings_percent = (token_savings as f64 / jsonld_tokens as f64) * 100.0;

        println!("=== Token Estimate (approx) ===");
        println!("JSON-LD:  {:>10} tokens", jsonld_tokens);
        println!("TOON-LD:  {:>10} tokens", toonld_tokens);
        println!(
            "Savings:  {:>10} tokens ({:+.1}%)",
            token_savings, token_savings_percent
        );
        println!();
    }

    // Line count
    let jsonld_lines = jsonld.lines().count();
    let toonld_lines = toonld.lines().count();
    println!("=== Line Count ===");
    println!("JSON-LD:  {:>10} lines", jsonld_lines);
    println!("TOON-LD:  {:>10} lines", toonld_lines);

    Ok(())
}

/// Rough token estimation based on common tokenizer patterns
/// This approximates GPT-style tokenization by counting:
/// - Words (whitespace-separated)
/// - Punctuation as separate tokens
/// - Numbers
fn estimate_tokens(text: &str) -> usize {
    let mut tokens = 0;
    let mut in_word = false;

    for c in text.chars() {
        if c.is_alphanumeric() || c == '_' {
            if !in_word {
                tokens += 1;
                in_word = true;
            }
        } else {
            in_word = false;
            // Punctuation and special characters are often separate tokens
            if !c.is_whitespace() {
                tokens += 1;
            }
        }
    }

    tokens
}

fn cmd_benchmark(input: Option<PathBuf>, max_records: usize) -> Result<()> {
    println!("=== TOON-LD Size & Token Savings Benchmark ===");
    println!();

    // Generate record counts: 10, 100, 1000, 10000, etc. up to max_records
    let mut record_counts = Vec::new();
    let mut count = 10;
    while count <= max_records {
        record_counts.push(count);
        count *= 10;
    }

    // If user provided a template, use it to extract the pattern
    let template = if let Some(path) = input {
        let content = read_input(Some(path))?;
        Some(content)
    } else {
        None
    };

    // Print header
    println!(
        "{:>10} | {:>12} | {:>12} | {:>10} | {:>12} | {:>10}",
        "Records", "JSON-LD", "TOON-LD", "Size Saved", "Token Est.", "Token Saved"
    );
    println!("{}", "-".repeat(78));

    for num_records in record_counts {
        let jsonld = if let Some(ref tmpl) = template {
            // Use template and try to expand it
            expand_template(tmpl, num_records)?
        } else {
            // Generate synthetic data
            generate_sample_jsonld(num_records)
        };

        let toonld = toon_core::jsonld_to_toon(&jsonld).context("Failed to convert to TOON-LD")?;

        // Normalize JSON for fair comparison
        let value: serde_json::Value = serde_json::from_str(&jsonld)?;
        let jsonld_compact = serde_json::to_string(&value)?;

        let jsonld_bytes = jsonld_compact.len();
        let toonld_bytes = toonld.len();
        let size_savings = (1.0 - toonld_bytes as f64 / jsonld_bytes as f64) * 100.0;

        let jsonld_tokens = estimate_tokens(&jsonld_compact);
        let toonld_tokens = estimate_tokens(&toonld);
        let token_savings = (1.0 - toonld_tokens as f64 / jsonld_tokens as f64) * 100.0;

        println!(
            "{:>10} | {:>10} B | {:>10} B | {:>9.1}% | {:>6}/{:>5} | {:>9.1}%",
            num_records,
            jsonld_bytes,
            toonld_bytes,
            size_savings,
            jsonld_tokens,
            toonld_tokens,
            token_savings
        );
    }

    println!();
    println!("Note: Token estimates are approximations based on whitespace/punctuation splitting.");

    Ok(())
}

/// Generate synthetic JSON-LD data for benchmarking
fn generate_sample_jsonld(num_records: usize) -> String {
    let mut graph = Vec::new();

    let names = [
        "Alice", "Bob", "Carol", "David", "Eva", "Frank", "Grace", "Henry", "Ivy", "Jack", "Kate",
        "Leo", "Mia", "Noah", "Olivia", "Peter", "Quinn", "Rose", "Sam", "Tina",
    ];
    let departments = [
        "Engineering",
        "Product",
        "Design",
        "Analytics",
        "Marketing",
        "Sales",
        "Support",
    ];
    let titles = [
        "Software Engineer",
        "Product Manager",
        "UX Designer",
        "Data Scientist",
        "DevOps Engineer",
        "QA Engineer",
        "Technical Writer",
        "Frontend Developer",
    ];

    for i in 1..=num_records {
        let name_idx = (i - 1) % names.len();
        let dept_idx = (i - 1) % departments.len();
        let title_idx = (i - 1) % titles.len();
        let age = 25 + (i % 30);

        graph.push(format!(
            r#"{{"@id":"http://example.org/person/{}","@type":"foaf:Person","foaf:name":"{} #{}","foaf:age":{},"foaf:mbox":"person{}@example.org","schema:jobTitle":"{}","schema:department":"{}"}}"#,
            i, names[name_idx], i, age, i, titles[title_idx], departments[dept_idx]
        ));
    }

    format!(
        r#"{{"@context":{{"foaf":"http://xmlns.com/foaf/0.1/","schema":"http://schema.org/","dc":"http://purl.org/dc/elements/1.1/"}},"@id":"http://example.org/dataset","@type":"schema:Dataset","dc:title":"Employee Directory","dc:description":"Benchmark dataset with {} records","@graph":[{}]}}"#,
        num_records,
        graph.join(",")
    )
}

/// Expand a template by replicating @graph entries
fn expand_template(template: &str, target_records: usize) -> Result<String> {
    let value: serde_json::Value =
        serde_json::from_str(template).context("Template must be valid JSON-LD")?;

    // Try to find @graph array
    if let Some(graph) = value.get("@graph").and_then(|g| g.as_array()) {
        if graph.is_empty() {
            // Fall back to synthetic data
            return Ok(generate_sample_jsonld(target_records));
        }

        // Replicate graph entries to reach target
        let mut new_graph = Vec::new();
        let base_len = graph.len();

        for i in 0..target_records {
            let mut entry = graph[i % base_len].clone();

            // Update @id to be unique
            if let Some(obj) = entry.as_object_mut() {
                if let Some(id) = obj.get("@id").and_then(|v| v.as_str()) {
                    let new_id = format!("{}_{}", id, i + 1);
                    obj.insert("@id".to_string(), serde_json::Value::String(new_id));
                }
            }

            new_graph.push(entry);
        }

        // Rebuild the document
        let mut new_doc = value.clone();
        if let Some(obj) = new_doc.as_object_mut() {
            obj.insert("@graph".to_string(), serde_json::Value::Array(new_graph));
        }

        Ok(serde_json::to_string(&new_doc)?)
    } else {
        // No @graph, fall back to synthetic
        Ok(generate_sample_jsonld(target_records))
    }
}
