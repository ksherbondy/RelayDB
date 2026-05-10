use relay_compiler::relay_jump_from;
use std::collections::HashSet;

/**
 * @bin reader
 * @description Simple demo reader for RelayDB.
 * For the universal workflow, prefer:
 * cargo run --bin relay -- jump --file output.relay project:relaydb
 */
fn main() {
    println!("--- RelayDB: Demo Reader ---");

    let mut visited = HashSet::new();

    relay_jump_from("output.relay", "project:relaydb", &mut visited, None);

    println!(
        "\n--- Navigation Complete: {} nodes visited ---",
        visited.len()
    );
}
