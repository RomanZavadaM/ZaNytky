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
use crate::result::{review_markdown, validate, ResultEntity, ResultError, ResultFile};

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

/// How an entity of a result relates to what the database already knows.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LinkMatch {
    /// Already known: its id and how it was recognised (pseudonym, code or name).
    Existing {
        id: i64,
        by: &'static str,
    },
    /// Several known entities carry this name; nothing is merged automatically.
    Ambiguous(Vec<i64>),
    New,
}

/// The same rule is used by the review and by the import. A pseudonym or a public code
/// decides first; without a code the name decides, but never against a different code.
pub fn match_entity(model: &Model, entity: &ResultEntity) -> Result<LinkMatch, ModelError> {
    let kind = EntityKind::parse(&entity.kind).ok_or(bad("entity kind"))?;
    if let Some(id) =
        model.find_entity(kind, entity.token.as_deref(), entity.public_code.as_deref())?
    {
        let by = if entity.token.is_some() {
            "псевдонімом"
        } else {
            "кодом"
        };
        return Ok(LinkMatch::Existing { id, by });
    }
    if kind != EntityKind::Person && entity.public_code.is_none() {
        if let Some(label) = entity.label.as_deref() {
            let ids = model.entities_by_label(kind, label)?;
            match ids.len() {
                0 => {}
                1 => {
                    return Ok(LinkMatch::Existing {
                        id: ids[0],
                        by: "назвою",
                    })
                }
                _ => return Ok(LinkMatch::Ambiguous(ids)),
            }
        }
    }
    Ok(LinkMatch::New)
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
        let id = match match_entity(model, entity)? {
            LinkMatch::Existing { id, by } => {
                summary.entities_existing += 1;
                if by == "назвою" {
                    if let Some(label) = entity.label.as_deref() {
                        model.add_alias(id, label)?;
                    }
                }
                id
            }
            LinkMatch::Ambiguous(_) | LinkMatch::New => {
                summary.entities_new += 1;
                model.add_entity(&NewEntity {
                    kind,
                    token: entity.token.as_deref(),
                    label: entity.label.as_deref(),
                    public_code: entity.public_code.as_deref(),
                })?
            }
        };
        for identifier in &entity.identifiers {
            model.add_identifier(id, &identifier.kind, &identifier.value)?;
        }
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

/// The review text plus what each entity of the result is linked to in the database:
/// new, the same as a known one, or a doubt ("is this the same one?").
pub fn review_with_links(file: &ResultFile, model: &Model) -> Result<String, ModelError> {
    let mut lines = String::new();
    let (mut new, mut known, mut doubt) = (0, 0, 0);
    for entity in &file.entities {
        let name = match (&entity.label, &entity.token) {
            (Some(label), _) => format!("«{label}»"),
            (None, Some(token)) => token.clone(),
            _ => entity.reference.clone(),
        };
        let head = format!("- {} {} ({})", entity.reference, name, entity.kind);
        let kind = EntityKind::parse(&entity.kind).ok_or(bad("entity kind"))?;
        let shown = |id: i64| -> Result<String, ModelError> {
            let row = model.entity(id)?;
            Ok(format!(
                "№{id} {}",
                row.label.or(row.token).unwrap_or_default()
            ))
        };
        match match_entity(model, entity)? {
            LinkMatch::Existing { id, by } => {
                known += 1;
                lines.push_str(&format!(
                    "{head}: збіг із наявним {} (за {by})\n",
                    shown(id)?
                ));
            }
            LinkMatch::Ambiguous(ids) => {
                doubt += 1;
                let list: Vec<String> = ids.iter().map(|id| format!("№{id}")).collect();
                lines.push_str(&format!(
                    "{head}: сумнів: таку назву має кілька наявних ({}); імпорт створить нового, об'єднайте пізніше\n",
                    list.join(", ")
                ));
            }
            LinkMatch::New => {
                let similar = match entity.label.as_deref() {
                    Some(label) if kind != EntityKind::Person => {
                        model.similar_entities(kind, label)?
                    }
                    _ => Vec::new(),
                };
                if similar.is_empty() {
                    new += 1;
                    lines.push_str(&format!("{head}: новий\n"));
                } else {
                    doubt += 1;
                    let list: Vec<String> = similar
                        .iter()
                        .map(|(id, label)| format!("№{id} «{label}»"))
                        .collect();
                    lines.push_str(&format!(
                        "{head}: новий, але є схожі наявні: {}; якщо це той самий, об'єднайте пізніше\n",
                        list.join(", ")
                    ));
                }
            }
        }
    }
    let mut out = review_markdown(file);
    out.push_str(&format!(
        "\n## Підв'язка до бази\n\nСуб'єктів: нових {new}, збігів {known}, сумнівів {doubt}.\n\n{lines}"
    ));
    Ok(out)
}

#[cfg(test)]
mod link_tests {
    use super::*;
    use crate::result::parse_result;

    fn org_result(package: &str, label: &str, code: Option<&str>) -> String {
        let code = match code {
            Some(code) => format!("\"{code}\""),
            None => "null".to_string(),
        };
        format!(
            r#"{{
  "package": "{package}",
  "sources": [{{"ref": "s1", "kind": "media", "title": "Стаття", "url": null,
    "published_at": null, "reliability": "C", "received_from": null}}],
  "entities": [
    {{"ref": "e1", "kind": "org", "label": "{label}", "token": null, "public_code": {code}}},
    {{"ref": "e2", "kind": "org", "label": "Міська рада", "token": null, "public_code": null}}
  ],
  "statements": [{{"kind": "customer", "value": null, "valid_from": null, "valid_to": null,
    "as_of": null,
    "parties": [{{"entity": "e2", "role": "customer"}}, {{"entity": "e1", "role": "supplier"}}],
    "sources": ["s1"], "credibility": 3, "quote": "замовник - міська рада", "note": null}}]
}}"#
        )
    }

    fn import(model: &Model, package: &str, label: &str, code: Option<&str>) -> ImportSummary {
        let file = parse_result(&org_result(package, label, code), package).unwrap();
        import_result(model, &file).unwrap()
    }

    fn with_address(package: &str, label: &str, code: &str, identifier: &str) -> String {
        format!(
            r#"{{
  "package": "{package}",
  "sources": [{{"ref": "s1", "kind": "media", "title": "Стаття", "url": null,
    "published_at": null, "reliability": "C", "received_from": null}}],
  "entities": [
    {{"ref": "e1", "kind": "org", "label": "{label}", "token": null, "public_code": "{code}",
      "identifiers": [{identifier}]}}
  ],
  "statements": []
}}"#
        )
    }

    #[test]
    fn two_organisations_at_one_address_stay_apart_but_share_the_sign() {
        let model = Model::open_in_memory("t").unwrap();
        let address = r#"{"kind": "address", "value": "A-ABCDEFGHIJKLMNOPQRSTUVWXYZ"}"#;
        for (package, label, code) in [("p1", "Перша", "11111111"), ("p2", "Друга", "22222222")]
        {
            let file = parse_result(&with_address(package, label, code, address), package).unwrap();
            import_result(&model, &file).unwrap();
        }
        assert_eq!(model.entities().unwrap().len(), 2);
        let shared = model.shared_identifiers().unwrap();
        assert_eq!(shared.len(), 1);
        assert_eq!(shared[0].0, "address");
        assert_eq!(shared[0].2.len(), 2);
    }

    #[test]
    fn a_bad_identifier_is_rejected() {
        let bad = r#"{"kind": "address", "value": "вулиця Приклад 1"}"#;
        assert!(parse_result(&with_address("p1", "Перша", "11111111", bad), "p1").is_err());
        let unknown = r#"{"kind": "shoe", "value": "A-ABCDEFGHIJKLMNOPQRSTUVWXYZ"}"#;
        assert!(parse_result(&with_address("p1", "Перша", "11111111", unknown), "p1").is_err());
    }

    #[test]
    fn the_same_organisation_named_differently_is_linked_not_duplicated() {
        let model = Model::open_in_memory("t").unwrap();
        let first = import(&model, "pkg-1", "ТОВ «Приклад»", None);
        assert_eq!((first.entities_new, first.entities_existing), (2, 0));
        let second = import(&model, "pkg-2", "Приклад ТОВ", None);
        assert_eq!((second.entities_new, second.entities_existing), (0, 2));
    }

    #[test]
    fn different_names_are_not_merged() {
        let model = Model::open_in_memory("t").unwrap();
        import(&model, "pkg-1", "ТОВ «Приклад»", None);
        let second = import(&model, "pkg-2", "ТОВ «Будмайстер»", None);
        assert_eq!((second.entities_new, second.entities_existing), (1, 1));
    }

    #[test]
    fn a_different_code_is_never_merged_by_name() {
        let model = Model::open_in_memory("t").unwrap();
        import(&model, "pkg-1", "ТОВ «Приклад»", Some("12345678"));
        let second = import(&model, "pkg-2", "ТОВ «Приклад»", Some("87654321"));
        assert_eq!((second.entities_new, second.entities_existing), (1, 1));
    }

    #[test]
    fn a_later_mention_without_a_code_joins_the_coded_organisation() {
        let model = Model::open_in_memory("t").unwrap();
        import(&model, "pkg-1", "ТОВ «Приклад»", Some("12345678"));
        let second = import(&model, "pkg-2", "Приклад", None);
        assert_eq!((second.entities_new, second.entities_existing), (0, 2));
    }

    #[test]
    fn the_review_shows_what_is_linked() {
        let model = Model::open_in_memory("t").unwrap();
        import(&model, "pkg-1", "ТОВ «Приклад»", None);
        let file = parse_result(&org_result("pkg-2", "Приклад ТОВ", None), "pkg-2").unwrap();
        let review = review_with_links(&file, &model).unwrap();
        assert!(review.contains("нових 0, збігів 2, сумнівів 0"));
        assert!(review.contains("збіг із наявним"));
    }

    #[test]
    fn a_similar_name_is_shown_as_a_doubt_and_imported_as_new() {
        let model = Model::open_in_memory("t").unwrap();
        import(&model, "pkg-1", "ТОВ «Приклад»", None);
        let file = parse_result(&org_result("pkg-2", "ТОВ «Приклад Плюс»", None), "pkg-2").unwrap();
        let review = review_with_links(&file, &model).unwrap();
        assert!(review.contains("сумнівів 1"));
        assert!(review.contains("є схожі наявні"));
        let summary = import_result(&model, &file).unwrap();
        assert_eq!((summary.entities_new, summary.entities_existing), (1, 1));
    }
}
