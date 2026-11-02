//! Finding subjects in texts (the "Sources" tab): persons by their names, organisations by
//! a legal form and a quoted name, registry codes (ЄДРПОУ). The program counts and shows what
//! it found and whether the base already knows it; the investigator confirms each one.
//! These are hints, not facts: names written in capitals can be places or institutions.

use std::collections::{BTreeMap, HashMap, HashSet};

use serde::Serialize;

use crate::deidentify::{find_candidates, is_surname_form};
use crate::make::ensure_entity;
use crate::model::{label_key, EntityKind, Model};
use crate::normalize::Kind;
use crate::registry::Registry;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Found {
    /// `person`, `org` or `code`.
    pub kind: String,
    /// The most frequent written form.
    pub text: String,
    /// How many times it occurs.
    pub count: u32,
    /// In how many sources.
    pub sources: u32,
    /// The entity of the base it is already known as.
    pub known: Option<i64>,
}

/// Beginnings of common first names (Ukrainian and others); a word that starts with one of
/// them (any case form) makes a pair of capitalised words a person.
#[rustfmt::skip]
const FIRST_NAMES: &[&str] = &[
    "олекс", "андрі", "сергі", "дмитр", "михай", "володи", "іван", "юрі", "віталі", "віктор",
    "ігор", "роман", "петр", "павл", "богдан", "тарас", "олег", "максим", "артем", "денис",
    "євген", "костянт", "вадим", "василь", "анатол", "валер", "леонід", "станіс", "ярослав",
    "руслан", "назар", "тимур", "тимоф", "гліб", "данил", "данія", "кирил", "герман", "геннад",
    "борис", "федір", "федор", "захар", "степан", "святосл", "мирон", "марко", "семен", "григор",
    "матві", "радіон", "остап", "орест", "антон", "микит", "микол", "едуард", "арсен", "філіп",
    "давид", "ростислав", "тихон", "ілон", "дональд", "олена", "наталі", "тетян", "ірина",
    "юлія", "юлію", "юлії", "світлан", "оксан", "марин", "анна", "анни", "ольг", "людмил",
    "галин", "вікторі", "катери", "надія", "надії", "валентин", "любов", "лариса", "ларис",
    "софі", "дарі", "анастас", "ксені", "поліна", "олесь", "марія", "марії", "соломі", "лілі",
    "лідія", "лідії", "севгіл", "кіра", "кіри", "емануел", "еммануел", "фрідріх", "олаф",
];

/// Words that start a sentence and are not part of a name.
#[rustfmt::skip]
const LEADING_NOISE: &[&str] = &[
    "дякую", "дякуємо", "президент", "також", "нагадаю", "привіт", "увага", "отже", "тому",
    "однак", "зараз", "сьогодні", "вчора", "завтра", "прокурор", "слідчий", "суддя", "депутат",
    "міністр", "голова", "директор", "радник", "нардеп", "журналіст", "журналістка", "пан",
    "пані", "ексміністр", "віцепрем'єр", "канцлер", "посол", "генерал", "полковник",
];

/// Legal forms that start the name of an organisation.
const LEGAL_FORMS: &[&str] = &["ТОВ", "ТзОВ", "ПП", "АТ", "ПАТ", "ПрАТ", "КП", "ДП", "НАЕК"];

fn is_first_name(word: &str) -> bool {
    let lower = word.to_lowercase();
    lower.chars().count() >= 4 && FIRST_NAMES.iter().any(|stem| lower.starts_with(stem))
}

fn is_initials(word: &str) -> bool {
    word.contains('.')
}

/// The words of a run that make a person's name, or none if it does not look like a name.
fn person_words(text: &str) -> Option<Vec<String>> {
    let mut words: Vec<String> = text.split_whitespace().map(str::to_string).collect();
    while words.len() > 2 && LEADING_NOISE.contains(&words[0].to_lowercase().as_str()) {
        words.remove(0);
    }
    if words.len() < 2 || words.len() > 4 {
        return None;
    }
    let looks_like_a_name = words
        .iter()
        .any(|w| is_first_name(w) || is_initials(w) || is_surname_form(w));
    looks_like_a_name.then_some(words)
}

/// The key that joins the case forms and the order of a person's name:
/// the first five letters of every word, sorted.
fn person_key(words: &[String]) -> String {
    let mut parts: Vec<String> = words
        .iter()
        .map(|w| w.to_lowercase().chars().take(5).collect())
        .collect();
    parts.sort();
    parts.join("|")
}

