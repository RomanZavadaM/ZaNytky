//! Making a statement by hand from a source (the "Sources" tab): parties are existing
//! entities or new ones. A new person is given by name: the name goes to the registry (sealed)
//! and the person is kept in the database under the pseudonym only. The statement is always
//! unverified and carries the quote from the source.

use serde::Deserialize;

use crate::model::{EntityKind, Model, ModelError, NewEntity, NewStatement, StatementType};
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

fn clean(text: &Option<String>) -> Option<&str> {
    text.as_deref().map(str::trim).filter(|t| !t.is_empty())
}

/// What a party needs before the database is touched.
enum Prepared {
    Existing(i64),
    Person { token: String },
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
        assert!(row.note.unwrap().starts_with("Цитата: володіє"));

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
        assert!(model.entities_by_label(EntityKind::Org, "А").unwrap().is_empty());
    }
}
