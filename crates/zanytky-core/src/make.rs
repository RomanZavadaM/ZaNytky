//! Making a statement by hand from a source (the "Sources" tab): parties are existing
//! entities or new ones. A new person is given by name: the name goes to the registry (sealed)
//! and the person is kept in the database under the pseudonym only. The statement is always
//! unverified and carries the quote from the source.

use serde::Deserialize;

use crate::model::{
    EntityKind, Model, ModelError, NewEntity, NewSource, NewStatement, Reliability, SourceKind,
    StatementType, Status,
};
use crate::normalize::Kind;
use crate::registry::Registry;

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PartyRequest {
    pub role: String,
    /// An existing entity ...
    pub entity_id: Option<i64>,
    /// ... or a new one: kind, and a name (for a person: the full name) or a label.
    pub kind: Option<String>,
    pub label: Option<String>,
    pub public_code: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StatementRequest {
    pub source_id: i64,
    pub kind: String,
    pub value: Option<String>,
    pub credibility: u8,
    pub quote: String,
    pub note: Option<String>,
    pub parties: Vec<PartyRequest>,
}

/// Statements from a JSON file: a list of requests (the field names are in camelCase).
pub fn parse_statement_requests(text: &str) -> Result<Vec<StatementRequest>, String> {
    serde_json::from_str(text)
        .map_err(|e| format!("файл не є коректним списком тверджень: {e}"))
}

/// An evidence for a statement: a document (a new source) attached to it, and optionally the
/// new status the evidence settles.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EvidenceRequest {
    pub statement_id: i64,
    pub kind: String,
    pub reliability: String,
    pub title: String,
    pub url: Option<String>,
    pub published_at: Option<String>,
    pub text: Option<String>,
    pub note: Option<String>,
    /// `confirmed` or `refuted`.
    pub status: Option<String>,
}

/// Attach an evidence to a statement; returns the id of its source. A source with the same
/// address is reused. A statement is confirmed only by a registry or an official statement.
pub fn add_evidence(model: &Model, request: &EvidenceRequest) -> Result<i64, String> {
    let kind = SourceKind::parse(&request.kind).ok_or("невідомий вид джерела")?;
    let reliability = Reliability::parse(&request.reliability).ok_or("невідома надійність")?;
    let status = match request.status.as_deref() {
        None | Some("") => None,
        Some("confirmed") => Some(Status::Confirmed),
        Some("refuted") => Some(Status::Refuted),
        Some(_) => return Err("невідомий статус".to_string()),
    };
    if status == Some(Status::Confirmed)
        && !matches!(kind, SourceKind::Registry | SourceKind::Authority)
    {
        return Err("підтвердити можна лише доказом виду «реєстр» або «заява органу»".to_string());
    }
    let title = request.title.trim();
    if title.is_empty() {
        return Err("потрібна назва документа".to_string());
    }
    model
        .in_transaction(|model| {
            let url = clean(&request.url);
            let existing = match url {
                Some(address) => model.find_source_by_url(address)?,
                None => None,
            };
            let source_id = match existing {
                Some(id) => id,
                None => model.add_source(&NewSource {
                    kind,
                    title,
                    url,
                    published_at: clean(&request.published_at),
                    reliability,
                    parent_id: None,
                })?,
            };
            if let Some(text) = clean(&request.text) {
                if model.source_text(source_id)?.is_none() {
                    model.set_source_text(source_id, text)?;
                }
            }
            model.add_statement_source(request.statement_id, source_id)?;
            if let Some(note) = clean(&request.note) {
                model.append_statement_note(
                    request.statement_id,
                    &format!("Доказ ({title}): {note}"),
                )?;
            }
            if let Some(status) = status {
                model.set_status(request.statement_id, status)?;
            }
            Ok(source_id)
        })
        .map_err(|e: ModelError| e.to_string())
}

fn clean(text: &Option<String>) -> Option<&str> {
    text.as_deref().map(str::trim).filter(|t| !t.is_empty())
}

/// What a party needs before the database is touched.
enum Prepared {
    Existing(i64),
    Person {
        token: String,
    },
    Other {
        kind: EntityKind,
        label: String,
        code: Option<String>,
    },
}