/// Plausible nominative forms of a word written in another case.
fn nominative_forms(word: &str) -> Vec<String> {
    let mut forms = vec![word.to_string()];
    for ending in [
        "ого", "ому", "ом", "ем", "єм", "ові", "єві", "а", "у", "і", "ю", "я", "ї",
    ] {
        if let Some(base) = word.strip_suffix(ending) {
            if base.chars().count() >= 3 {
                for tail in ["", "о", "й", "ь", "ий", "ій"] {
                    forms.push(format!("{base}{tail}"));
                }
            }
        }
    }
    forms.sort();
    forms.dedup();
    forms
}

/// The entity of the base that a person's name belongs to, if the registry knows the name
/// (in the written form or in a nominative form, in either order of the words).
fn known_person(model: &Model, registry: &Registry, words: &[String]) -> Option<i64> {
    let first = nominative_forms(&words[0]);
    let last = nominative_forms(&words[words.len() - 1]);
    let mut names = vec![words.join(" ")];
    for a in &first {
        for b in &last {
            names.push(format!("{a} {b}"));
            names.push(format!("{b} {a}"));
        }
    }
    for name in names {
        let Ok(Some(token)) = registry.find(Kind::Name, &name) else {
            continue;
        };
        if let Ok(Some(id)) = model.find_entity(EntityKind::Person, Some(&token), None) {
            return Some(id);
        }
    }
    None
}

/// Organisations written as a legal form and a quoted name: `ТОВ «Приклад»`.
fn org_names(text: &str) -> Vec<String> {
    let mut out = Vec::new();
    for form in LEGAL_FORMS {
        for (at, _) in text.match_indices(form) {
            let before = text[..at].chars().next_back();
            if before.is_some_and(|c| c.is_alphanumeric()) {
                continue;
            }
            let after = &text[at + form.len()..];
            let trimmed = after.trim_start();
            let Some(open) = trimmed.chars().next() else {
                continue;
            };
            let close = match open {
                '\u{ab}' => '\u{bb}',
                '\u{201c}' => '\u{201d}',
                '\u{201e}' => '\u{201c}',
                '"' => '"',
                _ => continue,
            };
            let inner = &trimmed[open.len_utf8()..];
            let Some(end) = inner.find(close) else {
                continue;
            };
            let name = inner[..end].trim();
            if (2..=60).contains(&name.chars().count()) && !name.contains('\n') {
                out.push(format!("{form} \u{ab}{name}\u{bb}"));
            }
        }
    }
    out
}

/// Eight-digit registry codes written after `ЄДРПОУ`.
fn codes_in(text: &str) -> Vec<String> {
    let mut out = Vec::new();
    for (at, _) in text.match_indices("ЄДРПОУ") {
        let rest: String = text[at..].chars().skip(6).take(24).collect();
        let mut digits = String::new();
        for c in rest.chars() {
            if c.is_ascii_digit() {
                digits.push(c);
            } else if !digits.is_empty() {
                break;
            }
        }
        if digits.len() == 8 {
            out.push(digits);
        }
    }
    out
}

/// Everything a text names: (kind, key, written form), once for every occurrence.
fn subjects_in(text: &str) -> Vec<(String, String, String)> {
    let mut out = Vec::new();
    for candidate in find_candidates(text) {
        if let Some(words) = person_words(&candidate.text) {
            for _ in 0..candidate.count {
                out.push(("person".to_string(), person_key(&words), words.join(" ")));
            }
        }
    }
    for name in org_names(text) {
        out.push(("org".to_string(), label_key(&name), name));
    }
    for code in codes_in(text) {
        out.push(("code".to_string(), code.clone(), code));
    }
    out
}

/// What one text brings: key -> (kind, written forms with counts, count).
#[derive(Default)]
struct Tally {
    forms: HashMap<(String, String), BTreeMap<String, u32>>,
    sources: HashMap<(String, String), u32>,
}

impl Tally {
    /// Count what one text brings (one more source for every thing it contains).
    fn add(&mut self, text: &str) {
        let mut seen: HashSet<(String, String)> = HashSet::new();
        for (kind, key, form) in subjects_in(text) {
            let id = (kind, key);
            *self
                .forms
                .entry(id.clone())
                .or_default()
                .entry(form)
                .or_insert(0) += 1;
            if seen.insert(id.clone()) {
                *self.sources.entry(id).or_insert(0) += 1;
            }
        }
    }

    fn into_found(self, model: &Model, registry: Option<&Registry>, only_new: bool) -> Vec<Found> {
        self.into_keyed(model, registry, only_new)
            .into_iter()
            .map(|(_, _, found)| found)
            .collect()
    }

