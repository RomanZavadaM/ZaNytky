//! Readable documents made from the base: a dossier of one or several subjects (a text a person
//! can read and keep) and the analysis of an investigator's article (who in it the base knows,
//! how they are linked, which documents stand behind them, who is new). Persons are shown by
//! pseudonym unless a map of names (opened with the investigator's key) is given; nothing is stored.

use std::collections::{BTreeMap, HashMap, HashSet};

use serde::Serialize;

use crate::card::{entity_card, CardStatement, EntityCard};
use crate::discover::{discover_in_text, Found};
use crate::model::{Model, ModelError};
use crate::path::find_paths;
use crate::registry::Registry;

/// Readable Ukrainian words for the kinds, roles and statuses kept in the base.
pub fn word_uk(key: &str) -> String {
    let text = match key {
        "person" => "особа",
        "org" => "організація",
        "group" => "група",
        "asset" => "актив",
        "procurement" => "закупівля",
        "market" => "ринок",
        "event" => "подія",
        "document" => "документ",
        "owns" => "володіє",
        "beneficiary" => "бенефіціар",
        "director" => "посада, керівництво",
        "founder" => "засновник",
        "won" => "виграв",
        "customer" => "замовник",
        "supplies" => "постачає",
        "uses" => "використовує",
        "coordinated" => "узгоджені дії",
        "member_of_group" => "член групи",
        "operates_in_market" => "працює на ринку",
        "reported_link" => "повідомлений зв'язок",
        "registry" => "реєстр",
        "authority" => "заява органу",
        "media" => "медіа",
        "testimony" => "свідчення",
        "rumor" => "чутка",
        "own" => "власне",
        "confirmed" => "підтверджено документом",
        "unverified" => "не підтверджено",
        "refuted" => "спростовано",
        "party" => "сторона",
        "company" => "компанія",
        "owner" => "власник",
        "owned" => "об'єкт власності",
        "supplier" => "постачальник",
        "member" => "член",
        "address" => "адреса",
        "phone" => "телефон",
        "email" => "пошта",
        "plate" => "номер авто",
        "iban" => "рахунок",
        "card" => "картка",
        other => return other.to_string(),
    };
    text.to_string()
}

/// A text cut to `max` characters on one line.
fn short(text: &str, max: usize) -> String {
    let flat = text.split_whitespace().collect::<Vec<_>>().join(" ");
    if flat.chars().count() <= max {
        flat
    } else {
        let cut: String = flat.chars().take(max).collect();
        format!("{cut}…")
    }
}

fn hashes(level: usize) -> String {
    "#".repeat(level.clamp(1, 6))
}

fn capitalize(text: &str) -> String {
    let mut chars = text.chars();
    match chars.next() {
        Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
        None => String::new(),
    }
}

fn render_statement(out: &mut String, s: &CardStatement) {
    let value = if s.value.is_empty() {
        "(без тексту)"
    } else {
        s.value.as_str()
    };
    let mut line = format!("- {value} [{}", word_uk(&s.status));
    if s.independent > 1 {
        line.push_str(&format!("; незалежних джерел: {}", s.independent));
    }
    line.push(']');
    let mut dates = Vec::new();
    if let Some(d) = &s.valid_from {
        dates.push(format!("з {d}"));
    }
    if let Some(d) = &s.valid_to {
        dates.push(format!("до {d}"));
    }
    if let Some(d) = &s.as_of {
        dates.push(format!("станом на {d}"));
    }
    if !dates.is_empty() {
        line.push_str(&format!(" ({})", dates.join(", ")));
    }
    out.push_str(&line);
    out.push('\n');
    if !s.parties.is_empty() {
        let with: Vec<String> = s
            .parties
            .iter()
            .map(|p| format!("{} ({})", p.label, word_uk(&p.role)))
            .collect();
        out.push_str(&format!("  - Разом із: {}\n", with.join("; ")));
    }
    if !s.note.trim().is_empty() {
        out.push_str(&format!("  - Примітка: {}\n", short(&s.note, 400)));
    }
    for source in s.sources.iter().take(3) {
        let mut src = format!(
            "  - Джерело №{}: {} ({}, надійність {})",
            source.id,
            short(&source.title, 120),
            word_uk(&source.kind),
            source.reliability
        );
        if let Some(url) = &source.url {
            src.push_str(&format!(" {url}"));
        }
        out.push_str(&src);
        out.push('\n');
    }
    if s.sources.len() > 3 {
        out.push_str(&format!("  - …і ще джерел: {}\n", s.sources.len() - 3));
    }
}

