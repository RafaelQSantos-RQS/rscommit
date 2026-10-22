use crate::config::Config;
use crate::git::{install_commit_msg_hook, uninstall_commit_msg_hook};
use crate::validation::{parse_commit_message, validate_commit};
use anyhow::Result;
use std::fs;
use std::path::Path;
use toml;

pub fn init() -> Result<()> {
    let config_path = Path::new("rscommit.toml");

    if config_path.exists() {
        println!("⚠ rscommit.toml já existe");
        return Ok(());
    }

    let config = Config::default();
    let toml_str = toml::to_string_pretty(&config)?;

    fs::write(config_path, toml_str)?;

    println!("✓ rscommit.toml criado com configuração padrão");

    // Check if we're in a git repo
    if Path::new(".git").exists() {
        println!("  Repositório Git detectado. Próximos passos:");
        println!("  1. Revise rscommit.toml se necessário");
        println!("  2. Rode: rscommit install");
    } else {
        println!("  ⚠ Nenhum repositório Git detectado (sem .git)");
        println!("  Rode 'git init' primeiro se quiser usar hooks");
    }

    Ok(())
}

pub fn validate(message: Option<String>) -> Result<()> {
    let raw_msg = match message {
        Some(m) => m,
        None => {
            let hook_msg_path = Path::new(".git/COMMIT_EDITMSG");
            if hook_msg_path.exists() {
                fs::read_to_string(hook_msg_path)?
            } else {
                use std::io::Read;
                let mut buf = String::new();
                std::io::stdin().read_to_string(&mut buf)?;
                buf.trim().to_string()
            }
        }
    };

    let config = Config::load(Path::new("rscommit.toml"))?;

    let parsed = parse_commit_message(&raw_msg)?;
    validate_commit(&parsed, &config.commit, &raw_msg)?;

    println!("✓ Commit message válida");
    Ok(())
}

pub fn install() -> Result<()> {
    let repo_path = Path::new(".");
    install_commit_msg_hook(repo_path)
}

pub fn uninstall() -> Result<()> {
    let repo_path = Path::new(".");
    uninstall_commit_msg_hook(repo_path)
}
