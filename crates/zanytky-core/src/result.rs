//! The AI's answer (`result.json`): parsing, validation and the review text (ADR-0007).
//!
//! Nothing here writes to the database. The investigator reads the review
//! first; only a confirmed result is imported.

use std::collections::{HashMap, HashSet};
use std::fmt;

use serde::Deserialize;

use crate::model::{EntityKind, Reliability, SourceKind, StatementType};

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ResultSource {
    #[serde(rename = "ref")]
    pub reference: String,
    pub kind: String,
    pub title: String,
    pub url: Option<String>,
    pub published_at: Option<String>,
    pub reliability: String,
    /// `ref` of the source this one was received from.
    pub received_from: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ResultEntity {
    #[serde(rename = "ref")]
    pub reference: String,
    pub kind: String,
    pub token: Option<String>,
    pub label: Option<String>,
    pub public_code: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ResultParty {
    pub entity: String,
    pub role: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ResultStatement {
    pub kind: String,
    pub value: Option<String>,
    pub valid_from: Option<String>,
    pub valid_to: Option<String>,
    pub as_of: Option<String>,
    pub parties: Vec<ResultParty>,
    pub sources: Vec<String>,
    pub credibility: u8,
    pub quote: String,
    pub note: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ResultFile {
    pub package: String,
    pub sources: Vec<ResultSource>,
    pub entities: Vec<ResultEntity>,
    pub statements: Vec<ResultStatement>,
}

#[derive(Debug, PartialEq, Eq)]
pub enum ResultError {
    Json(String),
    WrongPackage { expected: String, found: String },
    Invalid(String),
}

impl fmt::Display for ResultError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ResultError::Json(message) => write!(f, "not valid result.json: {message}"),
            ResultError::WrongPackage { expected, found } => {
                write!(f, "result is for package {found}, expected {expected}")
            }
            ResultError::Invalid(message) => write!(f, "invalid result: {message}"),
        }
    }
}

impl std::error::Error for ResultError {}

fn invalid(message: impl Into<String>) -> ResultError {
    ResultError::Invalid(message.into())
}

/// Shape of a pseudonym: one prefix letter, a hyphen, 26 base32 characters.
fn valid_token(token: &str) -> bool {
    let (prefix, body) = token.split_once('-').unwrap_or(("", ""));
    matches!(prefix, "N" | "I" | "E" | "T" | "X" | "M" | "B" | "V" | "A")
        && body.len() == 26
        && body
            .chars()
            .all(|c| c.is_ascii_uppercase() || ('2'..='7').contains(&c))
}

fn validate_sources(sources: &[ResultSource]) -> Result<(), ResultError> {
    let mut refs = HashSet::new();
    for source in sources {
        if !refs.insert(source.reference.as_str()) {
            return Err(invalid(format!(
                "duplicate source ref {}",
                source.reference
            )));
        }
        if SourceKind::parse(&source.kind).is_none() {
            return Err(invalid(format!("unknown source kind {}", source.kind)));
        }
        if Reliability::parse(&source.reliability).is_none() {
            return Err(invalid(format!(
                "unknown reliability {}",
                source.reliability
            )));
        }
        if source.title.trim().is_empty() {
            return Err(invalid(format!("source {} has no title", source.reference)));
        }
    }
    let parents: HashMap<&str, &str> = sources
        .iter()
        .filter_map(|s| {
            s.received_from
                .as_deref()
                .map(|parent| (s.reference.as_str(), parent))
        })
        .collect();
    for (child, parent) in &parents {
        if !refs.contains(parent) {
            return Err(invalid(format!(
                "{child} is received from unknown {parent}"
            )));
        }
        let mut current = *parent;
        let mut steps = 0;
        while let Some(next) = parents.get(current) {
            steps += 1;
            if *next == *child || steps > sources.len() {
                return Err(invalid(format!("source chain of {child} has a cycle")));
            }
            current = *next;
        }
    }
    Ok(())
}

fn validate_entities(entities: &[ResultEntity]) -> Result<(), ResultError> {
    let mut refs = HashSet::new();
    for entity in entities {
        if !refs.insert(entity.reference.as_str()) {
            return Err(invalid(format!(
                "duplicate entity ref {}",
                entity.reference
            )));
        }
        let kind = EntityKind::parse(&entity.kind)
            .ok_or_else(|| invalid(format!("unknown entity kind {}", entity.kind)))?;
        if kind == EntityKind::Person {
            if entity.label.is_some() {
                return Err(invalid(format!(
                    "person {} must not have a label",
                    entity.reference
                )));
            }
            match &entity.token {
                Some(token) if valid_token(token) => {}
                _ => {
                    return Err(invalid(format!(
                        "person {} needs a valid token",
                        entity.reference
                    )))
                }
            }
        } else if entity.label.as_deref().is_none_or(|l| l.trim().is_empty()) {
            return Err(invalid(format!(
                "entity {} needs a label",
                entity.reference
            )));
        }
    }
    Ok(())
}

fn validate_statements(file: &ResultFile) -> Result<(), ResultError> {
    let entity_refs: HashSet<&str> = file.entities.iter().map(|e| e.reference.as_str()).collect();
    let source_refs: HashSet<&str> = file.sources.iter().map(|s| s.reference.as_str()).collect();
    for (index, statement) in file.statements.iter().enumerate() {
        let at = index + 1;
        if StatementType::parse(&statement.kind).is_none() {
            return Err(invalid(format!(
                "statement {at}: unknown kind {}",
                statement.kind
            )));
        }
        if !(1..=6).contains(&statement.credibility) {
            return Err(invalid(format!("statement {at}: credibility must be 1..6")));
        }
        if statement.quote.trim().is_empty() {
            return Err(invalid(format!("statement {at}: a quote is required")));
        }
        if statement.parties.is_empty() || statement.sources.is_empty() {
            return Err(invalid(format!(
                "statement {at}: parties and sources are required"
            )));
        }
        for party in &statement.parties {
            if !entity_refs.contains(party.entity.as_str()) {
                return Err(invalid(format!(
                    "statement {at}: unknown entity {}",
                    party.entity
                )));
            }
            if party.role.trim().is_empty() {
                return Err(invalid(format!("statement {at}: a party has no role")));
            }
        }
        for source in &statement.sources {
            if !source_refs.contains(source.as_str()) {
                return Err(invalid(format!("statement {at}: unknown source {source}")));
            }
        }
    }
    Ok(())
}

/// Check a parsed result against the rules of the model (ADR-0006).
pub fn validate(file: &ResultFile) -> Result<(), ResultError> {
    validate_sources(&file.sources)?;
    validate_entities(&file.entities)?;
    validate_statements(file)
}

/// Parse and validate the AI's `result.json` for the expected package.
pub fn parse_result(json: &str, expected_package: &str) -> Result<ResultFile, ResultError> {
    let file: ResultFile =
        serde_json::from_str(json).map_err(|error| ResultError::Json(error.to_string()))?;
    if file.package != expected_package {
        return Err(ResultError::WrongPackage {
            expected: expected_package.to_string(),
            found: file.package,
        });
    }
    validate(&file)?;
    Ok(file)
}

/// The text the investigator reads before confirming the import.
pub fn review_markdown(file: &ResultFile) -> String {
    let mut out = format!("# Вичитка результату розбору {}\n\n", file.package);
    out.push_str(&format!(
        "Джерел: {}; суб'єктів: {}; тверджень: {}. Нічого ще не записано в базу.\n\n",
        file.sources.len(),
        file.entities.len(),
        file.statements.len()
    ));
    let label_of = |reference: &str| -> String {
        file.entities
            .iter()
            .find(|e| e.reference == reference)
            .map(|e| match (&e.label, &e.token) {
                (Some(label), _) => label.clone(),
                (None, Some(token)) => token.clone(),
                _ => reference.to_string(),
            })
            .unwrap_or_else(|| reference.to_string())
    };
    out.push_str("## Твердження\n\n");
    for (index, statement) in file.statements.iter().enumerate() {
        let parties: Vec<String> = statement
            .parties
            .iter()
            .map(|p| format!("{} ({})", label_of(&p.entity), p.role))
            .collect();
        out.push_str(&format!(
            "{}. **{}** {} - {}; достовірність (пропозиція) {}\n   Цитата: \"{}\"\n",
            index + 1,
            statement.kind,
            statement.value.as_deref().unwrap_or(""),
            parties.join(", "),
            statement.credibility,
            statement.quote
        ));
        if let Some(note) = &statement.note {
            out.push_str(&format!("   Примітка: {note}\n"));
        }
    }
    out.push_str("\n## Джерела\n\n");
    for source in &file.sources {
        out.push_str(&format!(
            "- {} [{}] {} - надійність (пропозиція) {}\n",
            source.reference, source.kind, source.title, source.reliability
        ));
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    const TOKEN: &str = "N-DNNKSHBLI2EYTBLZ3HCPEVW4OQ";

    fn sample() -> String {
        format!(
            r#"{{
  "package": "pkg-1",
  "sources": [
    {{"ref": "s1", "kind": "media", "title": "Стаття", "url": null,
      "published_at": "2026-10-01", "reliability": "C", "received_from": null}}
  ],
  "entities": [
    {{"ref": "e1", "kind": "org", "label": "ТОВ Приклад", "token": null, "public_code": "12345678"}},
    {{"ref": "e2", "kind": "person", "label": null, "token": "{TOKEN}", "public_code": null}}
  ],
  "statements": [
    {{"kind": "owns", "value": "40%", "valid_from": null, "valid_to": null, "as_of": null,
      "parties": [{{"entity": "e2", "role": "owner"}}, {{"entity": "e1", "role": "owned"}}],
      "sources": ["s1"], "credibility": 3, "quote": "володіє 40 відсотками", "note": null}}
  ]
}}"#
        )
    }

    fn mutate(from: &str, to: &str) -> String {
        let text = sample();
        assert!(text.contains(from), "fixture lacks {from}");
        text.replace(from, to)
    }

    fn rejected(json: &str) -> bool {
        parse_result(json, "pkg-1").is_err()
    }

    #[test]
    fn a_valid_result_is_accepted() {
        let file = parse_result(&sample(), "pkg-1").unwrap();
        assert_eq!(file.statements.len(), 1);
        assert_eq!(file.entities[1].token.as_deref(), Some(TOKEN));
    }

    #[test]
    fn the_package_must_match() {
        assert!(matches!(
            parse_result(&sample(), "pkg-2"),
            Err(ResultError::WrongPackage { .. })
        ));
    }

    #[test]
    fn broken_json_and_unknown_fields_are_rejected() {
        assert!(matches!(
            parse_result("{", "pkg-1"),
            Err(ResultError::Json(_))
        ));
        assert!(rejected(&mutate(
            "\"credibility\": 3,",
            "\"credibility\": 3, \"extra\": 1,"
        )));
    }

    #[test]
    fn a_person_needs_a_token_and_no_label() {
        assert!(rejected(&mutate(
            "\"label\": null, \"token\": \"",
            "\"label\": \"Ім'я\", \"token\": \""
        )));
        assert!(rejected(&mutate(TOKEN, "not-a-token")));
    }

    #[test]
    fn an_organisation_needs_a_label() {
        assert!(rejected(&mutate(
            "\"label\": \"ТОВ Приклад\"",
            "\"label\": null"
        )));
    }

    #[test]
    fn kinds_and_grades_are_checked() {
        assert!(rejected(&mutate(
            "\"kind\": \"owns\"",
            "\"kind\": \"loves\""
        )));
        assert!(rejected(&mutate(
            "\"kind\": \"media\"",
            "\"kind\": \"gossip\""
        )));
        assert!(rejected(&mutate(
            "\"reliability\": \"C\"",
            "\"reliability\": \"Z\""
        )));
        assert!(rejected(&mutate(
            "\"credibility\": 3",
            "\"credibility\": 7"
        )));
        assert!(rejected(&mutate(
            "\"credibility\": 3",
            "\"credibility\": 0"
        )));
    }

    #[test]
    fn a_quote_is_required() {
        assert!(rejected(&mutate("володіє 40 відсотками", "  ")));
    }

    #[test]
    fn references_must_resolve() {
        assert!(rejected(&mutate(
            "\"entity\": \"e2\"",
            "\"entity\": \"e9\""
        )));
        assert!(rejected(&mutate(
            "\"sources\": [\"s1\"]",
            "\"sources\": [\"s9\"]"
        )));
        assert!(rejected(&mutate(
            "\"sources\": [\"s1\"]",
            "\"sources\": []"
        )));
    }

    #[test]
    fn duplicate_refs_are_rejected() {
        assert!(rejected(&mutate("\"ref\": \"e2\"", "\"ref\": \"e1\"")));
    }

    #[test]
    fn source_chains_must_exist_and_not_loop() {
        assert!(rejected(&mutate(
            "\"received_from\": null",
            "\"received_from\": \"s9\""
        )));
        assert!(rejected(&mutate(
            "\"received_from\": null",
            "\"received_from\": \"s1\""
        )));
    }

    #[test]
    fn the_review_shows_quotes_and_labels() {
        let file = parse_result(&sample(), "pkg-1").unwrap();
        let review = review_markdown(&file);
        assert!(review.contains("володіє 40 відсотками"));
        assert!(review.contains("ТОВ Приклад (owned)"));
        assert!(review.contains(TOKEN));
        assert!(review.contains("Нічого ще не записано"));
    }
}
