use clap::Parser;
use relay_compiler::{
    HEADER_SIZE, POINTER_START, extract_anchor_id, extract_links_from_node, solder_node,
};
use serde_json::Value;
use std::collections::{HashMap, HashSet};
use std::fs;
use std::io::{Result, Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};

// --- CLI Args ---

#[derive(Parser, Debug)]
#[command(name = "relay-compiler")]
#[command(version = "1.2")]
#[command(about = "Universal RelayDB compiler for .json and .jsonl source memory")]
struct Args {
    /// Input file or directory containing .json / .jsonl files
    #[arg(short, long, default_value = "../data")]
    input: PathBuf,

    /// Output .relay filename
    #[arg(short, long, default_value = "output.relay")]
    output: PathBuf,

    /// Directory for generated audit artifacts
    #[arg(short = 'b', long, default_value = "builds")]
    builds: PathBuf,

    /// Fail the build if the relationship graph contains cycles.
    /// Default is false because ATLAS/project-memory graphs commonly contain valid semantic cycles.
    #[arg(long, default_value_t = false)]
    strict_acyclic: bool,
}

// --- Core Data Structures ---

struct GraphAnalysis {
    nodes: Vec<Value>,
    adj_list: HashMap<String, Vec<String>>,
}

/**
 * @bin compiler
 * @description Universal build engine for RelayDB.
 * Converts .json and .jsonl files into a validated .relay binary.
 */
fn main() -> Result<()> {
    let args = Args::parse();

    println!("--- RELAY-LINKER: v1.2 UNIVERSAL COMPILER ACTIVATED ---");
    println!("Input:  {}", args.input.display());
    println!("Output: {}", args.output.display());

    // 1. PHASE: Ingestion
    let analysis = ingest_data(&args.input)?;

    // 2. PHASE: Validation
    println!(
        "Validating topology across {} nodes...",
        analysis.nodes.len()
    );

    match verify_no_cycles(&analysis.adj_list) {
        Ok(()) => {
            println!("Topology check passed: no cycles detected.");
        }
        Err(e) if args.strict_acyclic => {
            eprintln!("FATAL: Circular reference detected at '{}'. Aborting.", e);
            std::process::exit(1);
        }
        Err(e) => {
            println!(
                "WARNING: Circular reference detected at '{}'. Continuing because strict acyclic mode is off.",
                e
            );
        }
    }

    // 3. PHASE: Artifact Generation
    let dtg = chrono::Local::now().format("%Y%m%d_%H%M%S").to_string();
    generate_artifacts(&analysis, &dtg, &args.builds, &args.output)?;

    // 4. PHASE: Binary Bake
    bake_binary(&analysis, &args.output)?;

    Ok(())
}

// --- Ingestion ---

fn ingest_data(input_path: &Path) -> Result<GraphAnalysis> {
    let mut analysis = GraphAnalysis {
        nodes: Vec::new(),
        adj_list: HashMap::new(),
    };

    let files = collect_input_files(input_path)?;

    for path in files {
        let mut nodes = read_nodes_from_file(&path)?;

        for node in nodes.drain(..) {
            let id = extract_anchor_id(&node).unwrap_or("unknown").to_string();

            let links = extract_links_from_node(&node);

            analysis.adj_list.insert(id, links);
            analysis.nodes.push(node);
        }
    }

    Ok(analysis)
}

fn collect_input_files(input_path: &Path) -> Result<Vec<PathBuf>> {
    let mut files = Vec::new();

    if input_path.is_file() {
        if is_supported_data_file(input_path) {
            files.push(input_path.to_path_buf());
        }
        return Ok(files);
    }

    for entry in fs::read_dir(input_path)? {
        let path = entry?.path();

        if path.is_file() && is_supported_data_file(&path) {
            files.push(path);
        }
    }

    files.sort();
    Ok(files)
}

fn is_supported_data_file(path: &Path) -> bool {
    matches!(
        path.extension().and_then(|s| s.to_str()),
        Some("json") | Some("jsonl")
    )
}

fn read_nodes_from_file(path: &Path) -> Result<Vec<Value>> {
    match path.extension().and_then(|s| s.to_str()) {
        Some("json") => read_json_nodes(path),
        Some("jsonl") => read_jsonl_nodes(path),
        _ => Ok(Vec::new()),
    }
}

fn read_json_nodes(path: &Path) -> Result<Vec<Value>> {
    let content = fs::read_to_string(path)?;
    let parsed: Value = serde_json::from_str(&content)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;

    if let Some(list) = parsed.as_array() {
        Ok(list.clone())
    } else if parsed.is_object() {
        Ok(vec![parsed])
    } else {
        Err(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            format!("{} must contain a JSON object or array", path.display()),
        ))
    }
}

fn read_jsonl_nodes(path: &Path) -> Result<Vec<Value>> {
    let content = fs::read_to_string(path)?;
    let mut nodes = Vec::new();

    for (line_index, line) in content.lines().enumerate() {
        let trimmed = line.trim();

        if trimmed.is_empty() {
            continue;
        }

        let node: Value = serde_json::from_str(trimmed).map_err(|e| {
            std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!(
                    "{} line {} is not valid JSONL: {}",
                    path.display(),
                    line_index + 1,
                    e
                ),
            )
        })?;

        if !node.is_object() {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!(
                    "{} line {} must be a JSON object",
                    path.display(),
                    line_index + 1
                ),
            ));
        }

        nodes.push(node);
    }

    Ok(nodes)
}

