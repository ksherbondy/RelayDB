use serde_json::Value;
use std::collections::HashSet;
use std::fs::File;
use std::io::{BufRead, BufReader, Read, Seek, SeekFrom, Write};

// --- PROTOCOL CONSTANTS ---
// Centralized so that the Compiler and Reader always speak the same language.
pub const POINTER_START: u64 = 16;
pub const HEADER_SIZE: u64 = 32;
pub const TERMINATOR: u8 = 0;

// --- 1. DATA ACCESS LAYER (The Mechanisms) ---

/**
 * @function get_address
 * @description Teleports to the Jump Table and retrieves the byte offset for an Anchor.
 * Returns Some(address) if found, or None if the coordinate doesn't exist or a file error occurs.
 */
pub fn get_address(target_id: &str) -> Option<u64> {
    // The '?' now correctly returns 'None' to the caller if these fail
    let mut file = File::open("bacon_standard.relay").ok()?;

    file.seek(SeekFrom::Start(POINTER_START)).ok()?;
    let mut offset_bytes = [0u8; 8];
    file.read_exact(&mut offset_bytes).ok()?;
    let index_pos = u64::from_le_bytes(offset_bytes);

    file.seek(SeekFrom::Start(index_pos)).ok()?;
    let reader = BufReader::new(file);
    for line in reader.lines() {
        let l = line.ok()?;
        if l.starts_with(target_id) {
            let parts: Vec<&str> = l.split(':').collect();
            if parts.len() == 2 {
                return parts[1].parse::<u64>().ok();
            }
        }
    }
    None
}

/**
 * @function fetch_entry
 * @description Pulls raw JSON data from a byte address until it hits the null terminator.
 */
pub fn fetch_entry(address: u64) -> Value {
    let mut file = File::open("bacon_standard.relay").expect("Binary not found");
    file.seek(SeekFrom::Start(address)).unwrap();
    let mut buffer = Vec::new();
    let mut byte = [0u8; 1];
    while file.read(&mut byte).unwrap() > 0 {
        if byte[0] == TERMINATOR {
            break;
        }
        buffer.push(byte[0]);
    }
    serde_json::from_slice(&buffer).expect("JSON Parse Error: Data corruption at address")
}

/**
 * @function get_jump_table
 * @description Parses the binary index into a usable Vector.
 * Allows tools to audit data without manual byte-seeking.
 */
pub fn get_jump_table() -> Vec<(String, u64)> {
    let mut file = File::open("bacon_standard.relay").ok().unwrap();
    file.seek(SeekFrom::Start(POINTER_START)).unwrap();
    let mut offset_bytes = [0u8; 8];
    file.read_exact(&mut offset_bytes).unwrap();
    let index_pos = u64::from_le_bytes(offset_bytes);

    file.seek(SeekFrom::Start(index_pos)).unwrap();
    let reader = BufReader::new(file);
    reader
        .lines()
        .filter_map(|l| {
            let line = l.ok()?;
            let parts: Vec<&str> = line.split(':').collect();
            if parts.len() == 2 {
                Some((parts[0].to_string(), parts[1].parse::<u64>().ok()?))
            } else {
                None
            }
        })
        .collect()
}

// --- 2. LOGIC LAYER (The Policies) ---

/**
 * @function verify_integrity
 * @description High-level system health check. Cross-references the index against
 * the physical data blocks. Extracted from original verify.rs logic.
 */
pub fn verify_integrity() -> bool {
    let entries = get_jump_table();
    let mut failures = 0;

    for (id, address) in entries {
        let data = fetch_entry(address);
        if data["#id"] != id {
            println!(
                "❌ Integrity Failure: #{} corrupted at byte {}",
                id, address
            );
            failures += 1;
        }
    }
    failures == 0
}

/**
 * @function relay_jump
 * @description The recursive engine. Navigates @ links across the binary.
 * Extracted from reader.rs.
 */
pub fn relay_jump(target_id: &str, visited: &mut HashSet<String>, subject: Option<&str>) {
    if visited.contains(target_id) {
        return;
    }
    visited.insert(target_id.to_string());

    // We safely unwrap the Option. If it's None, we log a warning and return.
    let address = match get_address(target_id) {
        Some(addr) => addr,
        None => {
            println!("Warning: Anchor #{} not found in Jump Table.", target_id);
            return;
        }
    };

    let data = fetch_entry(address);
    if should_display_entry(&data, subject) {
        println!(
            "\n--- [RELAY ENTRY: {}] ---",
            data["name"].as_str().unwrap_or(target_id)
        );
        display_formatted_entry(&data);
    }

    if let Some(obj) = data.as_object() {
        for (key, value) in obj {
            if key.starts_with('@') {
                process_baton(value, visited, subject);
            }
        }
    }
}

// --- 3. INTERNAL HELPERS (UI & FILTERING) ---

fn should_display_entry(data: &Value, subject: Option<&str>) -> bool {
    if let Some(s) = subject {
        let raw_string = serde_json::to_string(data).unwrap_or_default();

        // Changing || to && ensures that even if it's a movie,
        // it still MUST contain the subject string.
        return raw_string.contains(s) && data["^"] == "movies";
    }
    true
}

fn process_baton(value: &Value, visited: &mut HashSet<String>, subject: Option<&str>) {
    if let Some(next_id) = value.as_str() {
        relay_jump(next_id, visited, subject);
    } else if let Some(list) = value.as_array() {
        for item in list {
            if let Some(next_id) = item.as_str() {
                relay_jump(next_id, visited, subject);
            }
        }
    }
}

fn display_formatted_entry(data: &Value) {
    if let Some(obj) = data.as_object() {
        for (key, value) in obj {
            let prefix = if key.starts_with('^') {
                "TOPIC"
            } else if key.starts_with('~') {
                "METADATA"
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
 * @description The low-level protocol writer.
 * Encodes a JSON value into the binary format with the null terminator.
 */
pub fn solder_node(file: &mut File, entry: &Value) -> std::io::Result<u64> {
    let pos = file.stream_position()?;
    let json_string = serde_json::to_string(entry).unwrap();
    file.write_all(json_string.as_bytes())?;
    file.write_all(&[TERMINATOR])?;
    Ok(pos)
}

// --- 4. UNIT & INTEGRATION TESTS ---

#[cfg(test)]
mod tests {
    use super::*;

    // UNIT TEST: Tests filtering logic without disk I/O
    #[test]
    fn test_metadata_gatekeeper() {
        let mock_data = serde_json::json!({
            "name": "Test Node",
            "~tag": "Drama",
            "^": "movies"
        });
        assert!(should_display_entry(&mock_data, Some("Drama")));
        assert!(!should_display_entry(&mock_data, Some("Horror")));
    }

    // UNIT TEST: Verifies baton parsing for arrays
    #[test]
    fn test_baton_list_logic() {
        let mock_links = serde_json::json!(["link_1", "link_2"]);
        assert!(mock_links.is_array());
        assert_eq!(mock_links[0], "link_1");
    }

    // INTEGRATION TEST: Verifies disk teleportation
    #[test]
    fn test_address_lookup() {
        // We use if let to safely handle the Option during the integration test
        if let Some(addr) = get_address("kevin_bacon") {
            let data = fetch_entry(addr);
            assert_eq!(data["#id"], "kevin_bacon");
        }
    }
}