fn render_card(out: &mut String, card: &EntityCard, level: usize) {
    let confirmed = card
        .statements
        .iter()
        .filter(|s| s.status == "confirmed")
        .count();
    let refuted = card
        .statements
        .iter()
        .filter(|s| s.status == "refuted")
        .count();
    let open = card.statements.len() - confirmed - refuted;
    let mut documents: HashSet<i64> = HashSet::new();
    for statement in &card.statements {
        for source in &statement.sources {
            documents.insert(source.id);
        }
    }
    out.push_str(&format!("{} {}\n\n", hashes(level), card.label));
    let mut head = format!("Вид: {}", word_uk(&card.kind));
    if let Some(code) = &card.code {
        head.push_str(&format!(" · код: {code}"));
    }
    out.push_str(&format!("{head}\n\n"));
    out.push_str(&format!(
        "Коротко: тверджень {} (підтверджено документом {confirmed}, не підтверджено {open}, спростовано {refuted}); різних джерел {}.\n\n",
        card.statements.len(),
        documents.len()
    ));
    if card.statements.is_empty() {
        out.push_str("У базі про цей суб'єкт тверджень поки немає.\n\n");
    } else {
        out.push_str(&format!("{} Що відомо\n\n", hashes(level + 1)));
        let mut groups: BTreeMap<String, Vec<&CardStatement>> = BTreeMap::new();
        for statement in &card.statements {
            groups
                .entry(word_uk(&statement.kind))
                .or_default()
                .push(statement);
        }
        for (kind, statements) in groups {
            out.push_str(&format!("**{}**\n\n", capitalize(&kind)));
            for s in statements {
                render_statement(out, s);
            }
            out.push('\n');
        }
    }
    if !card.neighbors.is_empty() {
        out.push_str(&format!("{} З ким пов'язаний\n\n", hashes(level + 1)));
        let mut neighbors: Vec<_> = card.neighbors.iter().collect();
        neighbors.sort_by(|a, b| b.count.cmp(&a.count).then(a.label.cmp(&b.label)));
        for n in neighbors.into_iter().take(20) {
            out.push_str(&format!(
                "- {} ({}) — у {} твердженнях\n",
                n.label,
                word_uk(&n.kind),
                n.count
            ));
        }
        out.push('\n');
    }
    if !card.owners.is_empty() {
        out.push_str(&format!("{} Ланцюжок власності\n\n", hashes(level + 1)));
        for o in &card.owners {
            let value = if o.value.is_empty() {
                String::new()
            } else {
                format!(" ({})", short(&o.value, 120))
            };
            out.push_str(&format!(
                "- крок {}: {} → {}{value}\n",
                o.depth, o.owner, o.owned
            ));
        }
        out.push('\n');
    }
    if !card.shared.is_empty() {
        out.push_str(&format!(
            "{} Спільні ознаки з іншими суб'єктами\n\n",
            hashes(level + 1)
        ));
        for sign in &card.shared {
            let others: Vec<String> = sign.others.iter().map(|o| o.label.clone()).collect();
            let what = sign.text.clone().unwrap_or_else(|| sign.value.clone());
            out.push_str(&format!(
                "- {}: {} — також у: {}\n",
                word_uk(&sign.kind),
                what,
                others.join(", ")
            ));
        }
        out.push('\n');
    }
}

/// The dossier of one or several subjects as Markdown a person can read and keep.
pub fn dossier_markdown(
    model: &Model,
    names: &HashMap<String, String>,
    ids: &[i64],
) -> Result<String, ModelError> {
    let mut cards = Vec::new();
    for id in ids {
        cards.push(entity_card(model, *id, names)?);
    }
    let mut out = String::new();
    if cards.len() > 1 {
        out.push_str("# Досьє: добірка суб'єктів\n\n");
        for card in &cards {
            out.push_str(&format!("- {} ({})\n", card.label, word_uk(&card.kind)));
        }
        out.push_str("\n---\n\n");
    }
    let level = if cards.len() > 1 { 2 } else { 1 };
    for card in &cards {
        render_card(&mut out, card, level);
    }
    out.push_str("---\nФакти з бази ZaNytky; «не підтверджено» означає, що документа-підтвердження в базі ще немає. Висновків це досьє не робить.\n");
    Ok(out)
}

