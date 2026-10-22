use anyhow::{Context, Result};
use std::env;
use std::fs;
use std::path::Path;

fn get_rscommit_path() -> String {
    // Try to find rscommit binary
    // 1. Check if installed in PATH
    if let Ok(path) = which::which("rscommit") {
        return path.display().to_string();
    }
    // 2. Check cargo target directory (development)
    if let Ok(manifest_dir) = env::var("CARGO_MANIFEST_DIR") {
        let debug_path = Path::new(&manifest_dir).join("target/debug/rscommit");
        if debug_path.exists() {
            return debug_path.display().to_string();
        }
        let release_path = Path::new(&manifest_dir).join("target/release/rscommit");
        if release_path.exists() {
            return release_path.display().to_string();
        }
    }
    // 3. Fallback to just "rscommit" (will use PATH)
    "rscommit".to_string()
}

const HOOK_TEMPLATE: &str = r#"#!/bin/sh
# rscommit git hook - automatically generated
# Do not edit manually, use 'rscommit uninstall' to remove

exec "{RS_COMMIT_PATH}" validate 2>&1
"#;

pub fn install_commit_msg_hook(repo_path: &Path) -> Result<()> {
    let git_dir = repo_path.join(".git");
    if !git_dir.exists() {
        anyhow::bail!(
            "Not a git repository: {} (no .git directory)",
            repo_path.display()
        );
    }

    let hooks_dir = git_dir.join("hooks");
    fs::create_dir_all(&hooks_dir).context("Failed to create .git/hooks directory")?;

    let hook_path = hooks_dir.join("commit-msg");

    // Check if hook already exists and was created by us
    if hook_path.exists() {
        let existing = fs::read_to_string(&hook_path)?;
        if existing.contains("rscommit git hook") {
            println!("⚠ Hook commit-msg já instalado");
            return Ok(());
        } else {
            // Backup existing hook
            let backup = hooks_dir.join("commit-msg.rscommit.bak");
            fs::copy(&hook_path, &backup)?;
            println!("⚠ Hook existente movido para: {}", backup.display());
        }
    }

    let rscommit_path = get_rscommit_path();
    let hook_content = HOOK_TEMPLATE.replace("{RS_COMMIT_PATH}", &rscommit_path);

    fs::write(&hook_path, hook_content).context("Failed to write commit-msg hook")?;

    // Make executable on Unix
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(&hook_path)?.permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&hook_path, perms)?;
    }

    println!("✓ Hook commit-msg instalado em: {}", hook_path.display());
    println!("  Usando rscommit em: {}", rscommit_path);
    Ok(())
}

pub fn uninstall_commit_msg_hook(repo_path: &Path) -> Result<()> {
    let git_dir = repo_path.join(".git");
    if !git_dir.exists() {
        anyhow::bail!(
            "Not a git repository: {} (no .git directory)",
            repo_path.display()
        );
    }

    let hooks_dir = git_dir.join("hooks");
    let hook_path = hooks_dir.join("commit-msg");

    if !hook_path.exists() {
        println!("⚠ Nenhum hook commit-msg encontrado");
        return Ok(());
    }

    let existing = fs::read_to_string(&hook_path)?;

    if existing.contains("rscommit git hook") {
        fs::remove_file(&hook_path)?;
        println!("✓ Hook commit-msg removido: {}", hook_path.display());

        // Restore backup if exists
        let backup = hooks_dir.join("commit-msg.rscommit.bak");
        if backup.exists() {
            fs::rename(&backup, &hook_path)?;
            println!("✓ Hook anterior restaurado de: {}", backup.display());
        }
    } else {
        println!("⚠ Hook commit-msg não foi gerado pelo rscommit, mantendo intacto");
    }

    Ok(())
}

    #[allow(dead_code)]
pub fn is_git_repo(path: &Path) -> bool {
    path.join(".git").exists()
}
