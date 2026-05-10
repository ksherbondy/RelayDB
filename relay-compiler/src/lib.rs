use serde_json::Value;
use std::collections::{HashMap, HashSet};
use std::fs::{self, File};
use std::io::{BufRead, BufReader, Read, Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};

// --- PROTOCOL CONSTANTS ---
// Centralized so the Compiler and Reader always speak the same language.
pub const POINTER_START: u64 = 16;
pub const HEADER_SIZE: u64 = 32;
pub const TERMINATOR: u8 = 0;

// Backwards-compatible default used by the older demo tools.
pub const DEFAULT_RELAY_FILE: &str = "bacon_standard.relay";

// --- 1. SHARED TAG / RELATIONSHIP HELPERS ---

/**
 * @function extract_anchor_id
 * @description Returns the stable anchor ID for a JSON node.
 * Supports both RelayDB v1 style "#id" and ATLAS-style "#".
 */
pub fn extract_anchor_id(node: &Value) -> Option<&str> {
    node.get("#id")
        .and_then(Value::as_str)
        .or_else(|| node.get("#").and_then(Value::as_str))
}

/**
 * @function extract_links_from_node
 * @description Shared relationship extractor for compiler and runtime rules.
 * Captures string and array-valued links from @ and ^ tagged fields.
 */
pub fn extract_links_from_node(node: &Value) -> Vec<String> {
    let mut links = Vec::new();

    if let Some(obj) = node.as_object() {
        for (key, value) in obj {
            if key.starts_with('@') || key.starts_with('^') {
                extract_links_from_value(value, &mut links);
            }
        }
    }

    links
}

fn extract_links_from_value(value: &Value, links: &mut Vec<String>) {
    if let Some(link) = value.as_str() {
        links.push(link.to_string());
    } else if let Some(list) = value.as_array() {
        for item in list {
            if let Some(link) = item.as_str() {
                links.push(link.to_string());
            }
        }
    }
}

fn parse_jump_table_line(line: &str) -> Option<(String, u64)> {
    // New universal format uses a tab so anchors may safely contain ':'.
    if let Some((id, offset)) = line.split_once('\t') {
        return Some((id.to_string(), offset.parse::<u64>().ok()?));
    }

    // Backward-compatible fallback for older "id:offset" jump tables.
    let (id, offset) = line.rsplit_once(':')?;
    Some((id.to_string(), offset.parse::<u64>().ok()?))
}

// --- 2. DATA ACCESS LAYER ---

/**
 * @function get_address_from
 * @description Reads a .relay file's jump table and returns the byte offset for an Anchor.
 */
pub fn get_address_from<P: AsRef<Path>>(relay_path: P, target_id: &str) -> Option<u64> {
    let mut file = File::open(relay_path).ok()?;

    file.seek(SeekFrom::Start(POINTER_START)).ok()?;
    let mut offset_bytes = [0u8; 8];
    file.read_exact(&mut offset_bytes).ok()?;
    let index_pos = u64::from_le_bytes(offset_bytes);

    file.seek(SeekFrom::Start(index_pos)).ok()?;
    let reader = BufReader::new(file);

    for line in reader.lines() {
        let line = line.ok()?;
        let (id, offset) = parse_jump_table_line(&line)?;
        if id == target_id {
            return Some(offset);
        }
    }

    None
}

/**
 * @function fetch_entry_from
 * @description Pulls raw JSON data from a byte address until it hits the null terminator.
 */
pub fn fetch_entry_from<P: AsRef<Path>>(relay_path: P, address: u64) -> std::io::Result<Value> {
    let mut file = File::open(relay_path)?;
    file.seek(SeekFrom::Start(address))?;

    let mut buffer = Vec::new();
    let mut byte = [0u8; 1];

    while file.read(&mut byte)? > 0 {
        if byte[0] == TERMINATOR {
            break;
        }
        buffer.push(byte[0]);
    }

    serde_json::from_slice(&buffer)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))
}

/**
 * @function get_jump_table_from
 * @description Parses a .relay file's binary index into a usable vector.
 */
