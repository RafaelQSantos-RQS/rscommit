use serial_test::serial;
use std::fs;
use std::path::Path;
use std::process::Command;

/// Helper to run a command in a specific directory
fn run_cmd(cmd: &str, args: &[&str], dir: &Path) -> CommandResult {
    let output = Command::new(cmd)
        .args(args)
        .current_dir(dir)
        .output()
        .expect("Failed to execute command");
    CommandResult {
        status: output.status,
        stdout: String::from_utf8_lossy(&output.stdout).to_string(),
        stderr: String::from_utf8_lossy(&output.stderr).to_string(),
    }
}

struct CommandResult {
    status: std::process::ExitStatus,
    stdout: String,
    stderr: String,
}

impl CommandResult {
    fn success(&self) -> bool {
        self.status.success()
    }

    fn contains_stdout(&self, s: &str) -> bool {
        self.stdout.contains(s)
    }

    fn contains_stderr(&self, s: &str) -> bool {
        self.stderr.contains(s)
    }
}

/// Initialize a git repository
fn git_init(dir: &Path) {
    let result = run_cmd("git", &["init"], dir);
    assert!(result.success(), "git init failed: {}", result.stderr);

    // Configure git user for commits
    run_cmd("git", &["config", "user.email", "test@example.com"], dir);
    run_cmd("git", &["config", "user.name", "Test User"], dir);
}

/// Run rscommit command
fn rscommit_cmd(args: &[&str], dir: &Path) -> CommandResult {
    let binary = get_rscommit_binary();
    run_cmd(&binary, args, dir)
}

/// Get path to rscommit binary
fn get_rscommit_binary() -> String {
    // Try PATH first
    if let Ok(path) = which::which("rscommit") {
        return path.display().to_string();
    }
    // Fallback to cargo target dir
    if let Ok(manifest_dir) = std::env::var("CARGO_MANIFEST_DIR") {
        let debug_path = Path::new(&manifest_dir).join("target/debug/rscommit");
        if debug_path.exists() {
            return debug_path.display().to_string();
        }
        let release_path = Path::new(&manifest_dir).join("target/release/rscommit");
        if release_path.exists() {
            return release_path.display().to_string();
        }
    }
    "rscommit".to_string()
}

/// Run git commit with message
fn git_commit(dir: &Path, msg: &str) -> CommandResult {
    run_cmd("git", &["commit", "--allow-empty", "-m", msg], dir)
}

/// Test helper: setup a fresh repo with rscommit
fn setup_repo_with_config(config: &str) -> tempfile::TempDir {
    let tmp = tempfile::tempdir().expect("Failed to create temp dir");
    let repo = tmp.path();

    git_init(repo);

    // Write custom config if provided
    if !config.is_empty() {
        fs::write(repo.join("rscommit.toml"), config).expect("Write config");
    }

    tmp
}

#[test]
#[serial]
fn test_e2e_install_uninstall_cycle() {
    let tmp = tempfile::tempdir().unwrap();
    let repo = tmp.path();

    git_init(repo);

    // Install hook
    let result = rscommit_cmd(&["install"], repo);
    assert!(result.success(), "install failed: {}", result.stderr);
    assert!(result.contains_stdout("Hook commit-msg instalado"));

    // Hook file exists
    let hook_path = repo.join(".git/hooks/commit-msg");
    assert!(hook_path.exists(), "Hook file not created");

    let content = fs::read_to_string(&hook_path).unwrap();
    assert!(content.contains("rscommit git hook"));
    assert!(content.contains("validate"));

    // Uninstall
    let result = rscommit_cmd(&["uninstall"], repo);
    assert!(result.success(), "uninstall failed: {}", result.stderr);
    assert!(result.contains_stdout("Hook commit-msg removido"));

    // Hook file removed
    assert!(!hook_path.exists(), "Hook file not removed");
}

#[test]
#[serial]
fn test_e2e_hook_blocks_invalid_commit() {
    let tmp = setup_repo_with_config("");
    let repo = tmp.path();

    // Install hook
    let result = rscommit_cmd(&["install"], repo);
    assert!(result.success(), "install failed: {}", result.stderr);

    // Valid commit should pass
    let result = git_commit(repo, "feat: add login");
    assert!(result.success(), "Valid commit failed: {}", result.stderr);
    assert!(result.contains_stderr("Commit message válida"));

    // Invalid type should fail
    let result = git_commit(repo, "invalid: test");
    assert!(!result.success(), "Invalid commit should have failed");
    assert!(result.stderr.contains("Tipo de commit inválido"));

    // Uninstall should restore ability to commit anything
    let result = rscommit_cmd(&["uninstall"], repo);
    assert!(result.success(), "uninstall failed: {}", result.stderr);

    // Now invalid should pass (no hook)
    let result = git_commit(repo, "invalid: test");
    assert!(
        result.success(),
        "Invalid commit should pass after uninstall"
    );
}