/// Create the statement; returns its id. `registry` is needed only when a new person is named.
pub fn create_statement(
    model: &Model,
    registry: Option<&Registry>,
    request: &StatementRequest,
) -> Result<i64, String> {
    let kind = StatementType::parse(&request.kind).ok_or("невідомий вид твердження")?;
    let quote = request.quote.trim();
    if quote.is_empty() {
        return Err("потрібна цитата з джерела".to_string());
    }
    if request.parties.len() < 2 {
        return Err("потрібні щонайменше дві сторони".to_string());
    }
    let mut prepared = Vec::new();
    for party in &request.parties {
        if party.role.trim().is_empty() {
            return Err("у кожної сторони має бути роль".to_string());
        }
        if let Some(id) = party.entity_id {
            model.entity(id).map_err(|e| e.to_string())?;
            prepared.push(Prepared::Existing(id));
            continue;
        }
        let entity_kind = party
            .kind
            .as_deref()
            .and_then(EntityKind::parse)
            .ok_or("невідомий вид суб'єкта")?;
        let label = clean(&party.label).ok_or("потрібна назва або ім'я суб'єкта")?;
        if entity_kind == EntityKind::Person {
            let registry = registry.ok_or("для особи потрібні ключі в папці даних")?;
            let token = registry
                .register(Kind::Name, label)
                .map_err(|e| e.to_string())?;
            prepared.push(Prepared::Person { token });
        } else {
            prepared.push(Prepared::Other {
                kind: entity_kind,
                label: label.to_string(),
                code: clean(&party.public_code).map(str::to_string),
            });
        }
    }

    let statement = model
        .in_transaction(|model| {
            let mut parties: Vec<(i64, &str)> = Vec::new();
            for (item, party) in prepared.iter().zip(&request.parties) {
                let id = match item {
                    Prepared::Existing(id) => *id,
                    Prepared::Person { token } => {
                        match model.find_entity(EntityKind::Person, Some(token), None)? {
                            Some(id) => id,
                            None => model.add_entity(&NewEntity {
                                kind: EntityKind::Person,
                                token: Some(token),
                                label: None,
                                public_code: None,
                            })?,
                        }
                    }
                    Prepared::Other { kind, label, code } => {
                        reuse_or_add(model, *kind, label, code.as_deref())?
                    }
                };
                parties.push((id, party.role.trim()));
            }
            let note = match clean(&request.note) {
                Some(extra) => format!("Цитата: {quote}\nПримітка: {extra}"),
                None => format!("Цитата: {quote}"),
            };
            model.add_statement(&NewStatement {
                kind,
                value: clean(&request.value),
                valid_from: None,
                valid_to: None,
                as_of: None,
                credibility: request.credibility,
                note: Some(note.as_str()),
                parties: &parties,
                sources: &[request.source_id],
            })
        })
        .map_err(|e: ModelError| e.to_string())?;
    Ok(statement)
}

/// The entity for a name: an existing one (same code, same pseudonym, the only one with this
/// name) or a new one. A person is given by name; it is sealed in the registry and only the
/// pseudonym is kept in the base.
pub fn ensure_entity(
    model: &Model,
    registry: Option<&Registry>,
    kind: EntityKind,
    label: &str,
    code: Option<&str>,
) -> Result<i64, String> {
    let label = label.trim();
    if label.is_empty() {
        return Err("потрібна назва або ім'я суб'єкта".to_string());
    }
    if kind == EntityKind::Person {
        let registry = registry.ok_or("для особи потрібні ключі в папці даних")?;
        let token = registry
            .register(Kind::Name, label)
            .map_err(|e| e.to_string())?;
        return model
            .in_transaction(|model| {
                match model.find_entity(EntityKind::Person, Some(&token), None)? {
                    Some(id) => Ok(id),
                    None => model.add_entity(&NewEntity {
                        kind: EntityKind::Person,
                        token: Some(&token),
                        label: None,
                        public_code: None,
                    }),
                }
            })
            .map_err(|e| e.to_string());
    }
    model
        .in_transaction(|model| reuse_or_add(model, kind, label, code))
        .map_err(|e| e.to_string())
}

