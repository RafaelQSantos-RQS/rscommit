pub mod commands;

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(
    name = "rscommit",
    version,
    about = "CLI agnóstica para padrões de engenharia - Conventional Commits"
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Inicializa configuração padrão no repositório atual
    Init,
    /// Valida uma mensagem de commit
    Validate {
        /// Mensagem do commit (ou usa stdin/.git/COMMIT_EDITMSG)
        #[arg(short, long)]
        message: Option<String>,
    },
    /// Instala git hook commit-msg
    Install,
    /// Remove git hook commit-msg
    Uninstall,
}
