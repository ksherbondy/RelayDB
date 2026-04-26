use clap::{Parser, Subcommand};
use relay_compiler::{relay_jump, verify_integrity};
use std::collections::HashSet;

#[derive(Parser)]
#[command(name = "relay")]
#[command(version = "1.1")]
#[command(about = "The RelayDB 4-Tag Protocol CLI", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Navigate the graph starting at a specific Anchor (#)
    Jump {
        /// The Anchor ID to start from (e.g., #the_terminal)
        anchor: String,
        /// Optional metadata filter (e.g., ~Drama)
        #[arg(short, long)]
        filter: Option<String>,
    },
    /// Verify the physical integrity of the .relay file
    Check,
}

fn main() {
    let cli = Cli::parse();

    match &cli.command {
        Commands::Jump { anchor, filter } => {
            println!("--- RELAY-JUMP: Teleporting to #{} ---", anchor);
            let mut visited = HashSet::new();

            // The library handles the recursive teleportation logic.
            relay_jump(anchor, &mut visited, filter.as_deref());

            println!(
                "\n--- Traversal Complete: {} nodes mapped ---",
                visited.len()
            );
        }
        Commands::Check => {
            println!("--- RELAY-CHECK: Auditing Binary Solder Points ---");

            // Delegate the scan to the library policy.
            if verify_integrity() {
                // Success path
                println!("SUCCESS: System is physically sound and ready for transport. 🚀");
                std::process::exit(0);
            } else {
                // Failure path
                eprintln!("CRITICAL: Data corruption or address mismatch detected.");
                std::process::exit(1);
            }
        }
    }
}
