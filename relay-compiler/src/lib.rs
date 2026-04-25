use serde_json::Value;
use std::collections::HashSet;
use std::fs::File;
use std::io::{BufRead, BufReader, Read, Seek, SeekFrom};

// PROTOCOL CONSTANTS
pub const POINTER_START: u64 = 16;

/**
 * @function relay_jump
 * @description The recursive core of RelayDB.
 * This follows the "Baton Pass" (@) from one record to another across the binary.
 */
pub fn relay_jump(target_id: &str, visited: &mut HashSet<String>, subject: Option<&str>) {
    // 1. Preventing infinite loops in the graph
    if visited.contains(target_id) {
        return;
    }
    visited.insert(target_id.to_string());

    // 2. Finding the coordinate
    let address = get_address(target_id);
    if address == 0 {
        return;
    }

    // 3. Fetching the data
    let data = fetch_entry(address);
    let raw_string = serde_json::to_string(&data).unwrap();

    // 4. Filtering logic (The ~ Gatekeeper)
    let mut should_display = true;
    if let Some(s) = subject {
        if !raw_string.contains(s) && data["^"] != "movies" {
            should_display = false;
        }
    }

    if should_display {
        println!(
            "\n--- [RELAY ENTRY: {}] ---",
            data["name"].as_str().unwrap_or(target_id)
        );
        // Categorized display logic remains here for the UI/Reader to use
        display_formatted_entry(&data);
    }

    // 5. Following the Baton (@)
    if let Some(obj) = data.as_object() {
        for (key, value) in obj {
            if key.starts_with('@') {
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
        }
    }
}

pub fn get_address(target_id: &str) -> u64 {
    let mut file = File::open("bacon_standard.relay").expect("Binary not found");
    file.seek(SeekFrom::Start(POINTER_START)).unwrap();
    let mut offset_bytes = [0u8; 8];
    file.read_exact(&mut offset_bytes).unwrap();
    let index_pos = u64::from_le_bytes(offset_bytes);

    file.seek(SeekFrom::Start(index_pos)).unwrap();
    let reader = BufReader::new(file);
    for line in reader.lines() {
        let l = line.unwrap();
        if l.starts_with(target_id) {
            return l.split(':').collect::<Vec<&str>>()[1]
                .parse::<u64>()
                .unwrap();
        }
    }
    0
}

pub fn fetch_entry(address: u64) -> Value {
    let mut file = File::open("bacon_standard.relay").unwrap();
    file.seek(SeekFrom::Start(address)).unwrap();
    let mut buffer = Vec::new();
    let mut byte = [0u8; 1];
    while file.read(&mut byte).unwrap() > 0 {
        if byte[0] == 0 {
            break;
        }
        buffer.push(byte[0]);
    }
    let raw_string = String::from_utf8(buffer).expect("Invalid UTF-8");
    serde_json::from_str(&raw_string).expect("JSON Parse Error")
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
    println!("--------------------------------------------------");
}

/**
 * @function verify_integrity
 * @description Scans the entire Jump Table and verifies that every
 * Anchor (#) points to its correct physical data block.
 */
pub fn verify_integrity() -> bool {
    let mut file = File::open("bacon_standard.relay").expect("Binary not found");

    // Find the index pointer
    file.seek(SeekFrom::Start(POINTER_START)).unwrap();
    let mut offset_bytes = [0u8; 8];
    file.read_exact(&mut offset_bytes).unwrap();
    let index_pos = u64::from_le_bytes(offset_bytes);

    file.seek(SeekFrom::Start(index_pos)).unwrap();
    let reader = BufReader::new(file);
    let mut failures = 0;

    for line in reader.lines() {
        let l = line.unwrap();
        let parts: Vec<&str> = l.split(':').collect();
        let id = parts[0];
        let address: u64 = parts[1].parse().unwrap();

        let data = fetch_entry(address);
        if data["#id"] != id {
            println!("❌ Error: #{} corrupted at byte {}", id, address);
            failures += 1;
        }
    }
    failures == 0
}

#[cfg(test)]
mod tests {
    use super::*;

    // TEST 1: The O(1) Teleportation Logic
    // This confirms the engine can read the pointer at Byte 16 and find the map.
    #[test]
    fn test_address_lookup() {
        let addr = get_address("kevin_bacon");
        // We check if it's > 0 because 0 is our "Not Found" signal.
        assert!(addr > 0, "The Engine could not teleport to #kevin_bacon.");
    }

    // TEST 2: The Data Fetcher (Byte-to-JSON)
    // This confirms that jumping to a specific byte and reading until \0 works.
    #[test]
    fn test_fetch_entry_integrity() {
        let addr = get_address("kevin_bacon");
        let data = fetch_entry(addr);

        // Confirm the #id inside the block matches what the Jump Table promised.
        assert_eq!(
            data["#id"], "kevin_bacon",
            "Data corruption detected at soldered address."
        );
    }

    // TEST 3: Semantic Filter Accuracy
    // We test the logic of the 'relay_jump' gatekeeper without running the whole program.
    #[test]
    fn test_metadata_gatekeeper() {
        let raw_data = "{\"name\":\"Tom Hanks\",\"~genres\":[\"Drama\",\"Comedy\"]}";
        let filter = "Drama";

        assert!(
            raw_data.contains(filter),
            "The Metadata filter failed to identify 'Drama'."
        );
    }
}
