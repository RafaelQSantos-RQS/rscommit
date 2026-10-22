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

    // Type case
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

#[cfg(test)]
mod tests {
    use super::*;

    fn default_rules() -> crate::config::CommitRules {
        crate::config::CommitRules::default()
    }

    // Strict rules but WITHOUT require_scope (most tests don't need scopes)
    // Only basic rules enabled by default. Advanced rules (body, footer, breaking, signed-off-by)
    // are disabled by default and enabled only in tests that specifically test them.
    fn strict_rules() -> crate::config::CommitRules {
        crate::config::CommitRules {
            allowed_types: vec!["feat".into(), "fix".into(), "chore".into()],
            allowed_scopes: vec!["auth".into(), "api".into()],
            max_subject_length: 72,
            require_scope: false,
            subject_case: crate::config::SubjectCase::Lower,
            type_case: crate::config::SubjectCase::Lower,
            subject_full_stop: true,
            subject_max_length: 50,
            header_max_length: 80,
            scope_case: crate::config::SubjectCase::Lower,
            // Advanced rules disabled by default
            body_leading_blank: false,
            body_max_line_length: 100,
            footer_leading_blank: false,
            footer_max_line_length: 100,
            breaking_change_exclamation_mark: false,
            signed_off_by: false,
        }
    }

    // Strict rules WITH require_scope for specific tests
    fn strict_rules_with_scope() -> crate::config::CommitRules {
        let mut r = strict_rules();
        r.require_scope = true;
        r
    }

    fn validate_msg(msg: &str, rules: &crate::config::CommitRules) -> Option<String> {
        match parse_commit_message(msg) {
            Ok(parsed) => validate_commit(&parsed, rules, msg)
                .err()
                .map(|e| e.to_string()),
            Err(e) => Some(e.to_string()),
        }
    }

    // ========== TYPE RULES ==========

    #[test]
    fn test_type_enum_valid() {
        let rules = default_rules();
        assert!(validate_msg("feat: add login", &rules).is_none());
        assert!(validate_msg("fix: resolve bug", &rules).is_none());
        assert!(validate_msg("chore: update deps", &rules).is_none());
    }

    #[test]
    fn test_type_enum_invalid() {
        let rules = strict_rules();
        let err = validate_msg("invalid: msg", &rules).unwrap();
        assert!(err.contains("Tipo de commit inválido"));
        assert!(err.contains("invalid"));
    }

    #[test]
    fn test_type_case_lower() {
        let rules = strict_rules();
        let err = validate_msg("FEAT: add login", &rules).unwrap();
        assert!(err.contains("Tipo 'FEAT' não está em lower"));
    }

    #[test]
    fn test_type_case_lower_valid() {
        let rules = strict_rules();
        assert!(validate_msg("feat: add login", &rules).is_none());
    }

    #[test]
    fn test_type_empty() {
        let rules = strict_rules();
        let err = validate_msg(": add login", &rules).unwrap();
        assert!(err.contains("Missing type") || err.contains("tipo"));
    }

    // ========== SUBJECT RULES ==========

    #[test]
    fn test_subject_full_stop() {
        let rules = strict_rules();
        let err = validate_msg("feat: add login.", &rules).unwrap();
        assert!(err.contains("Subject não deve terminar com ponto final"));
    }

    #[test]
    fn test_subject_full_stop_valid() {
        let rules = strict_rules();
        assert!(validate_msg("feat: add login", &rules).is_none());
    }

    #[test]
    fn test_subject_max_length() {
        let rules = strict_rules();
        let long_subject = "a".repeat(51);
        let err = validate_msg(&format!("feat: {}", long_subject), &rules).unwrap();
        assert!(err.contains("Subject muito longo"));
        assert!(err.contains("51"));
    }

    #[test]
    fn test_subject_max_length_valid() {
        let rules = strict_rules();
        let exact = "a".repeat(50);
        assert!(validate_msg(&format!("feat: {}", exact), &rules).is_none());
    }

    #[test]
    fn test_subject_case_lower() {
        let rules = strict_rules();
        let err = validate_msg("feat: Add login", &rules).unwrap();
        assert!(err.contains("Subject 'Add login' não está em lower"));
    }

    #[test]
    fn test_subject_case_lower_valid() {
        let rules = strict_rules();
        assert!(validate_msg("feat: add login", &rules).is_none());
    }

