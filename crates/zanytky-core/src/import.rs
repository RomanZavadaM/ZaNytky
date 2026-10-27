//! Import of a confirmed AI result into the model (ADR-0007, step 7).
//!
//! Everything arrives as `unverified`; the AI's quote is kept in the note of
//! the statement. The whole import is one transaction, and a package can be
//! imported only once.

use std::collections::HashMap;
use std::fmt;

use crate::model::{
    EntityKind, Model, ModelError, NewEntity, NewSource, NewStatement, Reliability, SourceKind,
    StatementType,
};
use crate::result::{validate, ResultError, ResultFile};

#[derive(Debug, PartialEq, Eq)]
pub enum ImportError {
    Result(ResultError),
    Model(ModelError),
    AlreadyImported(String),
}

impl fmt::Display for ImportError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ImportError::Result(error) => write!(f, "{error}"),
            ImportError::Model(error) => write!(f, "{error}"),
            ImportError::AlreadyImported(package) => {
                write!(f, "package {package} was already imported")
            }
        }
    }
}

impl std::error::Error for ImportError {}

impl From<ResultError> for ImportError {
    fn from(error: ResultError) -> Self {
        ImportError::Result(error)
    }
}

impl From<ModelError> for ImportError {
    fn from(error: ModelError) -> Self {
        ImportError::Model(error)
    }
}

/// What an import added.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ImportSummary {
    pub sources: usize,
    pub entities_new: usize,
    pub entities_existing: usize,
    pub statements: usize,
}

fn bad(message: &'static str) -> ModelError {
    ModelError::Invalid(message)
}

fn import_inside(model: &Model, file: &ResultFile) -> Result<ImportSummary, ModelError> {
    let mut summary = ImportSummary {
        sources: 0,
        entities_new: 0,
        entities_existing: 0,
        statements: 0,
    };

    // Sources: a source goes in after the one it was received from.
    let mut source_ids: HashMap<&str, i64> = HashMap::new();
    let mut pending: Vec<&crate::result::ResultSource> = file.sources.iter().collect();
    while !pending.is_empty() {
        let before = pending.len();
        let mut waiting = Vec::new();
        for source in pending {
            let parent = match source.received_from.as_deref() {
                Some(reference) => match source_ids.get(reference) {
                    Some(id) => Some(*id),
                    None => {
                        waiting.push(source);
                        continue;
                    }
                },
                None => None,
            };
            let existing = match &source.url {
                Some(url) => model.find_source_by_url(url)?,
                None => None,
            };
            let id = match existing {
                Some(id) => id,
                None => {
                    summary.sources += 1;
                    model.add_source(&NewSource {
                        kind: SourceKind::parse(&source.kind).ok_or(bad("source kind"))?,
                        title: &source.title,
                        url: source.url.as_deref(),
                        published_at: source.published_at.as_deref(),
                        reliability: Reliability::parse(&source.reliability)
                            .ok_or(bad("reliability"))?,
                        parent_id: parent,
                    })?
                }
            };
            source_ids.insert(source.reference.as_str(), id);
        }
        if waiting.len() == before {
            return Err(bad("source chain cannot be resolved"));
        }
        pending = waiting;
    }

    // Entities: reuse what is already known (same pseudonym or public code).
    let mut entity_ids: HashMap<&str, i64> = HashMap::new();
    for entity in &file.entities {
        let kind = EntityKind::parse(&entity.kind).ok_or(bad("entity kind"))?;
        let existing =
            model.find_entity(kind, entity.token.as_deref(), entity.public_code.as_deref())?;
        let id = match existing {
            Some(id) => {
                summary.entities_existing += 1;
                id
            }
            None => {
                summary.entities_new += 1;
                model.add_entity(&NewEntity {
                    kind,
                    token: entity.token.as_deref(),
                    label: entity.label.as_deref(),
                    public_code: entity.public_code.as_deref(),
                })?
            }
        };
        entity_ids.insert(entity.reference.as_str(), id);
    }

    // Statements: always unverified; the quote travels with the statement.
    for statement in &file.statements {
        let parties: Vec<(i64, &str)> = statement
            .parties
            .iter()
            .map(|party| {
                entity_ids
                    .get(party.entity.as_str())
                    .map(|id| (*id, party.role.as_str()))
                    .ok_or(bad("unknown entity"))
            })
            .collect::<Result<_, _>>()?;
        let sources: Vec<i64> = statement
            .sources
            .iter()
            .map(|reference| {
                source_ids
                    .get(reference.as_str())
                    .copied()
                    .ok_or(bad("unknown source"))
            })
            .collect::<Result<_, _>>()?;
        let note = match &statement.note {
            Some(note) => format!("Цитата: {}\nПримітка: {note}", statement.quote),
            None => format!("Цитата: {}", statement.quote),
        };
        model.add_statement(&NewStatement {
            kind: StatementType::parse(&statement.kind).ok_or(bad("statement kind"))?,
            value: statement.value.as_deref(),
            valid_from: statement.valid_from.as_deref(),
            valid_to: statement.valid_to.as_deref(),
            as_of: statement.as_of.as_deref(),
            credibility: statement.credibility,
            note: Some(note.as_str()),
            parties: &parties,
            sources: &sources,
        })?;
        summary.statements += 1;
    }

    model.log("import_result", &format!("package:{}", file.package))?;
    Ok(summary)
}

