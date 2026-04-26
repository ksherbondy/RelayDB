use relay_compiler::relay_jump;
use std::collections::HashSet;

/**
 * @bin reader
 * @description The targeted entry point for RelayDB.
 * This tool demonstrates the "Bacon Standard" by jumping to a specific
 * anchor and filtering the subsequent graph traversal.
 */
fn main() {
    println!("--- RelayDB: The Bacon Standard (Filtered Relay) ---");

    // The 'visited' set is our safety net against infinite loops (cycles),
    // ensuring the engine stays "Unbreakable" during navigation.
    let mut visited = HashSet::new();

    // The current intent: We only want to see data related to "Drama".
    // This is passed to the library's 'should_display_entry' policy.
    let subject_filter = Some("Drama");

    // The Engine Execution:
    // 1. Physical Jump: Teleports to the #the_terminal byte address.
    // 2. Traversal: Follows all @ (Baton) links recursively.
    // 3. Gatekeeping: Only displays entries matching our subject_filter.
    relay_jump("the_terminal", &mut visited, subject_filter);

    println!(
        "\n--- Navigation Complete: {} nodes visited ---",
        visited.len()
    );
}