    /// Like `into_found`, with the (kind, key) of every find.
    fn into_keyed(
        self,
        model: &Model,
        registry: Option<&Registry>,
        only_new: bool,
    ) -> Vec<(String, String, Found)> {
        let mut out = Vec::new();
        for ((kind, key), forms) in self.forms.iter() {
            let count: u32 = forms.values().sum();
            // the most frequent form, the shortest among equals
            let text = forms
                .iter()
                .max_by(|a, b| a.1.cmp(b.1).then_with(|| b.0.len().cmp(&a.0.len())))
                .map(|(form, _)| form.clone())
                .unwrap_or_default();
            let known = match kind.as_str() {
                "person" => registry.and_then(|r| {
                    let words: Vec<String> = text.split_whitespace().map(str::to_string).collect();
                    known_person(model, r, &words)
                }),
                "org" => model
                    .entities_by_label(EntityKind::Org, &text)
                    .ok()
                    .and_then(|ids| ids.first().copied()),
                _ => model
                    .find_entity(EntityKind::Org, None, Some(&text))
                    .ok()
                    .flatten(),
            };
            if only_new && known.is_some() {
                continue;
            }
            let sources = self
                .sources
                .get(&(kind.clone(), key.clone()))
                .copied()
                .unwrap_or(1);
            out.push((
                kind.clone(),
                key.clone(),
                Found {
                    kind: kind.clone(),
                    text,
                    count,
                    sources,
                    known,
                },
            ));
        }
        out.sort_by(|a, b| {
            b.2.sources
                .cmp(&a.2.sources)
                .then_with(|| b.2.count.cmp(&a.2.count))
                .then_with(|| a.2.text.cmp(&b.2.text))
        });
        out
    }
}

/// Subjects found in one text, with whether the base already knows them.
pub fn discover_in_text(model: &Model, registry: Option<&Registry>, text: &str) -> Vec<Found> {
    let mut tally = Tally::default();
    tally.add(text);
    tally.into_found(model, registry, false)
}

/// The subjects that occur in the most sources of the whole base and are not known yet.
pub fn discover_everywhere(
    model: &Model,
    registry: Option<&Registry>,
    limit: usize,
) -> Result<Vec<Found>, String> {
    let mut tally = Tally::default();
    let conn = model.connection();
    let mut stmt = conn
        .prepare("SELECT body FROM source_text")
        .map_err(|e| e.to_string())?;
    let rows = stmt
        .query_map([], |r| r.get::<_, String>(0))
        .map_err(|e| e.to_string())?;
    for body in rows {
        let body = body.map_err(|e| e.to_string())?;
        tally.add(&body);
    }
    let mut found = tally.into_found(model, registry, true);
    found.retain(|f| f.sources >= 2 || f.kind == "code");
    found.truncate(limit);
    Ok(found)
}

/// Two subjects named in the same paragraph of several posts.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SubjectPair {
    pub a: String,
    pub b: String,
    pub known_a: bool,
    pub known_b: bool,
    /// In how many posts they are named together.
    pub posts: u32,
    pub channels: u32,
    /// The paragraph of the first such post, and its address.
    pub example: String,
    pub url: String,
}

type SubjectId = (String, String);

struct PairCount {
    posts: u32,
    channels: HashSet<i64>,
    example: String,
    url: String,
}

