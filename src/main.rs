mod arena;

use std::{
    error::Error,
    path::{Path, PathBuf},
};

use clap::{command, Parser, Subcommand};
use tracing::info;

#[derive(Parser)]
#[command(author, version, about, long_about = None)] // Read from `Cargo.toml`
#[command(propagate_version = true)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Create a new arena at `path`
    New { path: String },
    /// Run server or worker
    Run {
        #[command(subcommand)]
        command: RunCommands,
    },
}

#[derive(Subcommand)]
enum RunCommands {
    /// Run server
    Server { path: String },
    /// Run worker
    Worker {
        #[arg(short, long, default_value_t = 1)]
        threads: u8,
    },
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    tracing_subscriber::fmt().init();
    init_colored();

    let cli = Cli::parse();
    match cli.command {
        Commands::New { path } => {
            let path = PathBuf::from(path);
            arena::create_new_arena(&path)?;
            info!("New arena has been created");
            Ok(())
        }
        Commands::Run { command } => handle_run(command).await,
    }
}

async fn handle_run(command: RunCommands) -> Result<(), Box<dyn Error>> {
    match command {
        RunCommands::Server { path } => api::start_arena_server(Path::new(&path)).await,
        RunCommands::Worker { threads: _ } => todo!(),
    }
}

#[cfg(windows)]
fn init_colored() {
    colored::control::set_virtual_terminal(true).unwrap();
}

#[cfg(not(windows))]
fn init_colored() {}
