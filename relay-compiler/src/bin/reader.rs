use std::collections::HashSet;
// We pull the "Engine" from our library
use relay_compiler::relay_jump;

fn main() {
    println!("--- RelayDB: The Bacon Standard (Filtered Relay) ---");

    // We still need a 'visited' set to pass into the library
    // so it can track our path through the graph.
    let mut visited = HashSet::new();

    // The current intent (The Subject Tag)
    let subject_filter = Some("Drama");

    // We call the library function. It will:
    // 1. Teleport to #the_terminal
    // 2. Follow @batons to actors
    // 3. Filter by ~Drama
    relay_jump("the_terminal", &mut visited, subject_filter);
}
