use relay_compiler::verify_integrity_from;

/**
 * @bin verify
 * @description Integrity Inspector for RelayDB.
 * Usage:
 * cargo run --bin verify -- output.relay
 */
fn main() {
    let relay_file = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "output.relay".to_string());

    println!(
        "--- RelayDB: Running Protocol Integrity Check on {} ---",
        relay_file
    );

    if verify_integrity_from(&relay_file) {
        std::process::exit(0);
    } else {
        std::process::exit(1);
    }
}
