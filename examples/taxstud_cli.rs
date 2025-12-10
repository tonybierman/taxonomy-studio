use clap::Parser;
use std::process;
use taxstud_core::*;

/// Hybrid Taxonomy Browser - Filter and display items using faceted search
///
/// Examples:
///   # Display all items
///   faceted taxonomy.json
///
///   # Filter by genus (OR within genus)
///   faceted taxonomy.json --genus Coffee --genus Tea
///
///   # Filter by facet (OR within same facet name)
///   faceted taxonomy.json --facet temperature=hot --facet temperature=iced
///
///   # Combine filters (AND between different types)
///   faceted taxonomy.json --genus Coffee --facet caffeine_content=high
///
///   # Sort results by name
///   faceted taxonomy.json --sort name
///
///   # Group results by a facet
///   faceted taxonomy.json --group-by primary_theme
///
///   # Combine filtering, sorting, and grouping
///   faceted taxonomy.json --genus Coffee --sort name --group-by temperature
#[derive(Parser, Debug)]
#[command(name = "faceted")]
#[command(author, version, about, long_about = None)]
#[command(after_help = "Filtering Logic:\n  \
    - Multiple --genus values are combined with OR\n  \
    - Multiple --facet values for the SAME facet name are combined with OR\n  \
    - Different filter types (genus vs facets) are combined with AND\n  \
    - Different facet names are combined with AND\n\n\
Sorting Options:\n  \
    - name: Sort alphabetically by item name\n  \
    - Any facet name: Sort by that facet's value\n\n\
Grouping:\n  \
    - Group results by any facet name\n  \
    - Items with multiple values for the grouping facet appear in multiple groups")]
struct Cli {
    /// Path to the hybrid taxonomy JSON file
    #[arg(value_name = "FILE")]
    file: String,

    /// Filter by genus/species (can be specified multiple times for OR logic)
    #[arg(short, long = "genus", value_name = "NAME")]
    genera: Vec<String>,

    /// Filter by facet (format: facet_name=value, can be specified multiple times)
    #[arg(short, long = "facet", value_name = "NAME=VALUE")]
    facets: Vec<String>,

    /// Sort results by name or facet (e.g., "name", "temperature", "primary_theme")
    #[arg(short, long = "sort", value_name = "FIELD")]
    sort_by: Option<String>,

    /// Group results by a facet name
    #[arg(short = 'G', long = "group-by", value_name = "FACET")]
    group_by: Option<String>,
}

fn main() {
    let cli = Cli::parse();

    let taxonomy = load_taxonomy(&cli.file)
        .unwrap_or_else(|err| {
            eprintln!("Error loading taxonomy from '{}': {}", cli.file, err);
            process::exit(1);
        });

    // Validate the taxonomy schema
    if let Err(errors) = validate_taxonomy(&taxonomy) {
        eprintln!("Schema validation failed:\n");
        for (i, error) in errors.iter().enumerate() {
            eprintln!("  {}. {}", i + 1, error);
        }
        eprintln!("\nPlease fix these errors and try again.");
        process::exit(1);
    }

    let filters = parse_filters(&cli);

    if has_filters(&filters) || cli.sort_by.is_some() || cli.group_by.is_some() {
        print_filtered_taxonomy(&taxonomy, &filters, &cli);
    } else {
        print_taxonomy(&taxonomy);
    }
}

fn parse_filters(cli: &Cli) -> Filters {
    // Check for invalid facet formats and warn
    for facet_str in &cli.facets {
        if !facet_str.contains('=') {
            eprintln!("Warning: Invalid facet format '{}'. Expected 'name=value'", facet_str);
        }
    }

    let facet_map = parse_facet_filters(&cli.facets);

    Filters {
        genera: cli.genera.clone(),
        facets: facet_map,
    }
}

