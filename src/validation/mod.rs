use anyhow::Result;
use git_conventional::Commit;

#[derive(Debug, Clone)]
pub struct ValidatedCommit {
    pub commit_type: String,
    pub raw_type: String,
    pub scope: Option<String>,
    pub subject: String,
    pub body: Option<String>,
    pub breaking: bool,
    pub footers: Vec<(String, String)>,
    pub header: String,
}

impl<'a> From<Commit<'a>> for ValidatedCommit {
    fn from(commit: Commit<'a>) -> Self {
        let raw_type = commit.type_().to_string();
        let mut header = raw_type.to_string();
        if let Some(scope) = commit.scope() {
            header.push_str(&format!("({})", scope));
        }
        if commit.breaking() && !commit.description().is_empty() {
            header.push('!');
        }
        header.push_str(&format!(": {}", commit.description()));

        Self {
            commit_type: raw_type.to_lowercase(),
            raw_type,
            scope: commit.scope().map(|s| s.to_string()),
            subject: commit.description().to_string(),
            body: commit.body().map(|s| s.to_string()),
            breaking: commit.breaking(),
            footers: commit
                .footers()
                .iter()
                .map(|f| (f.token().to_string(), f.value().to_string()))
                .collect(),
            header,
        }
    }
}

pub fn parse_commit_message(message: &str) -> Result<ValidatedCommit> {
    let commit = Commit::parse(message)?;
    Ok(commit.into())
}

fn check_case(value: &str, case: &crate::config::SubjectCase) -> bool {
    match case {
        crate::config::SubjectCase::Lower => value
            .chars()
            .all(|c| c.is_lowercase() || !c.is_alphabetic()),
        crate::config::SubjectCase::Upper => value
            .chars()
            .all(|c| c.is_uppercase() || !c.is_alphabetic()),
        crate::config::SubjectCase::Sentence => {
            let first_alpha = value.chars().find(|c| c.is_alphabetic());
            matches!(first_alpha, Some(c) if c.is_uppercase())
                && value
                    .chars()
                    .skip(1)
                    .all(|c| c.is_lowercase() || !c.is_alphabetic())
        }
        crate::config::SubjectCase::Pascal => {
            value.split(|c: char| !c.is_alphanumeric()).all(|word| {
                word.chars()
                    .next()
                    .map(|c| c.is_uppercase())
                    .unwrap_or(true)
            })
        }
        crate::config::SubjectCase::Kebab => value
            .split('-')
            .all(|part| part.chars().all(|c| c.is_lowercase() || c.is_ascii_digit())),
        crate::config::SubjectCase::Any => true,
    }
}

fn check_header_max_length(header: &str, max: usize) -> Result<()> {
    if header.len() > max {
        anyhow::bail!(
            "Header muito longo ({} chars). Máximo: {}",
            header.len(),
            max
        );
    }
    Ok(())
}

fn check_subject_full_stop(subject: &str, enabled: bool) -> Result<()> {
    if enabled && subject.ends_with('.') {
        anyhow::bail!("Subject não deve terminar com ponto final");
    }
    Ok(())
}

fn check_subject_max_length(subject: &str, max: usize) -> Result<()> {
    if subject.len() > max {
        anyhow::bail!(
            "Subject muito longo ({} chars). Máximo: {}",
            subject.len(),
            max
        );
    }
    Ok(())
}

fn check_scope_case(scope: &str, case: &crate::config::SubjectCase) -> Result<()> {
    if !check_case(scope, case) {
        anyhow::bail!(
            "Scope '{}' não está em {}",
            scope,
            format!("{:?}", case).to_lowercase()
        );
    }
    Ok(())
}

fn check_body_leading_blank(body: &Option<String>, header: &str, message: &str) -> Result<()> {
    if let Some(body_text) = body
        && !body_text.is_empty()
    {
        let header_end = message.find(body_text).unwrap_or(0);
        let between = &message[header.len()..header_end];
        if !between.trim().is_empty() && !between.contains("\n\n") && !between.contains("\r\n\r\n")
        {
            anyhow::bail!("Body deve ser precedido por uma linha em branco");
        }
    }
    Ok(())
}

