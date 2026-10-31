//! The shortest chains that connect two entities through shared statements
//! ("how is A connected to B?"). Two entities are one step apart when they stand in the same
//! statement. Refuted statements are skipped unless asked for. Several shortest chains of the
//! same length are returned (a few of them), each step with all statements behind it.

use std::collections::{HashMap, HashSet, VecDeque};

use serde::Serialize;

use crate::card::label_of;
use crate::model::{Model, ModelError, StatementRow, Status};

/// How many chains are returned at most.
const MAX_PATHS: usize = 5;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PathStatement {
    pub id: i64,
    pub kind: String,
    pub from_role: String,
    pub to_role: String,
    pub value: String,
    pub grade: String,
    pub status: String,
}

/// An identifier (address, phone, plate...) that both ends of a step have.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PathSign {
    pub kind: String,
    pub value: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PathHop {
    pub from_id: i64,
    pub from: String,
    pub to_id: i64,
    pub to: String,
    pub statements: Vec<PathStatement>,
    /// Identifiers the two ends have in common.
    pub signs: Vec<PathSign>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EntityPath {
    /// Number of steps.
    pub length: u32,
    pub hops: Vec<PathHop>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PathResult {
    pub paths: Vec<EntityPath>,
}

fn collect(
    node: i64,
    from: i64,
    previous: &HashMap<i64, Vec<i64>>,
    tail: &mut Vec<i64>,
    out: &mut Vec<Vec<i64>>,
) {
    if out.len() >= MAX_PATHS {
        return;
    }
    tail.push(node);
    if node == from {
        let mut path = tail.clone();
        path.reverse();
        out.push(path);
    } else if let Some(list) = previous.get(&node) {
        for before in list {
            collect(*before, from, previous, tail, out);
            if out.len() >= MAX_PATHS {
                break;
            }
        }
    }
    tail.pop();
}

fn role_of(statement: &StatementRow, entity: i64) -> String {
    statement
        .parties
        .iter()
        .find(|(id, _)| *id == entity)
        .map(|(_, role)| role.clone())
        .unwrap_or_default()
}

/// The shortest chains from `from` to `to`, at most `max_depth` steps long.
pub fn find_paths(
    model: &Model,
    from: i64,
    to: i64,
    max_depth: u32,
    include_refuted: bool,
    names: &HashMap<String, String>,
) -> Result<PathResult, ModelError> {
    model.entity(from)?;
    model.entity(to)?;
    if from == to {
        return Err(ModelError::Invalid("choose two different entities"));
    }
    let statements: Vec<StatementRow> = model
        .statements()?
        .into_iter()
        .filter(|statement| include_refuted || statement.status != Status::Refuted)
        .collect();

    let mut adjacent: HashMap<i64, Vec<i64>> = HashMap::new();
    let mut linking: HashMap<(i64, i64), Vec<usize>> = HashMap::new();
    for (index, statement) in statements.iter().enumerate() {
        let mut seen = HashSet::new();
        let ids: Vec<i64> = statement
            .parties
            .iter()
            .map(|(entity, _)| *entity)
            .filter(|entity| seen.insert(*entity))
            .collect();
        for a in &ids {
            for b in &ids {
                if a == b {
                    continue;
                }
                if a < b {
                    linking.entry((*a, *b)).or_default().push(index);
                }
                let list = adjacent.entry(*a).or_default();
                if !list.contains(b) {
                    list.push(*b);
                }
            }
        }
    }

    let mut signs: HashMap<(i64, i64), Vec<PathSign>> = HashMap::new();
    for (kind, value, holders) in model.shared_identifiers()? {
        for a in &holders {
            for b in &holders {
                if a >= b {
                    continue;
                }
                signs.entry((*a, *b)).or_default().push(PathSign {
                    kind: kind.clone(),
                    value: value.clone(),
                });
                for (x, y) in [(a, b), (b, a)] {
                    let list = adjacent.entry(*x).or_default();
                    if !list.contains(y) {
                        list.push(*y);
                    }
                }
            }
        }
    }

    // breadth first search: distances and all predecessors on shortest ways
    let mut distance: HashMap<i64, u32> = HashMap::from([(from, 0)]);
    let mut previous: HashMap<i64, Vec<i64>> = HashMap::new();
    let mut queue = VecDeque::from([from]);
    while let Some(node) = queue.pop_front() {
        let here = distance[&node];
        if here >= max_depth {
            continue;
        }
        for next in adjacent.get(&node).map(Vec::as_slice).unwrap_or(&[]) {
            match distance.get(next) {
                None => {
                    distance.insert(*next, here + 1);
                    previous.insert(*next, vec![node]);
                    queue.push_back(*next);
                }
                Some(known) if *known == here + 1 => {
                    let list = previous.entry(*next).or_default();
                    if !list.contains(&node) {
                        list.push(node);
                    }
                }
                Some(_) => {}
            }
        }
    }

    let mut chains: Vec<Vec<i64>> = Vec::new();
    if distance.contains_key(&to) {
        collect(to, from, &previous, &mut Vec::new(), &mut chains);
    }

    let mut cache: HashMap<i64, String> = HashMap::new();
    let mut paths = Vec::new();
    for chain in chains {
        let mut hops = Vec::new();
        for pair in chain.windows(2) {
            let (a, b) = (pair[0], pair[1]);
            let key = if a < b { (a, b) } else { (b, a) };
            let mut behind = Vec::new();
            for index in linking.get(&key).map(Vec::as_slice).unwrap_or(&[]) {
                let statement = &statements[*index];
                behind.push(PathStatement {
                    id: statement.id,
                    kind: statement.kind.as_str().to_string(),
                    from_role: role_of(statement, a),
                    to_role: role_of(statement, b),
                    value: statement.value.clone().unwrap_or_default(),
                    grade: model.grade(statement.id)?.to_string(),
                    status: statement.status.as_str().to_string(),
                });
            }
            hops.push(PathHop {
                from_id: a,
                from: label_of(model, names, &mut cache, a)?,
                to_id: b,
                to: label_of(model, names, &mut cache, b)?,
                statements: behind,
                signs: signs.get(&key).cloned().unwrap_or_default(),
            });
        }
        paths.push(EntityPath {
            length: u32::try_from(hops.len()).unwrap_or(u32::MAX),
            hops,
        });
    }
    Ok(PathResult { paths })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::card::entity_list;
    use crate::demo::create_demo;

    fn demo() -> (tempfile::TempDir, Model, Vec<(i64, String)>) {
        let dir = tempfile::tempdir().unwrap();
        let home = dir.path().join("demo");
        create_demo(&home).unwrap();
        let model = Model::open(&home.join("model.db"), "test").unwrap();
        let list = entity_list(&model, &HashMap::new())
            .unwrap()
            .into_iter()
            .map(|row| (row.id, row.label))
            .collect();
        (dir, model, list)
    }

    fn id(list: &[(i64, String)], label: &str) -> i64 {
        list.iter().find(|(_, l)| l == label).unwrap().0
    }

    #[test]
    fn a_company_is_connected_to_the_customer_through_the_procurement() {
        let (_dir, model, list) = demo();
        let from = id(&list, "ТОВ «Приклад»");
        let to = id(&list, "Міська рада");
        let result = find_paths(&model, from, to, 6, false, &HashMap::new()).unwrap();
        assert_eq!(result.paths.len(), 1);
        let path = &result.paths[0];
        assert_eq!(path.length, 2);
        assert_eq!(path.hops[0].to, "Закупівля «Ремонт дороги»");
        assert_eq!(path.hops[0].statements[0].kind, "won");
        assert_eq!(path.hops[1].statements[0].kind, "customer");
    }

    #[test]
    fn two_people_are_connected_through_their_company() {
        let (_dir, model, list) = demo();
        let first_person = list[0].0;
        let third_person = list[2].0;
        let result = find_paths(
            &model,
            first_person,
            third_person,
            6,
            false,
            &HashMap::new(),
        )
        .unwrap();
        assert_eq!(result.paths.len(), 1);
        assert_eq!(result.paths[0].length, 2);
        assert_eq!(result.paths[0].hops[0].to, "ТОВ «Приклад»");
    }

    #[test]
    fn a_shared_address_is_a_step_of_its_own() {
        let (_dir, model, list) = demo();
        let from = id(&list, "ТОВ «Приклад»");
        let to = id(&list, "Міська рада");
        let address = "A-ABCDEFGHIJKLMNOPQRSTUVWXYZ";
        let other = model
            .add_entity(&crate::model::NewEntity {
                kind: crate::model::EntityKind::Org,
                token: None,
                label: Some("Сусідня фірма"),
                public_code: None,
            })
            .unwrap();
        model.add_identifier(from, "address", address).unwrap();
        model.add_identifier(other, "address", address).unwrap();
        let result = find_paths(&model, from, other, 3, false, &HashMap::new()).unwrap();
        assert_eq!(result.paths.len(), 1);
        let hop = &result.paths[0].hops[0];
        assert!(hop.statements.is_empty());
        assert_eq!(hop.signs[0].kind, "address");
        // ordinary chains are unchanged
        assert_eq!(
            find_paths(&model, from, to, 6, false, &HashMap::new())
                .unwrap()
                .paths[0]
                .length,
            2
        );
    }

    #[test]
    fn the_depth_limit_is_respected() {
        let (_dir, model, list) = demo();
        let from = list[1].0; // the second person: director of the company
        let to = id(&list, "Міська рада");
        assert!(find_paths(&model, from, to, 2, false, &HashMap::new())
            .unwrap()
            .paths
            .is_empty());
        let found = find_paths(&model, from, to, 3, false, &HashMap::new()).unwrap();
        assert_eq!(found.paths.len(), 1);
        assert_eq!(found.paths[0].length, 3);
    }

    #[test]
    fn a_refuted_statement_cuts_the_way_unless_asked_for() {
        let (_dir, model, list) = demo();
        let from = id(&list, "ТОВ «Приклад»");
        let to = id(&list, "Міська рада");
        let won = model
            .statements()
            .unwrap()
            .into_iter()
            .find(|s| s.kind.as_str() == "won")
            .unwrap()
            .id;
        model.set_status(won, Status::Refuted).unwrap();
        assert!(find_paths(&model, from, to, 6, false, &HashMap::new())
            .unwrap()
            .paths
            .is_empty());
        assert_eq!(
            find_paths(&model, from, to, 6, true, &HashMap::new())
                .unwrap()
                .paths
                .len(),
            1
        );
    }

    #[test]
    fn the_same_entity_or_an_unknown_one_is_an_error() {
        let (_dir, model, list) = demo();
        assert!(find_paths(&model, list[0].0, list[0].0, 6, false, &HashMap::new()).is_err());
        assert!(find_paths(&model, list[0].0, 9999, 6, false, &HashMap::new()).is_err());
    }
}
