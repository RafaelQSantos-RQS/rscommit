use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
pub struct Config {
    pub commit: CommitRules,
    pub hooks: HookConfig,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub extends: Option<String>,
}

impl Config {
    pub fn load(path: &Path) -> anyhow::Result<Self> {
        Self::load_with_tracking(path, &mut HashSet::new(), 0)
    }

    fn load_with_tracking(
        path: &Path,
        visited: &mut HashSet<PathBuf>,
        depth: usize,
    ) -> anyhow::Result<Self> {
        if depth > 10 {
            anyhow::bail!("extends chain too deep (max 10), possible cycle");
        }

        let canonical = path.canonicalize().unwrap_or_else(|_| path.to_path_buf());
        if !visited.insert(canonical.clone()) {
            anyhow::bail!("Circular extends detected: {}", path.display());
        }

        if !path.exists() {
            return Ok(Config::default());
        }

        let content = std::fs::read_to_string(path)?;
        let mut config: Config = toml::from_str(&content)?;

        // Handle extends if present
        if let Some(extends_path) = config.extends.take() {
            let parent_path = resolve_extends_path(path, &extends_path)?;
            let mut parent = Self::load_with_tracking(&parent_path, visited, depth + 1)?;
            parent.merge(config);
            return Ok(parent);
        }

        Ok(config)
    }

    /// Merge another config into this one (child overrides parent with deep merge)
    fn merge(&mut self, other: Config) {
        self.commit.merge_commit_rules(other.commit);
        self.hooks = other.hooks; // hooks replaced entirely
    }
}

fn resolve_extends_path(current_config: &Path, extends: &str) -> anyhow::Result<PathBuf> {
    // URL (http/https) - not implemented yet, return error
    if extends.starts_with("http://") || extends.starts_with("https://") {
        anyhow::bail!("URL extends not yet supported: {}", extends);
    }

    let path = Path::new(extends);
    if path.is_absolute() {
        Ok(path.to_path_buf())
    } else {
        // Relative to current config's directory
        let base = current_config
            .parent()
            .ok_or_else(|| anyhow::anyhow!("Cannot resolve parent directory"))?;
        Ok(base.join(path))
    }
}
#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CommitRules {
    #[serde(default = "default_types")]
    pub allowed_types: Vec<String>,
    #[serde(default)]
    pub allowed_scopes: Vec<String>,
    #[serde(default = "default_max_subject_length")]
    pub max_subject_length: usize,
    #[serde(default)]
    pub require_scope: bool,
    #[serde(default = "default_subject_case")]
    pub subject_case: SubjectCase,

    // Type rules
    #[serde(default = "default_type_case")]
    pub type_case: SubjectCase,

    // Subject rules
    #[serde(default)]
    pub subject_full_stop: bool,
    #[serde(
        default = "default_subject_max_length",
        skip_serializing_if = "is_default_subject_max_length"
    )]
    pub subject_max_length: usize,

    // Header rules
    #[serde(default = "default_header_max_length")]
    pub header_max_length: usize,

    // Scope rules
    #[serde(default = "default_scope_case")]
    pub scope_case: SubjectCase,

    // Body rules
    #[serde(default)]
    pub body_leading_blank: bool,
    #[serde(default = "default_body_max_line_length")]
    pub body_max_line_length: usize,

    // Footer rules
    #[serde(default)]
    pub footer_leading_blank: bool,
    #[serde(default = "default_footer_max_line_length")]
    pub footer_max_line_length: usize,

    // Breaking change rules
    #[serde(default)]
    pub breaking_change_exclamation_mark: bool,

    // Signed-off-by
    #[serde(default)]
    pub signed_off_by: bool,
}