/// A subject of the base that the article names.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ArticleSubject {
    pub id: i64,
    pub label: String,
    pub kind: String,
    /// How many times the article names it.
    pub mentions: u32,
    pub statements: u32,
    pub confirmed: u32,
}

/// A chain through the base between two subjects of the article.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ArticleLink {
    pub from: String,
    pub to: String,
    pub steps: Vec<String>,
}

/// A document of the base behind the statements about the article's subjects.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ArticleDoc {
    pub id: i64,
    pub title: String,
    pub kind: String,
    pub url: Option<String>,
    pub subjects: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ArticleAnalysis {
    pub words: usize,
    pub known: Vec<ArticleSubject>,
    /// Named in the article, not in the base yet.
    pub unknown: Vec<Found>,
    pub links: Vec<ArticleLink>,
    pub documents: Vec<ArticleDoc>,
}

impl ArticleAnalysis {
    /// The entity numbers of the known subjects (for a graph).
    pub fn known_ids(&self) -> Vec<i64> {
        self.known.iter().map(|s| s.id).collect()
    }
}

const MAX_KNOWN: usize = 40;
const MAX_LINK_SUBJECTS: usize = 14;
const MAX_LINKS: usize = 40;
const MAX_DOCS: usize = 40;

/// Who in the article the base knows, how they are linked and which documents stand behind them.
pub fn analyze_article(
    model: &Model,
    registry: Option<&Registry>,
    names: &HashMap<String, String>,
    text: &str,
) -> Result<ArticleAnalysis, ModelError> {
    let found = discover_in_text(model, registry, text);
    let mut mentions: BTreeMap<i64, u32> = BTreeMap::new();
    let mut unknown = Vec::new();
    for f in found {
        match f.known {
            Some(id) => *mentions.entry(id).or_default() += f.count,
            None => unknown.push(f),
        }
    }
    let mut order: Vec<(i64, u32)> = mentions.into_iter().collect();
    order.sort_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(&b.0)));
    order.truncate(MAX_KNOWN);

    let mut known = Vec::new();
    let mut docs: BTreeMap<i64, ArticleDoc> = BTreeMap::new();
    for (id, count) in &order {
        let card = entity_card(model, *id, names)?;
        for statement in &card.statements {
            for source in &statement.sources {
                let doc = docs.entry(source.id).or_insert_with(|| ArticleDoc {
                    id: source.id,
                    title: short(&source.title, 140),
                    kind: word_uk(&source.kind),
                    url: source.url.clone(),
                    subjects: Vec::new(),
                });
                if !doc.subjects.contains(&card.label) {
                    doc.subjects.push(card.label.clone());
                }
            }
        }
        known.push(ArticleSubject {
            id: *id,
            label: card.label.clone(),
            kind: word_uk(&card.kind),
            mentions: *count,
            statements: card.statements.len() as u32,
            confirmed: card
                .statements
                .iter()
                .filter(|s| s.status == "confirmed")
                .count() as u32,
        });
    }

    let mut links = Vec::new();
    let top: Vec<&ArticleSubject> = known.iter().take(MAX_LINK_SUBJECTS).collect();
    'pairs: for (i, a) in top.iter().enumerate() {
        for b in &top[i + 1..] {
            let result = find_paths(model, a.id, b.id, 3, false, names)?;
            let Some(path) = result.paths.first() else {
                continue;
            };
            let mut steps = Vec::new();
            for hop in &path.hops {
                let what = match hop.statements.first() {
                    Some(s) => format!("{}: {}", word_uk(&s.kind), short(&s.value, 100)),
                    None => match hop.signs.first() {
                        Some(sign) => format!("спільна ознака ({})", word_uk(&sign.kind)),
                        None => "зв'язок".to_string(),
                    },
                };
                steps.push(format!("{} — {} — {}", hop.from, what, hop.to));
            }
            links.push(ArticleLink {
                from: a.label.clone(),
                to: b.label.clone(),
                steps,
            });
            if links.len() >= MAX_LINKS {
                break 'pairs;
            }
        }
    }

    let mut documents: Vec<ArticleDoc> = docs.into_values().collect();
    documents.sort_by(|a, b| {
        b.subjects
            .len()
            .cmp(&a.subjects.len())
            .then(a.id.cmp(&b.id))
    });
    documents.truncate(MAX_DOCS);
    unknown.sort_by(|a, b| b.count.cmp(&a.count).then(a.text.cmp(&b.text)));

    Ok(ArticleAnalysis {
        words: text.split_whitespace().count(),
        known,
        unknown,
        links,
        documents,
    })
}