pub fn get_jump_table_from<P: AsRef<Path>>(relay_path: P) -> std::io::Result<Vec<(String, u64)>> {
    let mut file = File::open(relay_path)?;
    file.seek(SeekFrom::Start(POINTER_START))?;

    let mut offset_bytes = [0u8; 8];
    file.read_exact(&mut offset_bytes)?;
    let index_pos = u64::from_le_bytes(offset_bytes);

    file.seek(SeekFrom::Start(index_pos))?;
    let reader = BufReader::new(file);

    Ok(reader
        .lines()
        .filter_map(|line| parse_jump_table_line(&line.ok()?))
        .collect())
}

/**
 * @function verify_integrity_from
 * @description Cross-references the index against physical data blocks.
 */
pub fn verify_integrity_from<P: AsRef<Path>>(relay_path: P) -> bool {
    let relay_path = relay_path.as_ref();
    let entries = match get_jump_table_from(relay_path) {
        Ok(entries) => entries,
        Err(e) => {
            eprintln!("Could not read jump table: {}", e);
            return false;
        }
    };

    let mut failures = 0;

    for (id, address) in entries {
        match fetch_entry_from(relay_path, address) {
            Ok(data) => {
                if extract_anchor_id(&data) != Some(id.as_str()) {
                    println!(
                        "❌ Integrity Failure: #{} corrupted at byte {}",
                        id, address
                    );
                    failures += 1;
                }
            }
            Err(e) => {
                println!("❌ Read Failure: #{} at byte {}: {}", id, address, e);
                failures += 1;
            }
        }
    }

    failures == 0
}

// --- 3. BACKWARDS-COMPATIBLE DEFAULT WRAPPERS ---

pub fn get_address(target_id: &str) -> Option<u64> {
    get_address_from(DEFAULT_RELAY_FILE, target_id)
}

pub fn fetch_entry(address: u64) -> Value {
    fetch_entry_from(DEFAULT_RELAY_FILE, address).expect("JSON Parse Error or missing binary")
}

pub fn get_jump_table() -> Vec<(String, u64)> {
    get_jump_table_from(DEFAULT_RELAY_FILE).unwrap_or_default()
}

pub fn verify_integrity() -> bool {
    verify_integrity_from(DEFAULT_RELAY_FILE)
}

// --- 4. LOGIC LAYER ---

/**
 * @function relay_jump_from
 * @description Recursive traversal engine for a chosen .relay file.
 */
pub fn relay_jump_from<P: AsRef<Path>>(
    relay_path: P,
    target_id: &str,
    visited: &mut HashSet<String>,
    subject: Option<&str>,
) {
    let relay_path = relay_path.as_ref();

    if visited.contains(target_id) {
        return;
    }
    visited.insert(target_id.to_string());

    let address = match get_address_from(relay_path, target_id) {
        Some(addr) => addr,
        None => {
            println!("Warning: Anchor #{} not found in Jump Table.", target_id);
            return;
        }
    };

    let data = match fetch_entry_from(relay_path, address) {
        Ok(data) => data,
        Err(e) => {
            println!(
                "Warning: Could not fetch #{} at byte {}: {}",
                target_id, address, e
            );
            return;
        }
    };

    if should_display_entry(&data, subject) {
        println!(
            "\n--- [RELAY ENTRY: {}] ---",
            data.get("name")
                .or_else(|| data.get("title"))
                .and_then(Value::as_str)
                .unwrap_or(target_id)
        );
        display_formatted_entry(&data);
    }

    if let Some(obj) = data.as_object() {
        for (key, value) in obj {
            if key.starts_with('@') {
                process_baton(relay_path, value, visited, subject);
            }
        }
    }
}

pub fn relay_jump(target_id: &str, visited: &mut HashSet<String>, subject: Option<&str>) {
    relay_jump_from(DEFAULT_RELAY_FILE, target_id, visited, subject)
}

// --- 5. INTERNAL HELPERS ---

fn should_display_entry(data: &Value, subject: Option<&str>) -> bool {
    if let Some(s) = subject {
        let raw_string = serde_json::to_string(data).unwrap_or_default();
        return raw_string.contains(s);
    }

    true
}