/// The same code, or the only entity of this kind with this name, or a new one.
fn reuse_or_add(
    model: &Model,
    kind: EntityKind,
    label: &str,
    code: Option<&str>,
) -> Result<i64, ModelError> {
    if let Some(code) = code {
        if let Some(id) = model.find_entity(kind, None, Some(code))? {
            model.add_alias(id, label)?;
            return Ok(id);
        }
    }
    let mut same = model.entities_by_label(kind, label)?;
    if code.is_some() {
        same.retain(|id| model.entity(*id).is_ok_and(|e| e.public_code.is_none()));
    }
    if same.len() == 1 {
        let id = same[0];
        if let Some(code) = code {
            model.set_public_code(id, code)?;
        }
        return Ok(id);
    }
    model.add_entity(&NewEntity {
        kind,
        token: None,
        label: Some(label),
        public_code: code,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{NewSource, Reliability, SourceKind};

    fn party(role: &str, label: &str, code: Option<&str>) -> PartyRequest {
        PartyRequest {
            role: role.to_string(),
            entity_id: None,
            kind: Some("org".to_string()),
            label: Some(label.to_string()),
            public_code: code.map(str::to_string),
        }
    }

    fn request(source_id: i64, parties: Vec<PartyRequest>) -> StatementRequest {
        StatementRequest {
            source_id,
            kind: "owns".to_string(),
            value: Some("40%".to_string()),
            credibility: 4,
            quote: "володіє 40 відсотками".to_string(),
            note: None,
            parties,
        }
    }

    fn get(model: &Model, id: i64) -> crate::model::StatementRow {
        model
            .statements()
            .unwrap()
            .into_iter()
            .find(|s| s.id == id)
            .unwrap()
    }

    fn setup() -> (Model, i64) {
        let model = Model::open_in_memory("tester").unwrap();
        let source = model
            .add_source(&NewSource {
                kind: SourceKind::Media,
                title: "Допис",
                url: None,
                published_at: None,
                reliability: Reliability::F,
                parent_id: None,
            })
            .unwrap();
        (model, source)
    }

    #[test]
    fn a_statement_is_made_with_new_entities_and_reuses_them_next_time() {
        let (model, source) = setup();
        let first = create_statement(
            &model,
            None,
            &request(
                source,
                vec![
                    party("owner", "ТОВ Власник", Some("12345678")),
                    party("owned", "ТОВ «Об'єкт»", None),
                ],
            ),
        )
        .unwrap();
        let row = get(&model, first);
        assert_eq!(row.sources, vec![source]);
        assert_eq!(row.parties.len(), 2);
        assert!(row.note.as_ref().unwrap().starts_with("Цитата: володіє"));

        // the same names and the same code: no new entities
        let second = create_statement(
            &model,
            None,
            &request(
                source,
                vec![
                    party("owner", "Власник ТОВ", Some("12345678")),
                    party("owned", "Об'єкт ТОВ", None),
                ],
            ),
        )
        .unwrap();
        let again = get(&model, second);
        let ids = |r: &crate::model::StatementRow| {
            let mut v: Vec<i64> = r.parties.iter().map(|p| p.0).collect();
            v.sort();
            v
        };
        assert_eq!(ids(&row), ids(&again));
    }

    #[test]
    fn a_statement_needs_a_quote_two_parties_roles_and_a_known_person_route() {
        let (model, source) = setup();
        let parties = vec![party("owner", "А", None), party("owned", "Б", None)];

        let mut no_quote = request(source, parties.clone());
        no_quote.quote = "  ".to_string();
        assert!(create_statement(&model, None, &no_quote).is_err());

        assert!(create_statement(&model, None, &request(source, parties[..1].to_vec())).is_err());

        let mut no_role = request(source, parties.clone());
        no_role.parties[0].role = " ".to_string();
        assert!(create_statement(&model, None, &no_role).is_err());

        let mut person = request(source, parties.clone());
        person.parties[0].kind = Some("person".to_string());
        assert!(create_statement(&model, None, &person).is_err());

        let mut bad_source = request(999, parties);
        bad_source.credibility = 4;
        assert!(create_statement(&model, None, &bad_source).is_err());
        // a failed attempt leaves no half-made entities
        assert!(model
            .entities_by_label(EntityKind::Org, "А")
            .unwrap()
            .is_empty());
    }

    #[test]
    fn evidence_is_attached_and_only_a_document_confirms() {
        let (model, source) = setup();
        let id = create_statement(
            &model,
            None,
            &request(
                source,
                vec![party("owner", "ТОВ А", None), party("owned", "ТОВ Б", None)],
            ),
        )
        .unwrap();
        let evidence = |kind: &str, status: Option<&str>| EvidenceRequest {
            statement_id: id,
            kind: kind.to_string(),
            reliability: "A".to_string(),
            title: "Витяг з ЄДР".to_string(),
            url: Some("https://example.org/edr".to_string()),
            published_at: Some("2026-11-03".to_string()),
            text: Some("Засновники: ТОВ А - 100%".to_string()),
            note: Some("частка збігається".to_string()),
            status: status.map(str::to_string),
        };
        // a media article cannot confirm
        assert!(add_evidence(&model, &evidence("media", Some("confirmed"))).is_err());
        assert_eq!(model.status(id).unwrap(), Status::Unverified);
        // a registry extract can
        let source_id = add_evidence(&model, &evidence("registry", Some("confirmed"))).unwrap();
        assert_eq!(model.status(id).unwrap(), Status::Confirmed);
        let row = get(&model, id);
        assert!(row.sources.contains(&source_id));
        assert!(row
            .note
            .unwrap()
            .contains("Доказ (Витяг з ЄДР): частка збігається"));
        assert_eq!(
            model.source_text(source_id).unwrap().as_deref(),
            Some("Засновники: ТОВ А - 100%")
        );
        // the same address is the same source
        assert_eq!(
            add_evidence(&model, &evidence("registry", None)).unwrap(),
            source_id
        );
        // an unknown statement is refused
        let mut other = evidence("registry", None);
        other.statement_id = 9999;
        assert!(add_evidence(&model, &other).is_err());
    }
}