    // ========== HEADER RULES ==========

    #[test]
    fn test_header_max_length() {
        let mut rules = strict_rules();
        // Increase subject limit so we can test header limit (80) without hitting subject limit (72 default)
        rules.max_subject_length = 100;
        rules.subject_max_length = 100;
        let err = validate_msg("feat(auth): this header is way too long and exceeds the maximum allowed length of eighty characters", &rules).unwrap();
        assert!(err.contains("Header muito longo"));
    }

    #[test]
    fn test_header_max_length_valid() {
        let rules = strict_rules();
        assert!(validate_msg("feat(auth): short header", &rules).is_none());
    }

    // ========== SCOPE RULES ==========

    #[test]
    fn test_scope_enum_valid() {
        let rules = strict_rules();
        assert!(validate_msg("feat(auth): add login", &rules).is_none());
        assert!(validate_msg("fix(api): resolve bug", &rules).is_none());
    }

    #[test]
    fn test_scope_enum_invalid() {
        let rules = strict_rules();
        let err = validate_msg("feat(unknown): test", &rules).unwrap();
        assert!(err.contains("Scope inválido"));
        assert!(err.contains("unknown"));
    }

    #[test]
    fn test_scope_case_lower() {
        let rules = strict_rules();
        let err = validate_msg("feat(AUTH): test", &rules).unwrap();
        assert!(err.contains("Scope 'AUTH' não está em lower"));
    }

    #[test]
    fn test_scope_case_lower_valid() {
        let rules = strict_rules();
        assert!(validate_msg("feat(auth): test", &rules).is_none());
    }

    #[test]
    fn test_require_scope() {
        let rules = strict_rules_with_scope();
        let err = validate_msg("feat: no scope", &rules).unwrap();
        assert!(err.contains("Scope é obrigatório"));
    }

    #[test]
    fn test_require_scope_false() {
        let rules = default_rules();
        assert!(validate_msg("feat: no scope", &rules).is_none());
    }

    // ========== BODY RULES ==========

    #[test]
    fn test_body_max_line_length() {
        let mut rules = strict_rules();
        rules.body_max_line_length = 100;
        let long_line = "a".repeat(101);
        let msg = format!("feat: add login\n\n{}", long_line);
        let err = validate_msg(&msg, &rules).unwrap();
        assert!(err.contains("Linha 1 do body muito longa"));
        assert!(err.contains("101"));
    }

    #[test]
    fn test_body_max_line_length_url_allowed() {
        let mut rules = strict_rules();
        rules.body_max_line_length = 100;
        let url = "https://example.com/very/long/url/that/exceeds/one/hundred/characters/and/should/be/allowed";
        let msg = format!("feat: add login\n\n{}", url);
        assert!(validate_msg(&msg, &rules).is_none());
    }

    // ========== FOOTER RULES ==========

    #[test]
    fn test_footer_max_line_length() {
        let mut rules = strict_rules();
        rules.footer_max_line_length = 100;
        let long_footer = "a".repeat(120);
        let msg = format!("feat: add login\n\n\nBreaking: {}", long_footer);
        let err = validate_msg(&msg, &rules).unwrap();
        assert!(err.contains("Linha do footer muito longa"));
    }

    // ========== BREAKING CHANGE XNOR ==========

    #[test]
    fn test_breaking_change_xnor_both() {
        let mut rules = strict_rules();
        rules.breaking_change_exclamation_mark = true;
        assert!(validate_msg("feat!: add login\n\nBREAKING CHANGE: removes api", &rules).is_none());
    }

    #[test]
    fn test_breaking_change_xnor_neither() {
        let mut rules = strict_rules();
        rules.breaking_change_exclamation_mark = true;
        assert!(validate_msg("feat: add login", &rules).is_none());
    }

    #[test]
    fn test_breaking_change_xnor_only_exclamation() {
        let mut rules = strict_rules();
        rules.breaking_change_exclamation_mark = true;
        let err = validate_msg("feat!: add login", &rules).unwrap();
        assert!(
            err.contains("Breaking change deve ter '!' no header E 'BREAKING CHANGE' no footer")
        );
    }

    #[test]
    fn test_breaking_change_xnor_only_footer() {
        let mut rules = strict_rules();
        rules.breaking_change_exclamation_mark = true;
        let err = validate_msg("feat: add login\n\nBREAKING CHANGE: removes api", &rules).unwrap();
        assert!(
            err.contains("Breaking change deve ter '!' no header E 'BREAKING CHANGE' no footer")
        );
    }

