use relay_compiler::verify_integrity;

/**
 * @bin verify
 * @description The Integrity Inspector for RelayDB.
 * This tool is now a clean orchestrator. It calls the library's central
 * integrity logic to ensure the binary is physically sound.
 */
fn main() {
    println!("--- RelayDB: Running Protocol Integrity Check ---");

    // We delegate the byte-seeking, parsing, and comparison to lib.rs.
    // This ensures that the Inspector always uses the exact same rules
    // as the Compiler and the Reader.
    if verify_integrity() {
        // Success: The library prints the "All sound" message,
        // we just handle the OS exit code.
        std::process::exit(0);
    } else {
        // Failure: The library will have logged the specific corrupted bytes.
        std::process::exit(1);
    }
}