fn process_baton<P: AsRef<Path>>(
    relay_path: P,
    value: &Value,
    visited: &mut HashSet<String>,
    subject: Option<&str>,
) {
    let relay_path = relay_path.as_ref();

    if let Some(next_id) = value.as_str() {
        relay_jump_from(relay_path, next_id, visited, subject);
    } else if let Some(list) = value.as_array() {
        for item in list {
            if let Some(next_id) = item.as_str() {
                relay_jump_from(relay_path, next_id, visited, subject);
            }
        }
    }
}

fn display_formatted_entry(data: &Value) {
    if let Some(obj) = data.as_object() {
        for (key, value) in obj {
            let prefix = if key.starts_with('#') {
                "ANCHOR"
            } else if key.starts_with('^') {
                "PROVENANCE"
            } else if key.starts_with('~') {
                "ALIAS"
            } else if key.starts_with('@') {
                "RELAY-LINK"
            } else {
                "DATA"
            };
            println!("{}: {} => {:?}", prefix, key, value);
        }
    }
}

/**
 * @function solder_node
 * @description Encodes a JSON value into the binary format with the null terminator.
 */
pub fn solder_node(file: &mut File, entry: &Value) -> std::io::Result<u64> {
    let pos = file.stream_position()?;
    let json_string = serde_json::to_string(entry).unwrap();
    file.write_all(json_string.as_bytes())?;
    file.write_all(&[TERMINATOR])?;
    Ok(pos)
}

// --- 6. SOURCE MEMORY AUDIT LAYER ---

#[derive(Debug, Clone)]
struct SourceNode {
    file: PathBuf,
    line: usize,
    anchor: Option<String>,
    value: Value,
}

#[derive(Debug, Clone)]
struct SourceRef {
    source: String,
    tag: String,
    target: String,
    file: PathBuf,
    line: usize,
}

#[derive(Debug, Clone)]
struct MemoryAuditReport {
    input: PathBuf,
    nodes: Vec<SourceNode>,
    parse_errors: Vec<String>,
    refs: Vec<SourceRef>,
    duplicate_anchors: HashMap<String, Vec<(PathBuf, usize)>>,
    missing_refs: Vec<SourceRef>,
    external_refs: Vec<SourceRef>,
    orphans: Vec<String>,
    cycles: Vec<Vec<String>>,
}

/**
 * @function audit_memory_from
 * @description Audits JSON/JSONL source memory before it is compiled into .relay.
 */
pub fn audit_memory_from<P: AsRef<Path>>(input_path: P, mode: &str) -> bool {
    let input_path = input_path.as_ref();
    let report = build_memory_audit_report(input_path);

    match mode {
        "validate" => print_validate_report(&report),
        "summary" => {
            print_memory_summary(&report);
            true
        }
        "duplicates" => print_duplicates_report(&report),
        "missing" => print_missing_report(&report),
        "orphans" => {
            print_orphans_report(&report);
            true
        }
        "external" => {
            print_external_report(&report);
            true
        }
        "cycles" => {
            print_cycles_report(&report);
            true
        }
        "all" => print_full_memory_audit(&report),
        unknown => {
            eprintln!("Unknown audit-memory mode: {}", unknown);
            eprintln!(
                "Valid modes: validate, summary, duplicates, missing, orphans, external, cycles, all"
            );
            false
        }
    }
}