/// Write a validated result into the model, all or nothing.
pub fn import_result(model: &Model, file: &ResultFile) -> Result<ImportSummary, ImportError> {
    validate(file)?;
    if model.package_imported(&file.package)? {
        return Err(ImportError::AlreadyImported(file.package.clone()));
    }
    Ok(model.in_transaction(|model| import_inside(model, file))?)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::Status;
    use crate::result::parse_result;

    const TOKEN: &str = "N-DNNKSHBLI2EYTBLZ3HCPEVW4OQ";

    fn json(package: &str) -> String {
        format!(
            r#"{{
  "package": "{package}",
  "sources": [
    {{"ref": "s0", "kind": "registry", "title": "Реєстр", "url": "https://example.org/reg",
      "published_at": null, "reliability": "A", "received_from": null}},
    {{"ref": "s1", "kind": "media", "title": "Стаття", "url": null,
      "published_at": "2026-10-01", "reliability": "C", "received_from": "s0"}}
  ],
  "entities": [
    {{"ref": "e1", "kind": "org", "label": "ТОВ Приклад", "token": null, "public_code": "12345678"}},
    {{"ref": "e2", "kind": "person", "label": null, "token": "{TOKEN}", "public_code": null}}
  ],
  "statements": [
    {{"kind": "owns", "value": "40%", "valid_from": null, "valid_to": null, "as_of": null,
      "parties": [{{"entity": "e2", "role": "owner"}}, {{"entity": "e1", "role": "owned"}}],
      "sources": ["s1"], "credibility": 3, "quote": "володіє 40 відсотками", "note": "частка зі статті"}}
  ]
}}"#
        )
    }

    fn file(package: &str) -> ResultFile {
        parse_result(&json(package), package).unwrap()
    }

    #[test]
    fn an_import_adds_everything_as_unverified() {
        let model = Model::open_in_memory("tester").unwrap();
        let summary = import_result(&model, &file("pkg-1")).unwrap();
        assert_eq!(
            summary,
            ImportSummary {
                sources: 2,
                entities_new: 2,
                entities_existing: 0,
                statements: 1
            }
        );
        let person = model
            .find_entity(EntityKind::Person, Some(TOKEN), None)
            .unwrap()
            .unwrap();
        let company = model
            .find_entity(EntityKind::Org, None, Some("12345678"))
            .unwrap()
            .unwrap();
        let edges = model.ownership_up(company, 3).unwrap();
        assert_eq!(edges.len(), 1);
        assert_eq!(edges[0].owner, person);
        assert_eq!(edges[0].value.as_deref(), Some("40%"));
        assert_eq!(
            model.status(edges[0].statement_id).unwrap(),
            Status::Unverified
        );
        assert_eq!(
            model.grade(edges[0].statement_id).unwrap().to_string(),
            "C3"
        );
    }

    #[test]
    fn the_source_chain_is_kept_and_counts_one_origin() {
        let model = Model::open_in_memory("tester").unwrap();
        import_result(&model, &file("pkg-1")).unwrap();
        let s0 = model
            .find_source_by_url("https://example.org/reg")
            .unwrap()
            .unwrap();
        // the media source was received from the registry source
        assert_eq!(model.provenance(s0 + 1).unwrap(), vec![s0 + 1, s0]);
    }

    #[test]
    fn the_same_package_cannot_be_imported_twice() {
        let model = Model::open_in_memory("tester").unwrap();
        import_result(&model, &file("pkg-1")).unwrap();
        let again = import_result(&model, &file("pkg-1"));
        assert_eq!(
            again,
            Err(ImportError::AlreadyImported("pkg-1".to_string()))
        );
    }

    #[test]
    fn known_entities_and_sources_are_reused() {
        let model = Model::open_in_memory("tester").unwrap();
        import_result(&model, &file("pkg-1")).unwrap();
        let second = import_result(&model, &file("pkg-2")).unwrap();
        assert_eq!(second.entities_new, 0);
        assert_eq!(second.entities_existing, 2);
        // the registry source has an address and is reused; the media one has none
        assert_eq!(second.sources, 1);
        assert_eq!(second.statements, 1);
    }

    #[test]
    fn an_invalid_result_writes_nothing() {
        let model = Model::open_in_memory("tester").unwrap();
        let mut broken = file("pkg-1");
        broken.statements[0].credibility = 9;
        assert!(matches!(
            import_result(&model, &broken),
            Err(ImportError::Result(_))
        ));
        assert_eq!(model.log_len().unwrap(), 0);
    }

    #[test]
    fn the_quote_travels_with_the_statement_and_the_import_is_logged() {
        let model = Model::open_in_memory("tester").unwrap();
        import_result(&model, &file("pkg-1")).unwrap();
        assert!(model.package_imported("pkg-1").unwrap());
        assert!(!model.package_imported("pkg-2").unwrap());
        // 2 sources + 2 entities + 1 statement + 1 import entry
        assert_eq!(model.log_len().unwrap(), 6);
    }
}