impl CommitRules {
    /// Merge another CommitRules into this one (child overrides parent with deep merge)
    fn merge_commit_rules(&mut self, other: CommitRules) {
        let default = CommitRules::default();

        // For arrays: replace entirely if other has non-default values
        if other.allowed_types != default.allowed_types {
            self.allowed_types = other.allowed_types;
        }
        if other.allowed_scopes != default.allowed_scopes {
            self.allowed_scopes = other.allowed_scopes;
        }

        // For scalars: only override if explicitly set (non-default)
        if other.max_subject_length != default.max_subject_length {
            self.max_subject_length = other.max_subject_length;
        }
        if other.require_scope != default.require_scope {
            self.require_scope = other.require_scope;
        }
        if other.subject_case != default.subject_case {
            self.subject_case = other.subject_case;
        }
        if other.type_case != default.type_case {
            self.type_case = other.type_case;
        }
        if other.subject_full_stop != default.subject_full_stop {
            self.subject_full_stop = other.subject_full_stop;
        }
        if other.subject_max_length != default.subject_max_length {
            self.subject_max_length = other.subject_max_length;
        }
        if other.header_max_length != default.header_max_length {
            self.header_max_length = other.header_max_length;
        }
        if other.scope_case != default.scope_case {
            self.scope_case = other.scope_case;
        }
        if other.body_leading_blank != default.body_leading_blank {
            self.body_leading_blank = other.body_leading_blank;
        }
        if other.body_max_line_length != default.body_max_line_length {
            self.body_max_line_length = other.body_max_line_length;
        }
        if other.footer_leading_blank != default.footer_leading_blank {
            self.footer_leading_blank = other.footer_leading_blank;
        }
        if other.footer_max_line_length != default.footer_max_line_length {
            self.footer_max_line_length = other.footer_max_line_length;
        }
        if other.breaking_change_exclamation_mark != default.breaking_change_exclamation_mark {
            self.breaking_change_exclamation_mark = other.breaking_change_exclamation_mark;
        }
        if other.signed_off_by != default.signed_off_by {
            self.signed_off_by = other.signed_off_by;
        }
    }
}
#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum SubjectCase {
    #[default]
    Lower,
    Upper,
    Sentence,
    Pascal,
    Kebab,
    Any,
}
#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
pub struct HookConfig {
    #[serde(default = "default_true")]
    pub commit_msg: bool,
}

fn default_types() -> Vec<String> {
    vec![
        "build".into(),
        "chore".into(),
        "ci".into(),
        "docs".into(),
        "feat".into(),
        "fix".into(),
        "perf".into(),
        "refactor".into(),
        "revert".into(),
        "style".into(),
        "test".into(),
    ]
}

fn default_max_subject_length() -> usize {
    72
}

fn default_subject_case() -> SubjectCase {
    SubjectCase::Lower
}

fn default_type_case() -> SubjectCase {
    SubjectCase::Lower
}

fn default_scope_case() -> SubjectCase {
    SubjectCase::Lower
}

fn default_subject_max_length() -> usize {
    72
}

fn default_header_max_length() -> usize {
    72
}

fn default_body_max_line_length() -> usize {
    100
}

fn default_footer_max_line_length() -> usize {
    100
}

fn default_true() -> bool {
    true
}

fn is_default_subject_max_length(value: &usize) -> bool {
    *value == 72
}