fn build_memory_audit_report(input_path: &Path) -> MemoryAuditReport {
    let mut nodes = Vec::new();
    let mut parse_errors = Vec::new();

    let files = match collect_source_memory_files(input_path) {
        Ok(files) => files,
        Err(e) => {
            parse_errors.push(e);
            Vec::new()
        }
    };

    for file in files {
        match read_source_nodes_from_file(&file) {
            Ok(mut file_nodes) => nodes.append(&mut file_nodes),
            Err(e) => parse_errors.push(e),
        }
    }

    let mut anchor_locations: HashMap<String, Vec<(PathBuf, usize)>> = HashMap::new();

    for node in &nodes {
        if let Some(anchor) = &node.anchor {
            anchor_locations
                .entry(anchor.clone())
                .or_default()
                .push((node.file.clone(), node.line));
        }
    }

    let duplicate_anchors = anchor_locations
        .iter()
        .filter_map(|(anchor, locations)| {
            if locations.len() > 1 {
                Some((anchor.clone(), locations.clone()))
            } else {
                None
            }
        })
        .collect::<HashMap<_, _>>();

    let anchor_set = anchor_locations.keys().cloned().collect::<HashSet<_>>();
    let refs = extract_source_refs(&nodes);

    let missing_refs = refs
        .iter()
        .filter(|reference| {
            !reference.target.starts_with("external:") && !anchor_set.contains(&reference.target)
        })
        .cloned()
        .collect::<Vec<_>>();

    let external_refs = refs
        .iter()
        .filter(|reference| reference.target.starts_with("external:"))
        .cloned()
        .collect::<Vec<_>>();

    let mut incoming_counts: HashMap<String, usize> = HashMap::new();

    for reference in &refs {
        if anchor_set.contains(&reference.target) {
            *incoming_counts.entry(reference.target.clone()).or_insert(0) += 1;
        }
    }

    let mut orphans = anchor_set
        .iter()
        .filter(|anchor| !anchor.starts_with("project:"))
        .filter(|anchor| incoming_counts.get(*anchor).copied().unwrap_or(0) == 0)
        .cloned()
        .collect::<Vec<_>>();
    orphans.sort();

    let cycles = detect_source_cycles(&anchor_set, &refs);

    MemoryAuditReport {
        input: input_path.to_path_buf(),
        nodes,
        parse_errors,
        refs,
        duplicate_anchors,
        missing_refs,
        external_refs,
        orphans,
        cycles,
    }
}

fn collect_source_memory_files(input_path: &Path) -> Result<Vec<PathBuf>, String> {
    if input_path.is_file() {
        if is_supported_source_memory_file(input_path) {
            return Ok(vec![input_path.to_path_buf()]);
        }

        return Err(format!(
            "{} is not a supported .json or .jsonl source memory file",
            input_path.display()
        ));
    }

    if input_path.is_dir() {
        let mut files = Vec::new();
        collect_source_memory_files_recursive(input_path, &mut files)?;
        files.sort();
        return Ok(files);
    }

    Err(format!("Input path not found: {}", input_path.display()))
}

fn collect_source_memory_files_recursive(
    dir: &Path,
    files: &mut Vec<PathBuf>,
) -> Result<(), String> {
    let entries = fs::read_dir(dir)
        .map_err(|e| format!("Could not read directory {}: {}", dir.display(), e))?;

    for entry in entries {
        let entry = entry.map_err(|e| format!("Could not read directory entry: {}", e))?;
        let path = entry.path();

        if path.is_dir() {
            collect_source_memory_files_recursive(&path, files)?;
        } else if path.is_file() && is_supported_source_memory_file(&path) {
            files.push(path);
        }
    }

    Ok(())
}

fn is_supported_source_memory_file(path: &Path) -> bool {
    matches!(
        path.extension().and_then(|s| s.to_str()),
        Some("json") | Some("jsonl")
    )
}

fn read_source_nodes_from_file(path: &Path) -> Result<Vec<SourceNode>, String> {
    match path.extension().and_then(|s| s.to_str()) {
        Some("json") => read_json_source_nodes(path),
        Some("jsonl") => read_jsonl_source_nodes(path),
        _ => Ok(Vec::new()),
    }
}

fn read_json_source_nodes(path: &Path) -> Result<Vec<SourceNode>, String> {
    let content = fs::read_to_string(path)
        .map_err(|e| format!("Could not read {}: {}", path.display(), e))?;

    let parsed: Value = serde_json::from_str(&content)
        .map_err(|e| format!("{} is not valid JSON: {}", path.display(), e))?;

    if let Some(items) = parsed.as_array() {
        let mut nodes = Vec::new();

        for (index, item) in items.iter().enumerate() {
            if !item.is_object() {
                return Err(format!(
                    "{} item {} is not a JSON object",
                    path.display(),
                    index + 1
                ));
            }

            nodes.push(SourceNode {
                file: path.to_path_buf(),
                line: index + 1,
                anchor: extract_anchor_id(item).map(str::to_string),
                value: item.clone(),
            });
        }

        return Ok(nodes);
    }

    if parsed.is_object() {
        return Ok(vec![SourceNode {
            file: path.to_path_buf(),
            line: 1,
            anchor: extract_anchor_id(&parsed).map(str::to_string),
            value: parsed,
        }]);
    }

    Err(format!(
        "{} must contain a JSON object or an array of JSON objects",
        path.display()
    ))
}

