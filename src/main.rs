mod cli;
mod config;
mod git;
mod validation;

use clap::Parser;
use cli::{Cli, Commands};

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Init => cli::commands::init(),
        Commands::Validate { message } => cli::commands::validate(message),
        Commands::Install => cli::commands::install(),
        Commands::Uninstall => cli::commands::uninstall(),
    }
}
