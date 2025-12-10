use slint::SharedString;
use std::collections::HashMap;
use taxstud_core::{ClassicalHierarchy, HierarchyNode};

use super::types::{FacetInput, TreeNode};

/// Format facets into a displayable string
pub fn format_facets(facets: &HashMap<String, serde_json::Value>) -> String {
    let mut facet_lines: Vec<String> = facets
        .iter()
        .map(|(key, value)| {
            let value_str = match value {
                serde_json::Value::String(s) => s.clone(),
                serde_json::Value::Array(arr) => arr
                    .iter()
                    .filter_map(|v| v.as_str())
                    .collect::<Vec<_>>()
                    .join(", "),
                _ => value.to_string(),
            };
            format!("• {}: {}", key, value_str)
        })
        .collect();

    facet_lines.sort();
    facet_lines.join("\n")
}

/// Format facet dimensions into a displayable string
pub fn format_facet_dimensions(dimensions: &HashMap<String, Vec<String>>) -> String {
    let mut dim_lines: Vec<String> = dimensions
        .iter()
        .map(|(key, values)| format!("{}: {}", key, values.join(", ")))
        .collect();

    dim_lines.sort();
    dim_lines.join(" • ")
}

/// Create facet input list from taxonomy dimensions and current facet values
pub fn create_facet_inputs(
    dimensions: &HashMap<String, Vec<String>>,
    facets: &HashMap<String, serde_json::Value>,
) -> Vec<FacetInput> {
    let mut facet_inputs: Vec<FacetInput> = dimensions
        .keys()
        .map(|key| {
            let value = facets
                .get(key)
                .map(|v| match v {
                    serde_json::Value::String(s) => s.clone(),
                    serde_json::Value::Array(arr) => arr
                        .iter()
                        .filter_map(|v| v.as_str())
                        .collect::<Vec<_>>()
                        .join(", "),
                    _ => v.to_string(),
                })
                .unwrap_or_default();

            FacetInput {
                name: SharedString::from(key.as_str()),
                value: SharedString::from(value),
            }
        })
        .collect();

    facet_inputs.sort_by(|a, b| a.name.cmp(&b.name));
    facet_inputs
}

/// Flatten hierarchy tree into a list of tree nodes with indentation levels
pub fn flatten_hierarchy(hierarchy: &ClassicalHierarchy) -> Vec<TreeNode> {
    let mut nodes = Vec::new();

    if let Some(ref children) = hierarchy.children {
        for child in children {
            flatten_node(child, 0, &mut nodes);
        }
    }

    nodes
}

/// Recursively flatten a hierarchy node and its children
fn flatten_node(node: &HierarchyNode, indent_level: i32, nodes: &mut Vec<TreeNode>) {
    // Format: "species (differentia)"
    let label = if node.differentia.is_empty() {
        node.species.clone()
    } else {
        format!("{} ({})", node.species, node.differentia)
    };

    nodes.push(TreeNode {
        label: SharedString::from(label),
        indent_level,
    });

    if let Some(ref children) = node.children {
        for child in children {
            flatten_node(child, indent_level + 1, nodes);
        }
    }
}