fn check_footer_leading_blank(message: &str, footers: &[(String, String)]) -> Result<()> {
    if !footers.is_empty() {
        let footer_start = message.rfind(&footers[0].0).unwrap_or(message.len());
        let before_footer = &message[..footer_start];
        // Check if there's a blank line (two consecutive newlines) before footer
        if !before_footer.ends_with("\n\n") && !before_footer.trim_end().is_empty() {
            anyhow::bail!("Footer deve ser precedido por uma linha em branco");
        }
    }
    Ok(())
}

fn check_body_max_line_length(body: &Option<String>, max: usize) -> Result<()> {
    if let Some(body_text) = body {
        for (i, line) in body_text.lines().enumerate() {
            if line.len() > max && !line.contains("://") {
                anyhow::bail!(
                    "Linha {} do body muito longa ({} chars). Máximo: {} (URLs são permitidas)",
                    i + 1,
                    line.len(),
                    max
                );
            }
        }
    }
    Ok(())
}

fn check_footer_max_line_length(footers: &[(String, String)], max: usize) -> Result<()> {
    for (token, value) in footers {
        let line = format!("{}: {}", token, value);
        if line.len() > max {
            anyhow::bail!(
                "Linha do footer muito longa ({} chars). Máximo: {}",
                line.len(),
                max
            );
        }
    }
    Ok(())
}

fn check_breaking_change_xnor(
    breaking: bool,
    footers: &[(String, String)],
    message: &str,
) -> Result<()> {
    let has_breaking_footer = footers
        .iter()
        .any(|(token, _)| token.eq_ignore_ascii_case("BREAKING CHANGE"));
    let has_exclamation = message.contains("!)") || message.contains("!:");

    if breaking != (has_breaking_footer && has_exclamation) {
        anyhow::bail!(
            "Breaking change deve ter '!' no header E 'BREAKING CHANGE' no footer (ambos ou nenhum)"
        );
    }
    Ok(())
}

fn check_signed_off_by(footers: &[(String, String)], enabled: bool) -> Result<()> {
    if enabled {
        let has_signed_off = footers
            .iter()
            .any(|(token, _)| token.eq_ignore_ascii_case("Signed-off-by"));
        if !has_signed_off {
            anyhow::bail!("Footer 'Signed-off-by' é obrigatório");
        }
    }
    Ok(())
}

pub fn validate_commit(
    commit: &ValidatedCommit,
    rules: &crate::config::CommitRules,
    raw_message: &str,
) -> Result<()> {
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

    // Type case (check raw_type before lowercasing)
    if !check_case(&commit.raw_type, &rules.type_case) {
        anyhow::bail!(
            "Tipo '{}' não está em {}",
            commit.raw_type,
            format!("{:?}", rules.type_case).to_lowercase()
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
        check_scope_case(scope, &rules.scope_case)?;
    } else if rules.require_scope {
        anyhow::bail!("Scope é obrigatório");
    }

    // Subject length (original max_subject_length)
    if commit.subject.len() > rules.max_subject_length {
        anyhow::bail!(
            "Subject muito longo ({} chars). Máximo: {}",
            commit.subject.len(),
            rules.max_subject_length
        );
    }

    // Subject max length (separate rule)
    check_subject_max_length(&commit.subject, rules.subject_max_length)?;

    // Subject full stop
    check_subject_full_stop(&commit.subject, rules.subject_full_stop)?;

    // Subject case (never sentence/start/pascal/upper per commitlint)
    if !check_case(&commit.subject, &rules.subject_case) {
        anyhow::bail!(
            "Subject '{}' não está em {}",
            commit.subject,
            format!("{:?}", rules.subject_case).to_lowercase()
        );
    }

    // Header max length
    check_header_max_length(&commit.header, rules.header_max_length)?;

    // Body leading blank
    check_body_leading_blank(&commit.body, &commit.header, raw_message)?;

    // Body max line length
    check_body_max_line_length(&commit.body, rules.body_max_line_length)?;

    // Footer leading blank
    check_footer_leading_blank(raw_message, &commit.footers)?;

    // Footer max line length
    check_footer_max_line_length(&commit.footers, rules.footer_max_line_length)?;

    // Breaking change XNOR
    if rules.breaking_change_exclamation_mark {
        check_breaking_change_xnor(commit.breaking, &commit.footers, raw_message)?;
    }

    // Signed-off-by
    if rules.signed_off_by {
        check_signed_off_by(&commit.footers, rules.signed_off_by)?;
    }

    Ok(())
}
