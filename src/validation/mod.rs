use anyhow::Result;
use git_conventional::Commit;

#[derive(Debug, Clone)]
pub struct ValidatedCommit {
    pub commit_type: String,
    pub scope: Option<String>,
    pub subject: String,
    pub body: Option<String>,
    pub breaking: bool,
    pub footers: Vec<(String, String)>,
}

impl<'a> From<Commit<'a>> for ValidatedCommit {
    fn from(commit: Commit<'a>) -> Self {
        Self {
            commit_type: commit.type_().to_string().to_lowercase(),
            scope: commit.scope().map(|s| s.to_string()),
            subject: commit.description().to_string(),
            body: commit.body().map(|s| s.to_string()),
            breaking: commit.breaking(),
            footers: commit
                .footers()
                .iter()
                .map(|f| (f.token().to_string(), f.value().to_string()))
                .collect(),
        }
    }
}

pub fn parse_commit_message(message: &str) -> Result<ValidatedCommit> {
    let commit = Commit::parse(message)?;
    Ok(commit.into())
}

pub fn validate_commit(commit: &ValidatedCommit, rules: &crate::config::CommitRules) -> Result<()> {
    // Type validation
    if !rules
        .allowed_types
        .iter()
        .any(|t| t.eq_ignore_ascii_case(&commit.commit_type))
    {
        anyhow::bail!(
            "Tipo de commit inválido: '{}'. Tipos permitidos: {}",
            commit.commit_type,
            rules.allowed_types.join(", ")
        );
    }

    // Scope validation
    if let Some(scope) = &commit.scope {
        if !rules.allowed_scopes.is_empty()
            && !rules
                .allowed_scopes
                .iter()
                .any(|s| s.eq_ignore_ascii_case(scope))
        {
            anyhow::bail!(
                "Scope inválido: '{}'. Scopes permitidos: {}",
                scope,
                rules.allowed_scopes.join(", ")
            );
        }
    } else if rules.require_scope {
        anyhow::bail!("Scope é obrigatório");
    }

    // Subject length
    if commit.subject.len() > rules.max_subject_length {
        anyhow::bail!(
            "Subject muito longo ({} chars). Máximo: {}",
            commit.subject.len(),
            rules.max_subject_length
        );
    }

    Ok(())
}
