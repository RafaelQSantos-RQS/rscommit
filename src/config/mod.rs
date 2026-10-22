use serde::{Deserialize, Serialize};

#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Config {
    pub commit: CommitRules,
    pub hooks: HookConfig,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub extends: Option<String>,
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
        "feat".into(),
        "fix".into(),
        "chore".into(),
        "docs".into(),
        "refactor".into(),
        "test".into(),
        "ci".into(),
        "build".into(),
        "perf".into(),
        "revert".into(),
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
fn default_true() -> bool {
    true
}

impl Default for CommitRules {
    fn default() -> Self {
        Self {
            allowed_types: default_types(),
            allowed_scopes: vec![],
            max_subject_length: default_max_subject_length(),
            require_scope: false,
            subject_case: default_subject_case(),
        }
    }
}
