mod plan_archive;
mod session_start;

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "cc-panes-hook", about = "Claude Code hook for CC-Panes")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// SessionStart hook - inject project and workspace context
    SessionStart,
    /// PostToolUse hook - archive plan files
    PlanArchive,
}

fn main() {
    let cli = Cli::parse();
    match cli.command {
        Commands::SessionStart => session_start::run(),
        Commands::PlanArchive => plan_archive::run(),
    }
}
