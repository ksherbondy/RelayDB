use relay_compiler::{HEADER_SIZE, POINTER_START, solder_node};
use serde_json::Value;
use std::collections::{HashMap, HashSet};
use std::fs;
use std::io::{Result, Seek, SeekFrom, Write};

// --- Core Data Structures ---

struct GraphAnalysis {
    nodes: Vec<Value>,
    adj_list: HashMap<String, Vec<String>>,
}

/**
 * @bin compiler
 * @description The build engine for RelayDB.
 * Orchestrates the transition from raw JSON fragments to a unified
 * and validated 4-Tag Protocol binary.
 */
fn main() -> Result<()> {
    println!("--- RELAY-LINKER: v1.1 SMART-LINKER ACTIVATED ---");

    // 1. PHASE: Ingestion
    // Pulls raw data from the edge (JSON files) into memory.
    let analysis = ingest_data("../data")?;

    // 2. PHASE: Validation
    // The "Unbreakable Gate": Ensures no circular references exist before baking.
    println!("Validating Topology...");
    verify_no_cycles(&analysis.adj_list)
        .map_err(|e| {
            eprintln!("FATAL: Circular reference detected at '{}'. Aborting.", e);
            std::process::exit(1);
        })
        .ok();

    // 3. PHASE: Artifact Generation
    // Generates the .md audit and .dot visual schema for documentation.
    let dtg = chrono::Local::now().format("%Y%m%d_%H%M%S").to_string();
    generate_artifacts(&analysis, &dtg)?;

    // 4. PHASE: Binary Bake
    // Performs the final "solder" of the .relay binary file.
    bake_binary(&analysis, "bacon_standard.relay")?;

    Ok(())
}

// --- Logic Modules ---

fn ingest_data(data_path: &str) -> Result<GraphAnalysis> {
    let mut analysis = GraphAnalysis {
        nodes: Vec::new(),
        adj_list: HashMap::new(),
    };

    for entry in fs::read_dir(data_path)? {
        let path = entry?.path();
        if path.extension().and_then(|s| s.to_str()) == Some("json") {
            let content = fs::read_to_string(&path)?;
            let entries: Vec<Value> = serde_json::from_str(&content)
                .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;

            for node in entries {
                let id = node["#id"].as_str().unwrap_or("unknown").to_string();
                let mut links = Vec::new();

                if let Some(obj) = node.as_object() {
                    for (key, val) in obj {
                        // Capture Batons (@) and Topics (^) for the adjacency list
                        if (key.starts_with('@') || key.starts_with('^')) && val.is_string() {
                            links.push(val.as_str().unwrap().to_string());
                        }
                    }
                }
                analysis.adj_list.insert(id, links);
                analysis.nodes.push(node);
            }
        }
    }
    Ok(analysis)
}

fn generate_artifacts(analysis: &GraphAnalysis, dtg: &str) -> Result<()> {
    fs::create_dir_all("builds")?;

    // 3a. Generate Markdown Audit
    let mut hub_counts: HashMap<String, usize> = HashMap::new();
    for links in analysis.adj_list.values() {
        for link in links {
            *hub_counts.entry(link.clone()).or_insert(0) += 1;
        }
    }

    let md_path = format!("builds/relaySchema_{}.md", dtg);
    let mut md_file = fs::File::create(md_path)?;
    writeln!(md_file, "# RelayDB System Audit: {}\n", dtg)?;
    writeln!(
        md_file,
        "## 🛡️ Integrity Status\n- **Topology:** UNBREAKABLE ✅\n- **Cycle Detection:** Passed (Checked {} nodes)\n",
        analysis.nodes.len()
    )?;

    writeln!(md_file, "## 🚀 High-Frequency Anchors (Hubs)")?;
    let mut hubs: Vec<_> = hub_counts.into_iter().collect();
    hubs.sort_by(|a, b| b.1.cmp(&a.1));
    for (id, count) in hubs.iter().take(10) {
        writeln!(md_file, "- **{}**: {} incoming relationships", id, count)?;
    }

    generate_dot_file(analysis, dtg)?;
    Ok(())
}