#[test]
#[serial]
fn test_e2e_custom_config_applied() {
    let config = r#"
[commit]
allowed_types = ["feat", "fix"]
subject_full_stop = true

[hooks]
commit_msg = false
"#;
    let tmp = setup_repo_with_config(config);
    let repo = tmp.path();

    let result = rscommit_cmd(&["install"], repo);
    assert!(result.success(), "install failed: {}", result.stderr);

    // Type not allowed
    let result = git_commit(repo, "chore: update deps");
    eprintln!("chore stderr: {}", result.stderr);
    eprintln!("chore stdout: {}", result.stdout);
    eprintln!("chore success: {}", result.success());
    assert!(!result.success(), "chore should be rejected");
    assert!(result.stderr.contains("Tipo de commit inválido"));

    // Subject with full stop
    let result = git_commit(repo, "feat: add login.");
    eprintln!("full_stop stderr: {}", result.stderr);
    eprintln!("full_stop stdout: {}", result.stdout);
    eprintln!("full_stop success: {}", result.success());
    assert!(!result.success(), "full stop should be rejected");
    assert!(
        result
            .stderr
            .contains("Subject não deve terminar com ponto final")
    );

    // Valid commit
    let result = git_commit(repo, "feat: add login");
    assert!(result.success(), "Valid commit failed: {}", result.stderr);
}

#[test]
#[serial]
fn test_e2e_install_idempotent() {
    let tmp = tempfile::tempdir().unwrap();
    let repo = tmp.path();

    git_init(repo);

    // First install
    let result = rscommit_cmd(&["install"], repo);
    assert!(result.success());

    // Second install should be idempotent
    let result = rscommit_cmd(&["install"], repo);
    assert!(result.success());
    assert!(result.stdout.contains("já instalado"));
}

#[test]
#[serial]
fn test_e2e_backup_existing_hook() {
    let tmp = tempfile::tempdir().unwrap();
    let repo = tmp.path();

    git_init(repo);

    // Create existing hook
    let hooks_dir = repo.join(".git/hooks");
    fs::create_dir_all(&hooks_dir).unwrap();
    let hook_path = hooks_dir.join("commit-msg");
    fs::write(&hook_path, "#!/bin/sh\necho 'old hook'").unwrap();

    // Install should backup
    let result = rscommit_cmd(&["install"], repo);
    assert!(result.success());
    assert!(result.stdout.contains("movido para"));

    // Backup exists
    let backup = repo.join(".git/hooks/commit-msg.rscommit.bak");
    assert!(backup.exists());
    let backup_content = fs::read_to_string(&backup).unwrap();
    assert_eq!(backup_content, "#!/bin/sh\necho 'old hook'");

    // Uninstall should restore
    let result = rscommit_cmd(&["uninstall"], repo);
    eprintln!("uninstall stdout: {}", result.stdout);
    eprintln!("uninstall stderr: {}", result.stderr);
    assert!(result.success());
    assert!(result.stdout.contains("restaurado"));

    // Original hook restored
    let restored = fs::read_to_string(&hook_path).unwrap();
    assert_eq!(restored, "#!/bin/sh\necho 'old hook'");
}

#[test]
#[serial]
fn test_e2e_non_git_repo_error() {
    let tmp = tempfile::tempdir().unwrap();
    let repo = tmp.path();

    // Not a git repo
    let result = rscommit_cmd(&["install"], repo);
    assert!(!result.success());
    assert!(result.stderr.contains("Not a git repository"));
}

#[test]
#[serial]
fn test_e2e_validate_command_via_hook() {
    let tmp = tempfile::tempdir().unwrap();
    let repo = tmp.path();

    git_init(repo);

    // Test validate command directly (used by hook)
    let result = rscommit_cmd(&["validate", "--message", "feat: add login"], repo);
    assert!(result.success());
    assert!(result.stdout.contains("Commit message válida"));

    let result = rscommit_cmd(&["validate", "--message", "invalid: test"], repo);
    assert!(!result.success());
    assert!(result.stderr.contains("Tipo de commit inválido"));
}
