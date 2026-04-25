use serde_json::Value;
use std::collections::HashMap;
use std::fs;
use std::io::{Seek, SeekFrom, Write};
// We import the constants from our library to ensure the header matches
use relay_compiler::POINTER_START;

fn main() {
    println!("--- TOOL A: THE COMPILER (Building the Library) ---");
    let mut file = fs::File::create("bacon_standard.relay").expect("Failed to create file");

    // Reserve 32 bytes for the Header
    file.write_all(&[0u8; 32]).unwrap();

    let mut jump_table: HashMap<String, u64> = HashMap::new();
    let paths = fs::read_dir("../data").expect("Missing 'data' folder");

    for path in paths {
        let path = path.unwrap().path();
        if path.extension().and_then(|s| s.to_str()) == Some("json") {
            let content = fs::read_to_string(&path).unwrap();
            let entries: Vec<Value> = serde_json::from_str(&content).unwrap();

            for entry in entries {
                let id = entry["#id"].as_str().unwrap().to_string();
                let pos = file.stream_position().unwrap();

                jump_table.insert(id.clone(), pos);
                println!("Soldering #{} at byte {}", id, pos);

                let json_string = serde_json::to_string(&entry).unwrap();
                file.write_all(json_string.as_bytes()).unwrap();
                // The Null Terminator wall
                file.write_all(b"\0").unwrap();
            }
        }
    }

    // Append the Jump Table to the end
    let index_pos = file.stream_position().unwrap();
    for (id, offset) in &jump_table {
        writeln!(file, "{}:{}", id, offset).unwrap();
    }

    // Solder the Index location into the Header
    file.seek(SeekFrom::Start(POINTER_START)).unwrap();
    file.write_all(&index_pos.to_le_bytes()).unwrap();

    println!("SUCCESS: 'bacon_standard.relay' is ready.");
}