fn read_jsonl_source_nodes(path: &Path) -> Result<Vec<SourceNode>, String> {
    let content = fs::read_to_string(path)
        .map_err(|e| format!("Could not read {}: {}", path.display(), e))?;

    let mut nodes = Vec::new();

    for (line_index, line) in content.lines().enumerate() {
        let line_number = line_index + 1;
        let trimmed = line.trim();

        if trimmed.is_empty() {
            continue;
        }

        let parsed: Value = serde_json::from_str(trimmed).map_err(|e| {
            format!(
                "{} line {} is not valid JSONL: {}",
                path.display(),
                line_number,
                e
            )
        })?;

        if !parsed.is_object() {
            return Err(format!(
                "{} line {} must be a JSON object",
                path.display(),
                line_number
            ));
        }

        nodes.push(SourceNode {
            file: path.to_path_buf(),
            line: line_number,
            anchor: extract_anchor_id(&parsed).map(str::to_string),
            value: parsed,
        });
    }

    Ok(nodes)
}

fn extract_source_refs(nodes: &[SourceNode]) -> Vec<SourceRef> {
    let mut refs = Vec::new();

    for node in nodes {
        let source = node
            .anchor
            .clone()
            .unwrap_or_else(|| format!("{}:{}", node.file.display(), node.line));

        if let Some(obj) = node.value.as_object() {
            for (key, value) in obj {
                if key.starts_with('@') || key.starts_with('^') {
                    let mut links = Vec::new();
                    extract_links_from_value(value, &mut links);

                    for target in links {
                        refs.push(SourceRef {
                            source: source.clone(),
                            tag: key.clone(),
                            target,
                            file: node.file.clone(),
                            line: node.line,
                        });
                    }
                }
            }
        }
    }

    refs
}

fn detect_source_cycles(anchor_set: &HashSet<String>, refs: &[SourceRef]) -> Vec<Vec<String>> {
    let mut graph: HashMap<String, Vec<String>> = HashMap::new();

    for reference in refs {
        if anchor_set.contains(&reference.source) && anchor_set.contains(&reference.target) {
            graph
                .entry(reference.source.clone())
                .or_default()
                .push(reference.target.clone());
        }
    }

    let mut visited = HashSet::new();
    let mut stack = HashSet::new();
    let mut path = Vec::new();
    let mut cycles = Vec::new();

    let mut anchors = anchor_set.iter().cloned().collect::<Vec<_>>();
    anchors.sort();

    for anchor in anchors {
        if !visited.contains(&anchor) {
            detect_source_cycles_dfs(
                &anchor,
                &graph,
                &mut visited,
                &mut stack,
                &mut path,
                &mut cycles,
            );
        }
    }

    cycles
}

fn detect_source_cycles_dfs(
    node: &str,
    graph: &HashMap<String, Vec<String>>,
    visited: &mut HashSet<String>,
    stack: &mut HashSet<String>,
    path: &mut Vec<String>,
    cycles: &mut Vec<Vec<String>>,
) {
    if stack.contains(node) {
        if let Some(start_index) = path.iter().position(|item| item == node) {
            let mut cycle = path[start_index..].to_vec();
            cycle.push(node.to_string());
            cycles.push(cycle);
        }
        return;
    }

    if visited.contains(node) {
        return;
    }

    visited.insert(node.to_string());
    stack.insert(node.to_string());
    path.push(node.to_string());

    if let Some(neighbors) = graph.get(node) {
        for neighbor in neighbors {
            detect_source_cycles_dfs(neighbor, graph, visited, stack, path, cycles);
        }
    }

    path.pop();
    stack.remove(node);
}

