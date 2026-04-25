use relay_compiler::{POINTER_START, fetch_entry};
use std::fs::File;
// We must import Read and Seek traits to use methods like read_exact and seek
use std::io::{BufRead, BufReader, Read, Seek, SeekFrom};

/**
 * @bin verify
 * @description The Integrity Inspector for RelayDB.
 * Cross-references the Jump Table against physical binary data to ensure
 * 100% address accuracy and zero data corruption.
 */
fn main() {
    println!("--- RelayDB Integrity Inspector ---");

    let mut file = File::open("bacon_standard.relay").expect("Binary file not found");

    // 1. Locate the Jump Table
    // We jump to the standardized pointer location (byte 16)
    file.seek(SeekFrom::Start(POINTER_START)).unwrap();
    let mut offset_bytes = [0u8; 8];

    // This is where 'use std::io::Read' is required
    file.read_exact(&mut offset_bytes)
        .expect("Failed to read index pointer");
    let index_pos = u64::from_le_bytes(offset_bytes);

    // 2. Load the Index for verification
    file.seek(SeekFrom::Start(index_pos)).unwrap();
    let reader = BufReader::new(file);
    let mut entries_checked = 0;
    let mut failures = 0;

    println!("Checking Jump Table at offset {}...", index_pos);

    for line in reader.lines() {
        let l = line.unwrap();
        // Split the "id:offset" string soldered at the end of the file
        let parts: Vec<&str> = l.split(':').collect();
        let id = parts[0];
        let address: u64 = parts[1].parse().expect("Invalid address in jump table");

        // 3. The Physical Probe: Teleport and verify
        // We use the library function to jump to the byte and pull the JSON
        let soldered_data = fetch_entry(address);

        if soldered_data["#id"] == id {
            println!("✅ Match: #{} found at byte {}", id, address);
        } else {
            println!(
                "❌ CORRUPTION: #{} expected at {}, but found {:?}",
                id, address, soldered_data["#id"]
            );
            failures += 1;
        }
        entries_checked += 1;
    }

    println!("\n--- Verification Report ---");
    println!("Total Entries Scanned: {}", entries_checked);
    if failures == 0 {
        println!("RESULT: All soldered addresses are physically sound. 🚀");
    } else {
        println!(
            "RESULT: {} integrity failures detected. Re-bake required.",
            failures
        );
    }
}
