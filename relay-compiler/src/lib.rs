use serde_json::Value;
use std::collections::HashSet;
use std::fs::File;
use std::io::{BufRead, BufReader, Read, Seek, SeekFrom, Write};
use std::path::Path;

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

// --- 6. UNIT TESTS ---

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
}