fn print_validate_report(report: &MemoryAuditReport) -> bool {
    let missing_anchor_records = report
        .nodes
        .iter()
        .filter(|node| node.anchor.is_none())
        .collect::<Vec<_>>();

    if report.parse_errors.is_empty() && missing_anchor_records.is_empty() {
        println!("PASS ✅ JSON/JSONL memory is parseable and all records have anchors.");
        return true;
    }

    for error in &report.parse_errors {
        eprintln!("ERROR: {}", error);
    }

    for node in missing_anchor_records {
        eprintln!(
            "ERROR: record missing # or #id anchor at {}:{}",
            node.file.display(),
            node.line
        );
    }

    false
}

fn print_memory_summary(report: &MemoryAuditReport) {
    let unique_anchors = report
        .nodes
        .iter()
        .filter_map(|node| node.anchor.as_ref())
        .collect::<HashSet<_>>();

    println!("RelayDB Memory Summary");
    println!("----------------------");
    println!("Input: {}", report.input.display());
    println!("Records: {}", report.nodes.len());
    println!(
        "Anchors: {}",
        report
            .nodes
            .iter()
            .filter(|node| node.anchor.is_some())
            .count()
    );
    println!("Unique anchors: {}", unique_anchors.len());
    println!("Relationship refs: {}", report.refs.len());
    println!("Parse errors: {}", report.parse_errors.len());
    println!(
        "Records missing anchors: {}",
        report
            .nodes
            .iter()
            .filter(|node| node.anchor.is_none())
            .count()
    );
    println!("Duplicate anchors: {}", report.duplicate_anchors.len());
    println!("Missing internal refs: {}", report.missing_refs.len());
    println!("External refs: {}", report.external_refs.len());
    println!("Orphan nodes: {}", report.orphans.len());
    println!("Cycles detected: {}", report.cycles.len());
}

fn print_duplicates_report(report: &MemoryAuditReport) -> bool {
    if report.duplicate_anchors.is_empty() {
        println!("PASS ✅ Duplicate anchors: 0");
        return true;
    }

    println!("FAIL ❌ Duplicate anchors found:");

    let mut duplicates = report.duplicate_anchors.iter().collect::<Vec<_>>();
    duplicates.sort_by(|a, b| a.0.cmp(b.0));

    for (anchor, locations) in duplicates {
        println!("- {}", anchor);
        for (file, line) in locations {
            println!("  - {}:{}", file.display(), line);
        }
    }

    false
}

fn print_missing_report(report: &MemoryAuditReport) -> bool {
    if report.missing_refs.is_empty() {
        println!("PASS ✅ Missing internal anchors: 0");
        return true;
    }

    println!("FAIL ❌ Missing internal anchors found:");

    for reference in &report.missing_refs {
        println!(
            "- {} referenced by {} via {} at {}:{}",
            reference.target,
            reference.source,
            reference.tag,
            reference.file.display(),
            reference.line
        );
    }

    false
}

fn print_orphans_report(report: &MemoryAuditReport) {
    if report.orphans.is_empty() {
        println!("PASS ✅ Orphan nodes: 0");
        return;
    }

    println!("WARN ⚠️ Orphan nodes: {}", report.orphans.len());
    for orphan in &report.orphans {
        println!("- {}", orphan);
    }
}

fn print_external_report(report: &MemoryAuditReport) {
    if report.external_refs.is_empty() {
        println!("External refs: 0");
        return;
    }

    println!("External refs: {}", report.external_refs.len());
    for reference in &report.external_refs {
        println!(
            "- {} referenced by {} via {} at {}:{}",
            reference.target,
            reference.source,
            reference.tag,
            reference.file.display(),
            reference.line
        );
    }
}

fn print_cycles_report(report: &MemoryAuditReport) {
    if report.cycles.is_empty() {
        println!("Cycles detected: 0");
        return;
    }

    println!("WARN ⚠️ Cycles detected: {}", report.cycles.len());
    for cycle in report.cycles.iter().take(10) {
        println!("- {}", cycle.join(" -> "));
    }
}

