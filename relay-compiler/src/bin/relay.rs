use clap::{Parser, Subcommand};
use std::collections::HashSet;
// We only need relay_jump and verify_integrity now
use relay_compiler::{relay_jump, verify_integrity};

#[derive(Parser)]
#[command(name = "relay")]
#[command(about = "The RelayDB 4-Tag Protocol CLI", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Navigate the graph starting at a specific Anchor (#)
    Jump {
        /// The Anchor ID to start from (e.g., tom_hanks)
        anchor: String,
        /// Optional metadata filter (e.g., Drama)
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
            let mut visited = HashSet::new();
            relay_jump(anchor, &mut visited, filter.as_deref());
        }
        Commands::Check => {
            println!("--- Running Integrity Check ---");
            if verify_integrity() {
                println!("RESULT: All soldered addresses are physically sound. 🚀");
            } else {
                println!("RESULT: Integrity failures detected. Re-bake required.");
            }
        }
    }
}