// --- Artifact Generation ---

fn generate_artifacts(
    analysis: &GraphAnalysis,
    dtg: &str,
    builds_dir: &Path,
    output_path: &Path,
) -> Result<()> {
    fs::create_dir_all(builds_dir)?;

    let mut hub_counts: HashMap<String, usize> = HashMap::new();

    for links in analysis.adj_list.values() {
        for link in links {
            *hub_counts.entry(link.clone()).or_insert(0) += 1;
        }
    }

    let md_path = builds_dir.join(format!("relaySchema_{}.md", dtg));
    let mut md_file = fs::File::create(md_path)?;

    writeln!(md_file, "# RelayDB System Audit: {}\n", dtg)?;
    writeln!(md_file, "## Build Output")?;
    writeln!(md_file, "- **Relay File:** `{}`", output_path.display())?;
    writeln!(md_file, "- **Nodes:** {}", analysis.nodes.len())?;
    writeln!(md_file)?;

    writeln!(md_file, "## Integrity Status")?;

    match verify_no_cycles(&analysis.adj_list) {
        Ok(()) => {
            writeln!(md_file, "- **Topology:** Acyclic ✅")?;
            writeln!(md_file, "- **Cycle Detection:** No cycles detected")?;
        }
        Err(e) => {
            writeln!(md_file, "- **Topology:** Relational graph with cycles ⚠️")?;
            writeln!(
                md_file,
                "- **Cycle Detection:** First detected cycle near `{}`",
                e
            )?;
            writeln!(
                md_file,
                "- **Note:** Cycles are allowed unless `--strict-acyclic` is enabled."
            )?;
        }
    }

    writeln!(md_file)?;

    writeln!(md_file, "## High-Frequency Anchors / Hubs")?;
    let mut hubs: Vec<_> = hub_counts.into_iter().collect();
    hubs.sort_by(|a, b| b.1.cmp(&a.1));

    if hubs.is_empty() {
        writeln!(md_file, "- No relationships found.")?;
    } else {
        for (id, count) in hubs.iter().take(10) {
            writeln!(md_file, "- **{}**: {} incoming relationships", id, count)?;
        }
    }

    generate_dot_file(analysis, dtg, builds_dir, output_path)?;
    Ok(())
}

fn generate_dot_file(
    analysis: &GraphAnalysis,
    dtg: &str,
    builds_dir: &Path,
    output_path: &Path,
) -> Result<()> {
    let dot_path = builds_dir.join(format!("relaySchema_{}.dot", dtg));
    let mut f = fs::File::create(dot_path)?;

    writeln!(f, "digraph RelaySchema {{")?;
    writeln!(f, "  rankdir=LR;")?;
    writeln!(
        f,
        "  node [shape=box, style=filled, fillcolor=lightgray, fontname=\"Arial\"];"
    )?;
    writeln!(
        f,
        "  \"RelayDB_Root\" [shape=cylinder, fillcolor=gold, label=\"{}\"];",
        output_path.display()
    )?;

    for node in analysis.adj_list.keys() {
        writeln!(f, "  \"{}\";", node)?;
    }

    for (node, links) in &analysis.adj_list {
        for link in links {
            writeln!(f, "  \"{}\" -> \"{}\";", node, link)?;
        }
    }

    writeln!(f, "}}")?;
    Ok(())
}

// --- Binary Bake ---

fn bake_binary(analysis: &GraphAnalysis, output_path: &Path) -> Result<()> {
    println!("Validation complete. Soldering binary...");

    let mut file = fs::File::create(output_path)?;

    // Step A: Reserve header space
    file.write_all(&vec![0u8; HEADER_SIZE as usize])?;

    // Step B: Solder node payloads
    let mut jump_table: HashMap<String, u64> = HashMap::new();

    for entry in &analysis.nodes {
        let id = extract_anchor_id(entry).unwrap_or("unknown").to_string();

        let pos = solder_node(&mut file, entry)?;
        jump_table.insert(id, pos);
    }

    // Step C: Write jump table
    let index_pos = file.stream_position()?;

    let mut sorted: Vec<_> = jump_table.into_iter().collect();
    sorted.sort_by(|a, b| a.0.cmp(&b.0));

    for (id, offset) in &sorted {
        // Tab separator allows anchors like "function:fetch_entry".
        writeln!(file, "{}\t{}", id, offset)?;
    }

    // Step D: Write index pointer into header
    file.seek(SeekFrom::Start(POINTER_START))?;
    file.write_all(&index_pos.to_le_bytes())?;

    println!("SUCCESS: '{}' is soldered.", output_path.display());
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
            // Only follow neighbors that are actual nodes in this graph.
            // External tags/concepts can exist as references without being compiled nodes.
            if adj.contains_key(neighbor) && has_cycle(neighbor, adj, visited, stack) {
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
