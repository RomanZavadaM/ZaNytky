//! The list of sources ("Sources" tab): search over titles, texts and addresses, pages of
//! newest first, and one source with its text and files. Nothing is stored here.

use serde::Serialize;

use crate::card::{card_source, CardSource};
use crate::model::{Model, ModelError};

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SourceListRow {
    pub id: i64,
    pub title: String,
    pub kind: String,
    pub reliability: String,
    pub url: Option<String>,
    pub published_at: Option<String>,
    /// The beginning of the text of the source.
    pub excerpt: String,
    pub has_text: bool,
    pub files: u32,
    pub statements: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SourceListPage {
    /// All sources that match the query (not only this page).
    pub total: u32,
    pub rows: Vec<SourceListRow>,
}

fn excerpt(body: &str) -> String {
    let flat: String = body.split_whitespace().collect::<Vec<_>>().join(" ");
    let mut short: String = flat.chars().take(220).collect();
    if flat.chars().count() > 220 {
        short.push('…');
    }
    short
}

/// Sources whose title, address or text contain every word of `query` (any letter case),
/// newest first; `offset` and `limit` pick the page. An empty query lists everything.
pub fn source_list(
    model: &Model,
    query: &str,
    offset: u32,
    limit: u32,
) -> Result<SourceListPage, ModelError> {
    let words: Vec<String> = query.split_whitespace().map(str::to_lowercase).collect();
    let mut total = 0u32;
    let mut rows = Vec::new();
    for row in model.sources_overview()? {
        if !words.is_empty() {
            let haystack = format!(
                "{}\n{}\n{}",
                row.title,
                row.url.as_deref().unwrap_or(""),
                row.body.as_deref().unwrap_or("")
            )
            .to_lowercase();
            if !words.iter().all(|word| haystack.contains(word.as_str())) {
                continue;
            }
        }
        if total >= offset && (rows.len() as u32) < limit {
            rows.push(SourceListRow {
                id: row.id,
                title: row.title,
                kind: row.kind.as_str().to_string(),
                reliability: row.reliability.as_str().to_string(),
                url: row.url,
                published_at: row.published_at,
                excerpt: row.body.as_deref().map(excerpt).unwrap_or_default(),
                has_text: row.body.is_some(),
                files: row.files,
                statements: row.statements,
            });
        }
        total += 1;
    }
    Ok(SourceListPage { total, rows })
}

/// One source with its full text and files.
pub fn source_view(model: &Model, id: i64) -> Result<CardSource, ModelError> {
    card_source(model, id)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{NewSource, Reliability, SourceKind};

    fn add(m: &Model, title: &str, date: &str, text: Option<&str>) -> i64 {
        let id = m
            .add_source(&NewSource {
                kind: SourceKind::Media,
                title,
                url: None,
                published_at: Some(date),
                reliability: Reliability::F,
                parent_id: None,
            })
            .unwrap();
        if let Some(text) = text {
            m.set_source_text(id, text).unwrap();
        }
        id
    }

    #[test]
    fn search_ignores_letter_case_needs_every_word_and_pages_newest_first() {
        let m = Model::open_in_memory("tester").unwrap();
        let old = add(&m, "Перший", "2024-01-01", Some("Закупівля у ТОВ Приклад"));
        let mid = add(&m, "Другий", "2024-02-01", Some("Нічого"));
        let new = add(&m, "Третій", "2024-03-01", Some("приклад і закупівля"));

        let all = source_list(&m, "", 0, 10).unwrap();
        assert_eq!(all.total, 3);
        let ids: Vec<i64> = all.rows.iter().map(|r| r.id).collect();
        assert_eq!(ids, vec![new, mid, old]);

        let found = source_list(&m, "ПРИКЛАД закупівля", 0, 10).unwrap();
        assert_eq!(found.total, 2);
        assert_eq!(found.rows[0].id, new);
        assert!(found.rows[0].has_text);

        assert_eq!(source_list(&m, "приклад немає", 0, 10).unwrap().total, 0);

        let page = source_list(&m, "", 1, 1).unwrap();
        assert_eq!(page.total, 3);
        assert_eq!(page.rows.len(), 1);
        assert_eq!(page.rows[0].id, mid);
    }

    #[test]
    fn a_source_is_viewed_with_its_text() {
        let m = Model::open_in_memory("tester").unwrap();
        let id = add(&m, "Джерело", "2024-01-01", Some("Повний текст"));
        let view = source_view(&m, id).unwrap();
        assert_eq!(view.text.as_deref(), Some("Повний текст"));
        assert!(source_view(&m, 999).is_err());
    }
}