/// Likely links: pairs of the most frequent subjects (persons and organisations) named in the
/// same paragraph in at least three posts, the most widespread first.
pub fn subject_pairs(
    model: &Model,
    registry: Option<&Registry>,
    limit: usize,
) -> Result<Vec<SubjectPair>, String> {
    let mut stmt = model
        .connection()
        .prepare(
            "SELECT p.channel_id, s.url, t.body FROM telegram_post p \
             JOIN source s ON s.id = p.source_id JOIN source_text t ON t.source_id = s.id",
        )
        .map_err(|e| e.to_string())?;
    let rows = stmt
        .query_map([], |r| {
            Ok((
                r.get::<_, i64>(0)?,
                r.get::<_, Option<String>>(1)?.unwrap_or_default(),
                r.get::<_, String>(2)?,
            ))
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    let mut tally = Tally::default();
    for (_, _, body) in &rows {
        tally.add(body);
    }
    let top: HashMap<SubjectId, (String, bool)> = tally
        .into_keyed(model, registry, false)
        .into_iter()
        .filter(|(kind, _, found)| kind.as_str() != "code" && found.sources >= 3)
        .take(200)
        .map(|(kind, key, found)| ((kind, key), (found.text, found.known.is_some())))
        .collect();

    let mut pairs: HashMap<(SubjectId, SubjectId), PairCount> = HashMap::new();
    for (channel, url, body) in &rows {
        let mut counted: HashSet<(SubjectId, SubjectId)> = HashSet::new();
        for paragraph in body.split("\n\n") {
            if paragraph.starts_with("Переслано з:") || paragraph.chars().count() < 20 {
                continue;
            }
            let mut ids: Vec<SubjectId> = subjects_in(paragraph)
                .into_iter()
                .map(|(kind, key, _)| (kind, key))
                .filter(|id| top.contains_key(id))
                .collect();
            ids.sort();
            ids.dedup();
            for i in 0..ids.len() {
                for j in (i + 1)..ids.len() {
                    let pair = (ids[i].clone(), ids[j].clone());
                    if !counted.insert(pair.clone()) {
                        continue;
                    }
                    let entry = pairs.entry(pair).or_insert_with(|| PairCount {
                        posts: 0,
                        channels: HashSet::new(),
                        example: paragraph.chars().take(300).collect(),
                        url: url.clone(),
                    });
                    entry.posts += 1;
                    entry.channels.insert(*channel);
                }
            }
        }
    }
    let mut out: Vec<SubjectPair> = pairs
        .into_iter()
        .filter(|(_, count)| count.posts >= 3)
        .filter_map(|((a, b), count)| {
            let (name_a, known_a) = top.get(&a)?.clone();
            let (name_b, known_b) = top.get(&b)?.clone();
            Some(SubjectPair {
                a: name_a,
                b: name_b,
                known_a,
                known_b,
                posts: count.posts,
                channels: count.channels.len() as u32,
                example: count.example,
                url: count.url,
            })
        })
        .collect();
    out.sort_by(|x, y| {
        y.channels
            .cmp(&x.channels)
            .then_with(|| y.posts.cmp(&x.posts))
            .then_with(|| x.a.cmp(&y.a))
            .then_with(|| x.b.cmp(&y.b))
    });
    out.truncate(limit);
    Ok(out)
}

/// Add a found subject to the base (`person` needs the registry: the name is sealed there and
/// only a pseudonym stays in the base). Returns the id of the entity.
pub fn add_found(
    model: &Model,
    registry: Option<&Registry>,
    kind: &str,
    text: &str,
) -> Result<i64, String> {
    let kind = match kind {
        "person" => EntityKind::Person,
        "org" => EntityKind::Org,
        _ => return Err("this kind cannot be added".to_string()),
    };
    ensure_entity(model, registry, kind, text, None)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn registry() -> Registry {
        let (_, public) = crate::seal::generate_identity();
        let recipient = crate::seal::parse_recipient(&public).unwrap();
        Registry::open_in_memory(&(0u8..32).collect::<Vec<u8>>(), vec![recipient]).unwrap()
    }

    #[test]
    fn organisations_codes_and_names_are_found_in_a_text() {
        let model = Model::open_in_memory("tester").unwrap();
        let text = "Дякую Олегу Гороховскому за донат. ТОВ «Приклад Плюс» (ЄДРПОУ: 12345678) \
                    виграло. Тимур Міндіч і Тимура Міндіча. Верховної Ради. Назва ТОВ «А».";
        let found = discover_in_text(&model, None, text);
        let kinds: Vec<(&str, &str)> = found
            .iter()
            .map(|f| (f.kind.as_str(), f.text.as_str()))
            .collect();
        assert!(kinds.contains(&("org", "ТОВ «Приклад Плюс»")));
        assert!(kinds.contains(&("code", "12345678")));
        assert!(kinds.contains(&("person", "Олегу Гороховскому")));
        // two forms of one name are one candidate
        let mindich = found.iter().find(|f| f.text.contains("Мінді")).unwrap();
        assert_eq!(mindich.count, 2);
        // an institution without a first name is not a person
        assert!(!kinds.iter().any(|(_, t)| t.contains("Верховної")));
    }

    #[test]
    fn a_known_person_is_recognised_in_another_case_form() {
        use crate::model::NewEntity;
        let model = Model::open_in_memory("tester").unwrap();
        let registry = registry();
        let id = add_found(&model, Some(&registry), "person", "Тимур Міндіч").unwrap();
        // the same person again: no second entity
        assert_eq!(
            add_found(&model, Some(&registry), "person", "Тимур Міндіч").unwrap(),
            id
        );
        let found = discover_in_text(&model, Some(&registry), "Зустріч з Тимуром Міндічем.");
        let person = found.iter().find(|f| f.kind == "person").unwrap();
        assert_eq!(person.known, Some(id));
        // an organisation becomes known by its name
        model
            .add_entity(&NewEntity {
                kind: EntityKind::Org,
                token: None,
                label: Some("ТОВ «Приклад»"),
                public_code: None,
            })
            .unwrap();
        let found = discover_in_text(&model, None, "ТОВ «Приклад» виграло.");
        assert!(found.iter().any(|f| f.kind == "org" && f.known.is_some()));
    }
}