fn print_filtered_taxonomy(taxonomy: &HybridTaxonomy, filters: &Filters, cli: &Cli) {
    println!("# Filtered Results\n");

    if has_filters(filters) {
        println!("## Active Filters\n");

        if !filters.genera.is_empty() {
            println!("- **Genus:** {}", filters.genera.join(" OR "));
        }

        if !filters.facets.is_empty() {
            for (facet_name, values) in &filters.facets {
                println!("- **{}:** {}", facet_name, values.join(" OR "));
            }
        }
        println!();
    }

    if let Some(sort_field) = &cli.sort_by {
        println!("**Sorted by:** {}\n", sort_field);
    }

    if let Some(group_field) = &cli.group_by {
        println!("**Grouped by:** {}\n", group_field);
    }

    if let Some(examples) = &taxonomy.example_items {
        let mut filtered_items: Vec<_> = examples
            .iter()
            .filter(|item| matches_filters(item, filters))
            .cloned()
            .collect();

        println!("**Matching Items:** {}\n", filtered_items.len());

        if filtered_items.is_empty() {
            println!("_No items match the specified filters._\n");
        } else {
            // Apply sorting
            if let Some(sort_field) = &cli.sort_by {
                sort_items(&mut filtered_items, sort_field);
            }

            // Apply grouping or direct display
            if let Some(group_field) = &cli.group_by {
                print_grouped_items(&filtered_items, group_field);
            } else {
                for item in filtered_items.iter() {
                    print_example_item(item);
                }
            }
        }
    } else {
        println!("\n_No example items found in taxonomy._\n");
    }
}

fn print_grouped_items(items: &[Item], group_field: &str) {
    let groups = group_items_by_facet(items, group_field);
    let group_names = get_sorted_group_names(&groups);

    for group_name in group_names {
        if let Some(group_items) = groups.get(&group_name) {
            println!("## {}: {}\n", group_field, group_name);

            for item in group_items {
                print_example_item(item);
            }
        }
    }
}

fn print_taxonomy(taxonomy: &HybridTaxonomy) {
    println!("# Hybrid Taxonomy\n");

    if let Some(desc) = &taxonomy.taxonomy_description {
        println!("## Description\n");
        println!("{}\n", desc);
    }

    println!("## Classical Hierarchy\n");

    println!("**Root:** {}\n", taxonomy.classical_hierarchy.root);

    if let Some(children) = &taxonomy.classical_hierarchy.children {
        for child in children {
            print_hierarchy_node(child, 1);
        }
    }

    println!("\n## Faceted Dimensions\n");

    let mut facets: Vec<_> = taxonomy.faceted_dimensions.iter().collect();
    facets.sort_by_key(|(name, _)| *name);

    for (facet_name, values) in facets {
        println!("### {}\n", facet_name);
        for value in values.iter() {
            println!("- {}", value);
        }
        println!();
    }

    if let Some(examples) = &taxonomy.example_items {
        println!("## Example Items\n");

        for item in examples.iter() {
            print_example_item(item);
        }
    }

    if !taxonomy.extra.is_empty() {
        println!("## Additional Information\n");

        for (key, value) in &taxonomy.extra {
            println!("### {}\n", key);
            print_json_value(value, 0);
            println!();
        }
    }
}

fn print_hierarchy_node(node: &HierarchyNode, depth: usize) {
    let indent = "  ".repeat(depth);

    println!("{}* **{}**", indent, node.species);
    println!("{}  - Genus: {}", indent, node.genus);
    println!("{}  - Differentia: {}", indent, node.differentia);

    if let Some(children) = &node.children {
        for child in children {
            print_hierarchy_node(child, depth + 1);
        }
    }
}

fn print_example_item(item: &Item) {
    println!("### {}\n", item.name);

    println!("**Path:** {}\n", item.classical_path.join(" → "));

    println!("**Facets:**\n");
    let mut facets: Vec<_> = item.facets.keys().collect();
    facets.sort();

    for facet_name in facets {
        if let Some(value_str) = item.get_facet_as_string(facet_name) {
            println!("- {}: {}", facet_name, value_str);
        }
    }

    for (key, value) in &item.extra {
        if key != "name" && key != "classical_path" && key != "facets" {
            println!("\n**{}:** {}", key, value);
        }
    }

    println!();
}

fn print_json_value(value: &serde_json::Value, indent: usize) {
    let indent_str = "  ".repeat(indent);

    match value {
        serde_json::Value::Array(arr) => {
            for item in arr.iter() {
                match item {
                    serde_json::Value::String(s) => {
                        println!("{}- {}", indent_str, s);
                    }
                    _ => {
                        print_json_value(item, indent + 1);
                    }
                }
            }
        }
        serde_json::Value::String(s) => println!("{}{}", indent_str, s),
        _ => println!("{}{}", indent_str, value),
    }
}