fn generate_dot_file(analysis: &GraphAnalysis, dtg: &str) -> Result<()> {
    let dot_path = format!("builds/relaySchema_{}.dot", dtg);
    let mut f = fs::File::create(dot_path)?;
    writeln!(
        f,
        "digraph RelaySchema {{\n  rankdir=LR;\n  node [shape=box, style=filled, fillcolor=lightgray, fontname=\"Arial\"];"
    )?;
    writeln!(
        f,
        "  \"RelayDB_Root\" [shape=cylinder, fillcolor=gold, label=\"bacon_standard.relay\"];"
    )?;

    let categories = [
        ("directors", "royalblue1"),
        ("movies", "tomato"),
        ("actors", "springgreen3"),
    ];
    for (cat, color) in categories {
        writeln!(
            f,
            "  subgraph cluster_{} {{ label=\"{}\"; style=dashed; color=\"{}\";",
            cat,
            cat.to_uppercase(),
            color
        )?;
        for (node, links) in &analysis.adj_list {
            if links.contains(&cat.to_string()) {
                writeln!(f, "    \"{}\" [fillcolor=\"{}\"];", node, color)?;
            }
        }
        writeln!(f, "  }}")?;
    }

    for (node, links) in &analysis.adj_list {
        for link in links {
            if ["directors", "movies", "actors"].contains(&link.as_str()) {
                writeln!(f, "  \"RelayDB_Root\" -> \"{}\" [style=bold];", link)?;
            }
            writeln!(f, "  \"{}\" -> \"{}\";", node, link)?;
        }
    }
    writeln!(f, "}}")?;
    Ok(())
}

fn bake_binary(analysis: &GraphAnalysis, output_path: &str) -> Result<()> {
    println!("Validation passed. Soldering binary...");
    let mut file = fs::File::create(output_path)?;

    // Step A: Reserve space for the header using the library constant
    file.write_all(&vec![0u8; HEADER_SIZE as usize])?;

    // Step B: Solder entries via lib.rs protocol
    let mut jump_table: HashMap<String, u64> = HashMap::new();
    for entry in &analysis.nodes {
        let id = entry["#id"].as_str().unwrap_or("unknown").to_string();

        // DELEGATION: The library now handles the physical write and terminator logic
        let pos = solder_node(&mut file, entry)?;

        jump_table.insert(id, pos);
    }

    // Step C: Write Jump Table at end of file
    let index_pos = file.stream_position()?;
    for (id, offset) in &jump_table {
        writeln!(file, "{}:{}", id, offset)?;
    }

    // Step D: Standardized Pointer
    // Write the index location to the Header using library constant (Byte 16)
    file.seek(SeekFrom::Start(POINTER_START))?;
    file.write_all(&index_pos.to_le_bytes())?;

    println!("SUCCESS: '{}' is soldered.", output_path);
    Ok(())
}

// --- Validation Helpers ---

fn verify_no_cycles(adj: &HashMap<String, Vec<String>>) -> std::result::Result<(), String> {
    let mut visited = HashSet::new();
    let mut stack = HashSet::new();
    for node in adj.keys() {
        if has_cycle(node, adj, &mut visited, &mut stack) {
            return Err(node.clone());
        }
    }
    Ok(())
}

fn has_cycle(
    node: &String,
    adj: &HashMap<String, Vec<String>>,
    visited: &mut HashSet<String>,
    stack: &mut HashSet<String>,
) -> bool {
    if stack.contains(node) {
        return true;
    }
    if visited.contains(node) {
        return false;
    }
    visited.insert(node.clone());
    stack.insert(node.clone());
    if let Some(neighbors) = adj.get(node) {
        for neighbor in neighbors {
            if has_cycle(neighbor, adj, visited, stack) {
                return true;
            }
        }
    }
    stack.remove(node);
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_verify_no_cycles_safe() {
        let mut adj = HashMap::new();
        adj.insert("A".to_string(), vec!["B".to_string()]);
        adj.insert("B".to_string(), vec!["C".to_string()]);
        adj.insert("C".to_string(), vec![]);
        assert!(verify_no_cycles(&adj).is_ok());
    }

    #[test]
    fn test_verify_no_cycles_fail() {
        let mut adj = HashMap::new();
        adj.insert("A".to_string(), vec!["B".to_string()]);
        adj.insert("B".to_string(), vec!["A".to_string()]);
        assert!(verify_no_cycles(&adj).is_err());
    }
}
