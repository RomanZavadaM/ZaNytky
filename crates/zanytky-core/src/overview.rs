//! The overview of the base (the "Overview" tab): the program goes through everything and
//! writes one report in Markdown for the investigator to read: for every channel its
//! activity, peaks and connections; what the channels have in common; the subjects that
//! occur in many texts and are not in the base yet; likely links between subjects (named in
//! the same paragraph). Everything here is a hint for a hypothesis, not a fact.

use std::fmt::Write;

use crate::channel::{channel_gaps, channel_list, channel_timeline};
use crate::discover::{discover_everywhere, subject_pairs};
use crate::model::Model;
use crate::overlap::{channel_overlaps, is_generic_site};
use crate::registry::Registry;

/// Rows of a query that returns (text, count): the most frequent first.
fn counted(
    model: &Model,
    sql: &str,
    params: &[&dyn rusqlite::ToSql],
) -> Result<Vec<(String, i64)>, String> {
    let mut stmt = model.connection().prepare(sql).map_err(|e| e.to_string())?;
    let rows = stmt
        .query_map(params, |r| {
            Ok((r.get::<_, String>(0)?, r.get::<_, i64>(1)?))
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;
    Ok(rows)
}

fn list(items: &[(String, i64)], take: usize) -> String {
    if items.is_empty() {
        return "-".to_string();
    }
    items
        .iter()
        .take(take)
        .map(|(name, n)| format!("{name} ({n})"))
        .collect::<Vec<_>>()
        .join(", ")
}

/// A cell of a table: no line breaks or bars.
fn cell(text: &str) -> String {
    text.replace(['\n', '\r'], " ").replace('|', "/")
}

/// Months with clearly more posts than usual (the mean plus two deviations).
fn peaks(posts: &[u32]) -> Vec<usize> {
    if posts.len() < 6 {
        return Vec::new();
    }
    let mean = posts.iter().map(|p| f64::from(*p)).sum::<f64>() / posts.len() as f64;
    let deviation = (posts
        .iter()
        .map(|p| (f64::from(*p) - mean).powi(2))
        .sum::<f64>()
        / posts.len() as f64)
        .sqrt();
    let limit = mean + 2.0 * deviation;
    posts
        .iter()
        .enumerate()
        .filter(|(_, p)| f64::from(**p) > limit)
        .map(|(i, _)| i)
        .collect()
}

pub fn build_overview(model: &Model, registry: Option<&Registry>) -> Result<String, String> {
    let mut out = String::new();
    let channels = channel_list(model).map_err(|e| e.to_string())?;
    let (sources, texts, files): (i64, i64, i64) = model
        .connection()
        .query_row(
            "SELECT (SELECT COUNT(*) FROM source), (SELECT COUNT(*) FROM source_text), \
             (SELECT COUNT(*) FROM source_file)",
            [],
            |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
        )
        .map_err(|e| e.to_string())?;
    let _ = writeln!(out, "# Огляд бази\n");
    let _ = writeln!(
        out,
        "Джерел: {sources}; текстів: {texts}; файлів: {files}; каналів Telegram: {}.\n",
        channels.len()
    );
    let _ = writeln!(
        out,
        "> Усе нижче - підказки для гіпотез, які порахувала програма, а не факти й не висновки. \
         Кожне твердження про людей чи організації потребує перевірки за документами.\n"
    );

    let _ = writeln!(out, "## Канали\n");
    for channel in &channels {
        let id = channel.channel_id;
        let _ = writeln!(out, "### {}\n", cell(&channel.name));
        let _ = writeln!(
            out,
            "- Дописів у базі: {}; номери {}-{}; період {} - {}; немає в експорті номерів: {}.",
            channel.posts,
            channel.first_id,
            channel.last_id,
            channel.first_date.as_deref().unwrap_or("?"),
            channel.last_date.as_deref().unwrap_or("?"),
            channel_gaps(model, id).map_err(|e| e.to_string())?.missing
        );
        let months = channel_timeline(model, id).map_err(|e| e.to_string())?;
        let posts: Vec<u32> = months.iter().map(|m| m.posts).collect();
        let marked = peaks(&posts);
        if !marked.is_empty() {
            let text: Vec<String> = marked
                .iter()
                .map(|i| {
                    let m = &months[*i];
                    match &m.top_origin {
                        Some(origin) => format!(
                            "{}: {} дописів, пересланих {} (найбільше з «{}», {})",
                            m.month, m.posts, m.forwarded, origin, m.top_count
                        ),
                        None => format!("{}: {} дописів", m.month, m.posts),
                    }
                })
                .collect();
            let _ = writeln!(out, "- Піки активності: {}.", text.join("; "));
        }
        let forwards = counted(
            model,
            "SELECT forwarded_from, COUNT(*) AS n FROM telegram_post \
             WHERE channel_id = ?1 AND forwarded_from IS NOT NULL AND forwarded_from <> '' \
             GROUP BY forwarded_from ORDER BY n DESC LIMIT 10",
            &[&id],
        )?;
        let _ = writeln!(out, "- Найчастіше пересилає з: {}.", list(&forwards, 10));
        let own = format!("c/{id}%");
        let sites: Vec<(String, i64)> = counted(
            model,
            "SELECT l.value, COUNT(DISTINCT l.source_id) AS n FROM source_link l \
             JOIN telegram_post p ON p.source_id = l.source_id \
             WHERE p.channel_id = ?1 AND l.kind = 'site' GROUP BY l.value ORDER BY n DESC LIMIT 60",
            &[&id],
        )?
        .into_iter()
        .filter(|(value, _)| !is_generic_site(value))
        .collect();
        let _ = writeln!(
            out,
            "- Найчастіше посилається на сайти: {}.",
            list(&sites, 8)
        );
        let tg = counted(
            model,
            "SELECT l.value, COUNT(DISTINCT l.source_id) AS n FROM source_link l \
             JOIN telegram_post p ON p.source_id = l.source_id \
             WHERE p.channel_id = ?1 AND l.kind = 'tg' AND l.value NOT LIKE ?2 \
             GROUP BY l.value ORDER BY n DESC LIMIT 8",
            &[&id, &own],
        )?;
        let _ = writeln!(out, "- Посилається на канали: {}.", list(&tg, 8));
        let mentions = counted(
            model,
            "SELECT l.value, COUNT(DISTINCT l.source_id) AS n FROM source_link l \
             JOIN telegram_post p ON p.source_id = l.source_id \
             WHERE p.channel_id = ?1 AND l.kind = 'mention' GROUP BY l.value \
             ORDER BY n DESC LIMIT 8",
            &[&id],
        )?;
        let _ = writeln!(out, "- Згадує: {}.\n", list(&mentions, 8));
    }

    let shared = channel_overlaps(model, 40).map_err(|e| e.to_string())?;
    let _ = writeln!(out, "## Спільне між каналами\n");
    if shared.is_empty() {
        let _ = writeln!(
            out,
            "Спільного не знайдено (потрібно щонайменше два канали).\n"
        );
    } else {
        let _ = writeln!(out, "| Вид | Що | Канали (дописів) |\n|---|---|---|");
        for item in &shared {
            let kind = match item.kind.as_str() {
                "forward" => "джерело пересилань",
                "tg" => "канал",
                "site" => "сайт",
                "mention" => "згадка",
                _ => "однаковий текст",
            };
            let who: Vec<String> = item
                .channels
                .iter()
                .map(|c| format!("{} ({})", c.name, c.count))
                .collect();
            let _ = writeln!(
                out,
                "| {kind} | {} | {} |",
                cell(&item.value),
                cell(&who.join(", "))
            );
        }
        let _ = writeln!(out);
    }

    let found = discover_everywhere(model, registry, 60)?;
    let _ = writeln!(out, "## Суб'єкти в текстах, яких ще немає в базі\n");
    if found.is_empty() {
        let _ = writeln!(out, "Нових не знайдено.\n");
    } else {
        let _ = writeln!(out, "| Вид | Назва | Разів | Джерел |\n|---|---|---|---|");
        for item in &found {
            let kind = match item.kind.as_str() {
                "person" => "особа",
                "org" => "організація",
                _ => "код",
            };
            let _ = writeln!(
                out,
                "| {kind} | {} | {} | {} |",
                cell(&item.text),
                item.count,
                item.sources
            );
        }
        let _ = writeln!(out);
    }

    let pairs = subject_pairs(model, registry, 60)?;
    let _ = writeln!(out, "## Імовірні зв'язки між суб'єктами\n");
    let _ = writeln!(
        out,
        "Два суб'єкти названі в одному абзаці принаймні у трьох дописах. Це ще не зв'язок: \
         прочитайте приклад і за потреби створіть твердження з цитатою.\n"
    );
    if pairs.is_empty() {
        let _ = writeln!(out, "Не знайдено.\n");
    } else {
        let _ = writeln!(
            out,
            "| Суб'єкт А | Суб'єкт Б | Дописів | Каналів | Приклад |\n|---|---|---|---|---|"
        );
        for pair in &pairs {
            let mark = |known: bool| if known { " (є в базі)" } else { "" };
            let _ = writeln!(
                out,
                "| {}{} | {}{} | {} | {} | {} {} |",
                cell(&pair.a),
                mark(pair.known_a),
                cell(&pair.b),
                mark(pair.known_b),
                pair.posts,
                pair.channels,
                cell(&pair.example),
                pair.url
            );
        }
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{NewSource, Reliability, SourceKind};

    fn post(model: &Model, channel: i64, msg: i64, text: &str, forwarded: Option<&str>) {
        let url = format!("https://t.me/c/{channel}/{msg}");
        let id = model
            .add_source(&NewSource {
                kind: SourceKind::Media,
                title: "Допис",
                url: Some(&url),
                published_at: Some("2024-01-01"),
                reliability: Reliability::F,
                parent_id: None,
            })
            .unwrap();
        model.set_source_text(id, text).unwrap();
        model
            .set_telegram_post(id, channel, msg, forwarded)
            .unwrap();
    }

    #[test]
    fn peaks_are_the_months_far_above_the_usual() {
        assert!(peaks(&[5, 5, 5, 5, 5]).is_empty());
        assert_eq!(peaks(&[5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 60]), vec![10]);
    }

    #[test]
    fn the_overview_names_channels_shared_sources_and_pairs_of_subjects() {
        let model = Model::open_in_memory("tester").unwrap();
        model.set_telegram_channel(1, "Один").unwrap();
        model.set_telegram_channel(2, "Два").unwrap();
        model.add_telegram_seen(1, &[1, 2, 3, 4]).unwrap();
        model.add_telegram_seen(2, &[1, 2, 3]).unwrap();
        let pair = "Тимур Міндіч зустрівся з Германом Галущенком у Києві.";
        for msg in 1..=2 {
            post(&model, 1, msg, pair, Some("Спільний"));
        }
        post(&model, 2, 1, pair, Some("Спільний"));
        post(&model, 2, 2, "Герман Галущенко і Тимур Міндіч.", None);
        let report = build_overview(&model, None).unwrap();
        assert!(report.contains("# Огляд бази"));
        assert!(report.contains("### Один"));
        assert!(report.contains("### Два"));
        assert!(report.contains("Спільний"));
        assert!(report.contains("## Імовірні зв'язки між суб'єктами"));
        assert!(report.contains("Міндіч"));
        assert!(report.contains("Галущенк"));
    }
}