/// The analysis as Markdown.
pub fn article_markdown(analysis: &ArticleAnalysis, title: &str) -> String {
    let mut out = format!("# Аналіз статті: {title}\n\n");
    out.push_str(&format!(
        "У статті близько {} слів. База впізнала суб'єктів: {}. Нових (яких у базі ще немає): {}.\n\n",
        analysis.words,
        analysis.known.len(),
        analysis.unknown.len()
    ));
    out.push_str("## Суб'єкти статті, які є в базі\n\n");
    if analysis.known.is_empty() {
        out.push_str("Жодного суб'єкта з бази в тексті не знайдено.\n\n");
    } else {
        out.push_str("| Суб'єкт | Вид | Згадок у статті | Тверджень у базі | З них підтверджено |\n|---|---|---|---|---|\n");
        for s in &analysis.known {
            out.push_str(&format!(
                "| №{} {} | {} | {} | {} | {} |\n",
                s.id,
                s.label.replace('|', "/"),
                s.kind,
                s.mentions,
                s.statements,
                s.confirmed
            ));
        }
        out.push('\n');
    }
    out.push_str("## Як вони пов'язані за даними бази\n\n");
    if analysis.links.is_empty() {
        out.push_str("Ланцюжків між суб'єктами статті в базі немає.\n\n");
    } else {
        for link in &analysis.links {
            out.push_str(&format!("- **{} ↔ {}**\n", link.from, link.to));
            for step in &link.steps {
                out.push_str(&format!("  - {step}\n"));
            }
        }
        out.push('\n');
    }
    out.push_str("## Документи й джерела бази про цих суб'єктів\n\n");
    if analysis.documents.is_empty() {
        out.push_str("Документів немає.\n\n");
    } else {
        for d in &analysis.documents {
            let mut line = format!("- №{} {} ({})", d.id, d.title, d.kind);
            if let Some(url) = &d.url {
                line.push_str(&format!(" {url}"));
            }
            line.push_str(&format!(" — про: {}", d.subjects.join(", ")));
            out.push_str(&line);
            out.push('\n');
        }
        out.push('\n');
    }
    out.push_str("## Названо в статті, у базі ще немає\n\n");
    if analysis.unknown.is_empty() {
        out.push_str("Нових суб'єктів немає.\n\n");
    } else {
        for f in analysis.unknown.iter().take(60) {
            let kind = match f.kind.as_str() {
                "code" => "код",
                "person" => "особа",
                "org" => "організація",
                other => other,
            };
            out.push_str(&format!("- {} ({kind}) — згадок: {}\n", f.text, f.count));
        }
        out.push('\n');
    }
    out.push_str("---\nЗбіг імені з базою не доводить, що це та сама особа: перевіряйте за кодом і документами. Звіт не робить висновків.\n");
    out
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
    fn a_dossier_is_readable_ukrainian_text() {
        let (_dir, model) = demo_model();
        let list = crate::card::entity_list(&model, &HashMap::new()).unwrap();
        let company = list
            .iter()
            .find(|r| r.label == "ТОВ «Приклад»")
            .unwrap()
            .id;
        let text = dossier_markdown(&model, &HashMap::new(), &[company]).unwrap();
        assert!(text.starts_with("# ТОВ «Приклад»"));
        assert!(text.contains("Що відомо") && text.contains("Джерело №"));
        // no raw codes of kinds and statuses leak into the text
        assert!(!text.contains("unverified") && !text.contains("reported_link"));
        let two = dossier_markdown(&model, &HashMap::new(), &[list[0].id, list[1].id]).unwrap();
        assert!(two.starts_with("# Досьє: добірка"));
    }

    #[test]
    fn the_article_analysis_finds_known_subjects_and_their_documents() {
        let (_dir, model) = demo_model();
        let analysis = analyze_article(
            &model,
            None,
            &HashMap::new(),
            "Компанія ТОВ «Приклад» (код 12345678) отримала контракт.",
        )
        .unwrap();
        assert!(analysis.known.iter().any(|s| s.label == "ТОВ «Приклад»"));
        assert!(!analysis.documents.is_empty());
        let text = article_markdown(&analysis, "проба");
        assert!(text.contains("# Аналіз статті: проба") && text.contains("Суб'єкти статті"));
    }
}