fn print_full_memory_audit(report: &MemoryAuditReport) -> bool {
    println!("RelayDB Memory Audit");
    println!("--------------------");
    print_memory_summary(report);
    println!();

    let validate_ok = print_validate_report(report);
    let duplicates_ok = print_duplicates_report(report);
    let missing_ok = print_missing_report(report);
    print_orphans_report(report);
    print_external_report(report);
    print_cycles_report(report);

    println!();

    if validate_ok && duplicates_ok && missing_ok {
        if report.orphans.is_empty() && report.external_refs.is_empty() && report.cycles.is_empty()
        {
            println!("Status: PASS ✅");
        } else {
            println!("Status: PASS WITH WARNINGS ⚠️");
        }

        true
    } else {
        println!("Status: FAIL ❌");
        false
    }
}

// --- 7. UNIT TESTS ---

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_anchor_id_supports_atlas_hash() {
        let node = serde_json::json!({ "#": "function:test" });
        assert_eq!(extract_anchor_id(&node), Some("function:test"));
    }

    #[test]
    fn test_extract_anchor_id_supports_relay_hash_id() {
        let node = serde_json::json!({ "#id": "kevin_bacon" });
        assert_eq!(extract_anchor_id(&node), Some("kevin_bacon"));
    }

    #[test]
    fn test_extract_links_from_node_supports_strings_and_arrays() {
        let node = serde_json::json!({
            "#": "function:test",
            "^": ["project:relaydb", "module:lib"],
            "@": ["function:a", "function:b"],
            "@single": "function:c",
            "~": ["alias"]
        });

        let links = extract_links_from_node(&node);

        assert!(links.contains(&"project:relaydb".to_string()));
        assert!(links.contains(&"module:lib".to_string()));
        assert!(links.contains(&"function:a".to_string()));
        assert!(links.contains(&"function:b".to_string()));
        assert!(links.contains(&"function:c".to_string()));
    }

    #[test]
    fn test_parse_jump_table_line_supports_tab_separator() {
        let parsed = parse_jump_table_line("function:fetch_entry\t123").unwrap();
        assert_eq!(parsed.0, "function:fetch_entry");
        assert_eq!(parsed.1, 123);
    }

    #[test]
    fn test_parse_jump_table_line_supports_legacy_colon_separator() {
        let parsed = parse_jump_table_line("function:fetch_entry:123").unwrap();
        assert_eq!(parsed.0, "function:fetch_entry");
        assert_eq!(parsed.1, 123);
    }

    #[test]
    fn test_audit_memory_detects_duplicate_anchors() {
        let nodes = vec![
            SourceNode {
                file: PathBuf::from("test.jsonl"),
                line: 1,
                anchor: Some("function:test".to_string()),
                value: serde_json::json!({"#": "function:test"}),
            },
            SourceNode {
                file: PathBuf::from("test.jsonl"),
                line: 2,
                anchor: Some("function:test".to_string()),
                value: serde_json::json!({"#": "function:test"}),
            },
        ];

        let report = {
            let input = PathBuf::from("test.jsonl");
            let mut anchor_locations: HashMap<String, Vec<(PathBuf, usize)>> = HashMap::new();

            for node in &nodes {
                if let Some(anchor) = &node.anchor {
                    anchor_locations
                        .entry(anchor.clone())
                        .or_default()
                        .push((node.file.clone(), node.line));
                }
            }

            let duplicate_anchors = anchor_locations
                .iter()
                .filter_map(|(anchor, locations)| {
                    if locations.len() > 1 {
                        Some((anchor.clone(), locations.clone()))
                    } else {
                        None
                    }
                })
                .collect::<HashMap<_, _>>();

            MemoryAuditReport {
                input,
                nodes,
                parse_errors: Vec::new(),
                refs: Vec::new(),
                duplicate_anchors,
                missing_refs: Vec::new(),
                external_refs: Vec::new(),
                orphans: Vec::new(),
                cycles: Vec::new(),
            }
        };

        assert_eq!(report.duplicate_anchors.len(), 1);
        assert!(report.duplicate_anchors.contains_key("function:test"));
    }
}
