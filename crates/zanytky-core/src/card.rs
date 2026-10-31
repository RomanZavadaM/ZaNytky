//! The card of an entity: everything the database knows about it, in one place
//! (the "Entities" tab): statements with roles, sources and grades, neighbours, and the
//! ownership chain upwards. Persons are shown by pseudonym unless a map of names
//! (pseudonym -> name, opened with the investigator's key) is given; nothing is stored.

use std::collections::{BTreeMap, HashMap, HashSet};

use serde::Serialize;

use crate::model::{EntityRow, Model, ModelError};

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EntityListRow {
    pub id: i64,
    pub kind: String,
    pub label: String,
    pub code: Option<String>,
    /// How many statements mention the entity.
    pub statements: u32,
    /// With how many other entities it shares an address, phone, plate, account or card.
    pub shared: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CardParty {
    pub id: i64,
    pub label: String,
    pub role: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CardSource {
    pub title: String,
    pub kind: String,
    pub reliability: String,
    pub url: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CardStatement {
    pub id: i64,
    pub kind: String,
    /// The role of this entity in the statement.
    pub role: String,
    pub value: String,
    pub valid_from: Option<String>,
    pub valid_to: Option<String>,
    pub as_of: Option<String>,
    pub grade: String,
    pub status: String,
    pub note: String,
    /// Independent origins of the sources (echoes of one rumour count once).
    pub independent: u32,
    /// The other parties of the statement.
    pub parties: Vec<CardParty>,
    pub sources: Vec<CardSource>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CardNeighbor {
    pub id: i64,
    pub label: String,
    pub kind: String,
    /// In how many statements it stands next to this entity.
    pub count: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CardOwner {
    pub depth: u32,
    pub owner: String,
    pub owner_id: i64,
    pub owned: String,
    pub owned_id: i64,
    pub value: String,
    pub statement: i64,
}

/// An identifier that this entity has in common with others.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SharedSign {
    /// address, phone, email, plate, iban or card
    pub kind: String,
    /// The pseudonym of the identifier.
    pub value: String,
    pub others: Vec<CardNeighbor>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EntityCard {
    pub id: i64,
    pub kind: String,
    pub label: String,
    pub code: Option<String>,
    pub statements: Vec<CardStatement>,
    pub neighbors: Vec<CardNeighbor>,
    pub owners: Vec<CardOwner>,
    /// Identifiers shared with other entities: always shown, they are a strong sign.
    pub shared: Vec<SharedSign>,
}

fn display(row: &EntityRow, names: &HashMap<String, String>) -> String {
    if let Some(label) = &row.label {
        return label.clone();
    }
    if let Some(token) = &row.token {
        return names.get(token).cloned().unwrap_or_else(|| token.clone());
    }
    format!("№{}", row.id)
}

pub(crate) fn label_of(
    model: &Model,
    names: &HashMap<String, String>,
    cache: &mut HashMap<i64, String>,
    id: i64,
) -> Result<String, ModelError> {
    if let Some(label) = cache.get(&id) {
        return Ok(label.clone());
    }
    let label = display(&model.entity(id)?, names);
    cache.insert(id, label.clone());
    Ok(label)
}

/// Every entity with the number of statements about it.
pub fn entity_list(
    model: &Model,
    names: &HashMap<String, String>,
) -> Result<Vec<EntityListRow>, ModelError> {
    let mut counts: HashMap<i64, u32> = HashMap::new();
    for statement in model.statements()? {
        let mut seen = HashSet::new();
        for (entity, _) in &statement.parties {
            if seen.insert(*entity) {
                *counts.entry(*entity).or_insert(0) += 1;
            }
        }
    }
    let mut sharing: HashMap<i64, HashSet<i64>> = HashMap::new();
    for (_, _, holders) in model.shared_identifiers()? {
        for a in &holders {
            for b in &holders {
                if a != b {
                    sharing.entry(*a).or_default().insert(*b);
                }
            }
        }
    }
    let mut out = Vec::new();
    for row in model.entities()? {
        out.push(EntityListRow {
            id: row.id,
            kind: row.kind.as_str().to_string(),
            label: display(&row, names),
            code: row.public_code.clone(),
            statements: counts.get(&row.id).copied().unwrap_or(0),
            shared: sharing
                .get(&row.id)
                .map_or(0, |set| u32::try_from(set.len()).unwrap_or(u32::MAX)),
        });
    }
    Ok(out)
}

/// The card of one entity.
pub fn entity_card(
    model: &Model,
    id: i64,
    names: &HashMap<String, String>,
) -> Result<EntityCard, ModelError> {
    let row = model.entity(id)?;
    let mut cache: HashMap<i64, String> = HashMap::new();
    let own_label = label_of(model, names, &mut cache, id)?;
    let mut statements = Vec::new();
    let mut neighbors: BTreeMap<i64, CardNeighbor> = BTreeMap::new();
    for statement in model.statements()? {
        let mine: Vec<&str> = statement
            .parties
            .iter()
            .filter(|(entity, _)| *entity == id)
            .map(|(_, role)| role.as_str())
            .collect();
        if mine.is_empty() {
            continue;
        }
        let mut parties = Vec::new();
        for (entity, role) in &statement.parties {
            if *entity == id {
                continue;
            }
            let label = label_of(model, names, &mut cache, *entity)?;
            let kind = model.entity(*entity)?.kind.as_str().to_string();
            neighbors
                .entry(*entity)
                .or_insert_with(|| CardNeighbor {
                    id: *entity,
                    label: label.clone(),
                    kind,
                    count: 0,
                })
                .count += 1;
            parties.push(CardParty {
                id: *entity,
                label,
                role: role.clone(),
            });
        }
        let mut sources = Vec::new();
        for source_id in &statement.sources {
            let source = model.source(*source_id)?;
            sources.push(CardSource {
                title: source.title,
                kind: source.kind.as_str().to_string(),
                reliability: source.reliability.as_str().to_string(),
                url: source.url,
            });
        }
        statements.push(CardStatement {
            id: statement.id,
            kind: statement.kind.as_str().to_string(),
            role: mine.join(", "),
            value: statement.value.clone().unwrap_or_default(),
            valid_from: statement.valid_from.clone(),
            valid_to: statement.valid_to.clone(),
            as_of: statement.as_of.clone(),
            grade: model.grade(statement.id)?.to_string(),
            status: statement.status.as_str().to_string(),
            note: statement.note.clone().unwrap_or_default(),
            independent: model.independent_origins(statement.id)?,
            parties,
            sources,
        });
    }
    let mut owners = Vec::new();
    for edge in model.ownership_up(id, 5)? {
        owners.push(CardOwner {
            depth: edge.depth,
            owner: label_of(model, names, &mut cache, edge.owner)?,
            owner_id: edge.owner,
            owned: label_of(model, names, &mut cache, edge.owned)?,
            owned_id: edge.owned,
            value: edge.value.unwrap_or_default(),
            statement: edge.statement_id,
        });
    }
    let mut shared = Vec::new();
    for (kind, value) in model.identifiers_of(id)? {
        let mut others = Vec::new();
        for holder in model.identifier_holders(&kind, &value)? {
            if holder == id {
                continue;
            }
            others.push(CardNeighbor {
                id: holder,
                label: label_of(model, names, &mut cache, holder)?,
                kind: model.entity(holder)?.kind.as_str().to_string(),
                count: 1,
            });
        }
        if !others.is_empty() {
            shared.push(SharedSign {
                kind,
                value,
                others,
            });
        }
    }
    let mut neighbors: Vec<CardNeighbor> = neighbors.into_values().collect();
    neighbors.sort_by(|a, b| b.count.cmp(&a.count).then_with(|| a.label.cmp(&b.label)));
    Ok(EntityCard {
        id,
        kind: row.kind.as_str().to_string(),
        label: own_label,
        code: row.public_code,
        statements,
        neighbors,
        owners,
        shared,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::demo::create_demo;

    fn demo_model() -> (tempfile::TempDir, Model) {
        let dir = tempfile::tempdir().unwrap();
        let home = dir.path().join("demo");
        create_demo(&home).unwrap();
        let model = Model::open(&home.join("model.db"), "test").unwrap();
        (dir, model)
    }

    #[test]
    fn the_list_has_every_entity_with_its_statement_count() {
        let (_dir, model) = demo_model();
        let list = entity_list(&model, &HashMap::new()).unwrap();
        assert_eq!(list.len(), 7);
        let company = list.iter().find(|r| r.label == "ТОВ «Приклад»").unwrap();
        assert_eq!(company.statements, 6);
        assert_eq!(company.code.as_deref(), Some("12345678"));
    }

    #[test]
    fn a_company_card_shows_statements_neighbours_and_owners() {
        let (_dir, model) = demo_model();
        let list = entity_list(&model, &HashMap::new()).unwrap();
        let id = list.iter().find(|r| r.label == "ТОВ «Приклад»").unwrap().id;
        let card = entity_card(&model, id, &HashMap::new()).unwrap();
        assert_eq!(card.kind, "org");
        assert_eq!(card.statements.len(), 6);
        assert_eq!(card.neighbors.len(), 5);
        assert_eq!(card.owners.len(), 2);
        let owns: Vec<&str> = card.owners.iter().map(|o| o.value.as_str()).collect();
        assert!(owns.contains(&"40%") && owns.contains(&"60%"));
        let won = card.statements.iter().find(|s| s.kind == "won").unwrap();
        assert_eq!(won.value, "1 500 000 грн");
        assert_eq!(won.sources.len(), 1);
        assert_eq!(won.sources[0].reliability, "C");
    }

    #[test]
    fn a_person_is_shown_by_pseudonym_or_by_name_when_names_are_given() {
        let (_dir, model) = demo_model();
        let person = model
            .entities()
            .unwrap()
            .into_iter()
            .find(|e| e.token.is_some())
            .unwrap();
        let token = person.token.clone().unwrap();
        let plain = entity_card(&model, person.id, &HashMap::new()).unwrap();
        assert_eq!(plain.label, token);
        let names = HashMap::from([(token, "Вигадане Ім'я".to_string())]);
        let named = entity_card(&model, person.id, &names).unwrap();
        assert_eq!(named.label, "Вигадане Ім'я");
    }

    #[test]
    fn a_shared_address_is_shown_on_both_cards_and_in_the_list() {
        let (_dir, model) = demo_model();
        let list = entity_list(&model, &HashMap::new()).unwrap();
        let company = list.iter().find(|r| r.label == "ТОВ «Приклад»").unwrap().id;
        let council = list.iter().find(|r| r.label == "Міська рада").unwrap().id;
        let address = "A-ABCDEFGHIJKLMNOPQRSTUVWXYZ";
        model.add_identifier(company, "address", address).unwrap();
        assert!(entity_card(&model, company, &HashMap::new())
            .unwrap()
            .shared
            .is_empty());
        model.add_identifier(council, "address", address).unwrap();
        let card = entity_card(&model, company, &HashMap::new()).unwrap();
        assert_eq!(card.shared.len(), 1);
        assert_eq!(card.shared[0].kind, "address");
        assert_eq!(card.shared[0].others[0].label, "Міська рада");
        let back = entity_card(&model, council, &HashMap::new()).unwrap();
        assert_eq!(back.shared[0].others[0].label, "ТОВ «Приклад»");
        let list = entity_list(&model, &HashMap::new()).unwrap();
        assert_eq!(list.iter().find(|r| r.id == company).unwrap().shared, 1);
        assert_eq!(list.iter().filter(|r| r.shared > 0).count(), 2);
    }

    #[test]
    fn an_unknown_entity_is_an_error() {
        let (_dir, model) = demo_model();
        assert!(entity_card(&model, 9999, &HashMap::new()).is_err());
    }
}