    // ========== SIGNED-OFF-BY ==========

    #[test]
    fn test_signed_off_by_required() {
        let mut rules = strict_rules();
        rules.signed_off_by = true;
        let err = validate_msg("feat: add login", &rules).unwrap();
        assert!(err.contains("Footer 'Signed-off-by' é obrigatório"));
    }

    #[test]
    fn test_signed_off_by_valid() {
        let rules = strict_rules();
        assert!(
            validate_msg(
                "feat: add login\n\nSigned-off-by: John <john@example.com>",
                &rules
            )
            .is_none()
        );
    }

    #[test]
    fn test_signed_off_by_case_insensitive() {
        let rules = strict_rules();
        assert!(
            validate_msg(
                "feat: add login\n\nSIGNED-OFF-BY: John <john@example.com>",
                &rules
            )
            .is_none()
        );
        assert!(
            validate_msg(
                "feat: add login\n\nsigned-off-by: John <john@example.com>",
                &rules
            )
            .is_none()
        );
    }

    // ========== EDGE CASES ==========

    #[test]
    fn test_parse_commit_message_basic() {
        let parsed = parse_commit_message("feat(auth): add login").unwrap();
        assert_eq!(parsed.commit_type, "feat");
        assert_eq!(parsed.raw_type, "feat");
        assert_eq!(parsed.scope, Some("auth".into()));
        assert_eq!(parsed.subject, "add login");
        assert!(!parsed.breaking);
    }

    #[test]
    fn test_parse_commit_message_breaking() {
        let parsed = parse_commit_message("feat!: add login").unwrap();
        assert!(parsed.breaking);
        assert!(parsed.header.contains('!'));
    }

    #[test]
    fn test_parse_commit_message_with_body_and_footer() {
        let parsed =
            parse_commit_message("feat: add login\n\nbody here\n\nBREAKING CHANGE: test").unwrap();
        assert_eq!(parsed.body, Some("body here".into()));
        assert_eq!(parsed.footers.len(), 1);
        assert_eq!(parsed.footers[0].0, "BREAKING CHANGE");
    }

    #[test]
    fn test_case_sentence() {
        let mut rules = strict_rules();
        rules.subject_case = crate::config::SubjectCase::Sentence;
        assert!(validate_msg("feat: Add login", &rules).is_none());
        let err = validate_msg("feat: add login", &rules).unwrap();
        assert!(err.contains("sentence"));
    }

    #[test]
    fn test_case_pascal() {
        let mut rules = strict_rules();
        rules.subject_case = crate::config::SubjectCase::Pascal;
        assert!(validate_msg("feat: Add Login", &rules).is_none());
    }

    #[test]
    fn test_case_kebab() {
        let mut rules = strict_rules();
        rules.subject_case = crate::config::SubjectCase::Kebab;
        assert!(validate_msg("feat: add-login-feature", &rules).is_none());
    }

    #[test]
    fn test_case_any() {
        let mut rules = strict_rules();
        rules.subject_case = crate::config::SubjectCase::Any;
        assert!(validate_msg("feat: AdD LoGiN", &rules).is_none());
    }

    #[test]
    fn test_scope_enum_empty_allows_any() {
        let mut rules = strict_rules();
        rules.allowed_scopes = vec![];
        assert!(validate_msg("feat(any): test", &rules).is_none());
        assert!(validate_msg("feat(whatever): test", &rules).is_none());
    }

    #[test]
    fn test_max_subject_length_original() {
        let mut rules = default_rules();
        rules.max_subject_length = 20;
        let err = validate_msg("feat: this subject is too long", &rules).unwrap();
        assert!(err.contains("Subject muito longo"));
    }

    #[test]
    fn test_breaking_change_disabled() {
        let mut rules = strict_rules();
        rules.breaking_change_exclamation_mark = false;
        assert!(validate_msg("feat!: add login", &rules).is_none());
        assert!(validate_msg("feat: add login\n\nBREAKING CHANGE: test", &rules).is_none());
    }

    #[test]
    fn test_signed_off_by_disabled() {
        let mut rules = strict_rules();
        rules.signed_off_by = false;
        assert!(validate_msg("feat: add login", &rules).is_none());
    }
}
