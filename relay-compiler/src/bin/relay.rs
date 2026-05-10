use clap::{Parser, Subcommand};
use relay_compiler::{relay_jump_from, verify_integrity_from};
use std::collections::HashSet;
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "relay")]
#[command(version = "1.2")]
#[command(about = "The Universal RelayDB 4-Tag Protocol CLI", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Navigate the graph starting at a specific Anchor (#)
    Jump {
        /// The .relay file to read
        #[arg(short, long, default_value = "output.relay")]
        file: PathBuf,

        /// The Anchor ID to start from, e.g. project:relaydb
        anchor: String,

        /// Optional text filter
        #[arg(short = 'F', long)]
        filter: Option<String>,
    },

    /// Verify the physical integrity of a .relay file
    Check {
        /// The .relay file to verify
        #[arg(short, long, default_value = "output.relay")]
        file: PathBuf,
    },
}

fn main() {
    let cli = Cli::parse();

    match &cli.command {
        Commands::Jump {
            file,
            anchor,
            filter,
        } => {
            println!(
                "--- RELAY-JUMP: Teleporting to #{} in {} ---",
                anchor,
                file.display()
            );

            let mut visited = HashSet::new();
            relay_jump_from(file, anchor, &mut visited, filter.as_deref());

            println!(
                "\n--- Traversal Complete: {} nodes mapped ---",
                visited.len()
            );
        }

        Commands::Check { file } => {
            println!("--- RELAY-CHECK: Auditing {} ---", file.display());

            if verify_integrity_from(file) {
                println!("SUCCESS: System is physically sound and ready for transport. 🚀");
                std::process::exit(0);
            } else {
                eprintln!("CRITICAL: Data corruption or address mismatch detected.");
                std::process::exit(1);
            }
        }
    }
}