impl Default for CommitRules {
    fn default() -> Self {
        Self {
            allowed_types: default_types(),
            allowed_scopes: vec![],
            max_subject_length: default_max_subject_length(),
            require_scope: false,
            subject_case: default_subject_case(),
            type_case: default_type_case(),
            subject_full_stop: false,
            subject_max_length: default_subject_max_length(),
            header_max_length: default_header_max_length(),
            scope_case: default_scope_case(),
            body_leading_blank: false,
            body_max_line_length: default_body_max_line_length(),
            footer_leading_blank: false,
            footer_max_line_length: default_footer_max_line_length(),
            breaking_change_exclamation_mark: false,
            signed_off_by: false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn test_extends_basic() {
        let tmp = tempdir().unwrap();
        let base = tmp.path().join("base.toml");
        let child = tmp.path().join("child.toml");

        fs::write(
            &base,
            r##"
[commit]
allowed_types = ["feat", "fix"]
max_subject_length = 50

[hooks]
commit_msg = false
"##,
        )
        .unwrap();

        fs::write(
            &child,
            r##"
extends = "base.toml"

[commit]
allowed_types = ["chore"]
subject_full_stop = true

[hooks]
commit_msg = false
"##,
        )
        .unwrap();

        let config = Config::load(&child).unwrap();
        assert_eq!(config.commit.allowed_types, vec!["chore"]);
        assert_eq!(config.commit.max_subject_length, 50);
        assert!(config.commit.subject_full_stop);
    }

    #[test]
    fn test_extends_relative_path() {
        let tmp = tempdir().unwrap();
        let config_dir = tmp.path().join("config");
        fs::create_dir_all(&config_dir).unwrap();

        let base = config_dir.join("base.toml");
        let child = tmp.path().join("child.toml");

        fs::write(
            &base,
            r##"
[commit]
allowed_types = ["feat"]
header_max_length = 80

[hooks]
commit_msg = false
"##,
        )
        .unwrap();

        fs::write(
            &child,
            r##"
extends = "config/base.toml"

[commit]
allowed_types = ["fix"]

[hooks]
commit_msg = false
"##,
        )
        .unwrap();

        let config = Config::load(&child).unwrap();
        assert_eq!(config.commit.allowed_types, vec!["fix"]);
        assert_eq!(config.commit.header_max_length, 80);
    }

    #[test]
    fn test_extends_cycle_detection() {
        let tmp = tempdir().unwrap();
        let a = tmp.path().join("a.toml");
        let b = tmp.path().join("b.toml");

        fs::write(
            &a,
            r##"
extends = "b.toml"

[commit]
allowed_types = ["feat"]

[hooks]
commit_msg = false
"##,
        )
        .unwrap();
        fs::write(
            &b,
            r##"
extends = "a.toml"

[commit]
allowed_types = ["fix"]

[hooks]
commit_msg = false
"##,
        )
        .unwrap();

        let result = Config::load(&a);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Circular"));
    }

    #[test]
    fn test_extends_chain_too_deep() {
        let tmp = tempdir().unwrap();

        let mut prev = tmp.path().join("base.toml");
        fs::write(
            &prev,
            r##"
[commit]
allowed_types = ["base"]

[hooks]
commit_msg = false
"##,
        )
        .unwrap();

        for i in 1..=11 {
            let current = tmp.path().join(format!("level{}.toml", i));
            fs::write(
                &current,
                format!(
                    r##"
extends = "{}"

[commit]
allowed_types = ["level{}"]

[hooks]
commit_msg = false
"##,
                    prev.file_name().unwrap().to_str().unwrap(),
                    i
                ),
            )
            .unwrap();
            prev = current;
        }

        let result = Config::load(&prev);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("too deep"));
    }

    #[test]
    fn test_extends_missing_file_uses_defaults() {
        let tmp = tempdir().unwrap();
        let child = tmp.path().join("child.toml");

        fs::write(
            &child,
            r##"
extends = "nonexistent.toml"

[commit]
allowed_types = ["feat"]

[hooks]
commit_msg = false
"##,
        )
        .unwrap();

        let config = Config::load(&child).unwrap();
        assert_eq!(config.commit.allowed_types, vec!["feat"]);
    }

    #[test]
    fn test_extends_absolute_path() {
        let tmp = tempdir().unwrap();
        let base = tmp.path().join("base.toml");
        let child = tmp.path().join("child.toml");

        fs::write(
            &base,
            r##"
[commit]
allowed_types = ["feat", "fix"]

[hooks]
commit_msg = false
"##,
        )
        .unwrap();

        fs::write(
            &child,
            format!(
                r##"
extends = "{}"

[commit]
allowed_types = ["chore"]

[hooks]
commit_msg = false
"##,
                base.display().to_string().replace('\\', "/")
            ),
        )
        .unwrap();

        let config = Config::load(&child).unwrap();
        assert_eq!(config.commit.allowed_types, vec!["chore"]);
    }
}
