use serde::{Deserialize, Serialize};
use std::path::Path;

#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Config {
    pub commit: CommitRules,
    pub hooks: HookConfig,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub extends: Option<String>,
}

impl Config {
    pub fn load(path: &Path) -> anyhow::Result<Self> {
        if !path.exists() {
            return Ok(Config::default());
        }
        let content = std::fs::read_to_string(path)?;
        let config: Config = toml::from_str(&content)?;
        Ok(config)
    }
}

#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize)]
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

#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
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
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct HookConfig {
    #[serde(default = "default_true")]
    pub commit_msg: bool,
}

#[allow(dead_code)]
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

#[allow(dead_code)]
fn default_max_subject_length() -> usize {
    72
}

#[allow(dead_code)]
fn default_subject_case() -> SubjectCase {
    SubjectCase::Lower
}

#[allow(dead_code)]
fn default_type_case() -> SubjectCase {
    SubjectCase::Lower
}

#[allow(dead_code)]
fn default_scope_case() -> SubjectCase {
    SubjectCase::Lower
}

#[allow(dead_code)]
fn default_subject_max_length() -> usize {
    72
}

#[allow(dead_code)]
fn default_header_max_length() -> usize {
    72
}

#[allow(dead_code)]
fn default_body_max_line_length() -> usize {
    100
}

#[allow(dead_code)]
fn default_footer_max_line_length() -> usize {
    100
}

#[allow(dead_code)]
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
