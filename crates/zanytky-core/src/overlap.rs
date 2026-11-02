//! What several imported channels have in common (the "Overlaps" tab): the same source of
//! forwarded posts, the same linked channel, site or mentioned account, the same long text,
//! and direct references of one channel to another. These are hints for the investigator, not
//! proofs: two channels forwarding the same widely known source is usual.

use std::collections::hash_map::DefaultHasher;
use std::collections::{BTreeMap, HashMap};
use std::hash::{Hash, Hasher};

use serde::Serialize;

use crate::card::EntityGraph;
use crate::channel::Builder;
use crate::model::{Model, ModelError};

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SharedChannel {
    pub channel_id: i64,
    pub name: String,
    /// In how many posts of this channel it occurs.
    pub count: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Shared {
    /// `forward`, `tg`, `site`, `mention` or `text`.
    pub kind: String,
    /// The source, address or account; for `text` the beginning of the text.
    pub value: String,
    pub channels: Vec<SharedChannel>,
}

/// Shorter texts are too common ("thanks", greetings) to say anything.
const MIN_TEXT_CHARS: usize = 80;

/// Platforms, shorteners and payment pages that every channel links to: sharing them says
/// nothing, so they are left out of the overlaps.
#[rustfmt::skip]
const GENERIC_SITES: &[&str] = &[
    "youtube.com", "youtu.be", "facebook.com", "fb.com", "fb.me", "instagram.com", "twitter.com",
    "x.com", "t.co", "bit.ly", "tinyurl.com", "goo.gl", "google.com", "telegra.ph", "telegram.org",
    "t.me", "monobank.ua", "privatbank.ua", "paypal.com", "patreon.com", "buymeacoffee.com",
    "linktr.ee", "wikipedia.org", "tiktok.com", "threads.net", "whatsapp.com", "wa.me",
    "linkedin.com", "spotify.com", "apple.com", "podcasts.apple.com",
];

pub(crate) fn is_generic_site(host: &str) -> bool {
    GENERIC_SITES
        .iter()
        .any(|generic| host == *generic || host.ends_with(&format!(".{generic}")))
}

fn names(model: &Model) -> Result<BTreeMap<i64, String>, ModelError> {
    let mut stmt = model
        .connection()
        .prepare("SELECT channel_id, name FROM telegram_channel")?;
    let rows = stmt
        .query_map([], |r| Ok((r.get::<_, i64>(0)?, r.get::<_, String>(1)?)))?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(rows.into_iter().collect())
}

/// A text for comparison: without the line "Переслано з: ...", in lower case, with single
/// spaces.
fn comparable(text: &str) -> String {
    let body = match text.strip_prefix("Переслано з:") {
        Some(rest) => rest.split_once("\n\n").map(|(_, tail)| tail).unwrap_or(""),
        None => text,
    };
    body.split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .to_lowercase()
}

/// Everything that two or more channels have in common, the most widely shared first.
pub fn channel_overlaps(model: &Model, limit: usize) -> Result<Vec<Shared>, ModelError> {
    let names = names(model)?;
    let conn = model.connection();
    // (kind, value) -> channel -> count
    let mut found: HashMap<(String, String), BTreeMap<i64, u32>> = HashMap::new();

    let mut stmt = conn.prepare(
        "SELECT forwarded_from, channel_id, COUNT(*) FROM telegram_post \
         WHERE forwarded_from IS NOT NULL AND forwarded_from <> '' \
         GROUP BY forwarded_from, channel_id",
    )?;
    let rows = stmt
        .query_map([], |r| {
            Ok((
                r.get::<_, String>(0)?,
                r.get::<_, i64>(1)?,
                r.get::<_, i64>(2)?,
            ))
        })?
        .collect::<Result<Vec<_>, _>>()?;
    for (value, channel, count) in rows {
        found
            .entry(("forward".to_string(), value))
            .or_default()
            .insert(channel, count as u32);
    }

    let mut stmt = conn.prepare(
        "SELECT l.kind, l.value, p.channel_id, COUNT(DISTINCT p.source_id) \
         FROM source_link l JOIN telegram_post p ON p.source_id = l.source_id \
         GROUP BY l.kind, l.value, p.channel_id",
    )?;
    let rows = stmt
        .query_map([], |r| {
            Ok((
                r.get::<_, String>(0)?,
                r.get::<_, String>(1)?,
                r.get::<_, i64>(2)?,
                r.get::<_, i64>(3)?,
            ))
        })?
        .collect::<Result<Vec<_>, _>>()?;
    for (kind, value, channel, count) in rows {
        if kind == "site" && is_generic_site(&value) {
            continue;
        }
        found
            .entry((kind, value))
            .or_default()
            .insert(channel, count as u32);
    }

    // the same long text in several channels
    let mut texts: HashMap<u64, (String, BTreeMap<i64, u32>)> = HashMap::new();
    let mut stmt = conn.prepare(
        "SELECT p.channel_id, t.body FROM telegram_post p \
         JOIN source_text t ON t.source_id = p.source_id",
    )?;
    let rows = stmt.query_map([], |r| Ok((r.get::<_, i64>(0)?, r.get::<_, String>(1)?)))?;
    for row in rows {
        let (channel, body) = row?;
        let plain = comparable(&body);
        if plain.chars().count() < MIN_TEXT_CHARS {
            continue;
        }
        let mut hasher = DefaultHasher::new();
        plain.hash(&mut hasher);
        let entry = texts
            .entry(hasher.finish())
            .or_insert_with(|| (plain.chars().take(100).collect(), BTreeMap::new()));
        *entry.1.entry(channel).or_insert(0) += 1;
    }
    for (_, (sample, per_channel)) in texts {
        found.insert(("text".to_string(), sample), per_channel);
    }

    let mut shared: Vec<Shared> = found
        .into_iter()
        .filter(|(_, per_channel)| per_channel.len() >= 2)
        .map(|((kind, value), per_channel)| Shared {
            kind,
            value,
            channels: per_channel
                .into_iter()
                .map(|(channel_id, count)| SharedChannel {
                    channel_id,
                    name: names.get(&channel_id).cloned().unwrap_or_default(),
                    count,
                })
                .collect(),
        })
        .collect();
    let weight = |s: &Shared| {
        (
            s.channels.len(),
            s.channels.iter().map(|c| c.count).min().unwrap_or(0),
            s.channels.iter().map(|c| c.count).sum::<u32>(),
        )
    };
    shared.sort_by(|a, b| {
        weight(b)
            .cmp(&weight(a))
            .then_with(|| a.kind.cmp(&b.kind))
            .then_with(|| a.value.cmp(&b.value))
    });
    shared.truncate(limit);
    Ok(shared)
}

/// The channels and what they share as a graph, with the direct references of one channel to
/// another (a forward from a channel of the base, a link to it).
pub fn overlaps_graph(model: &Model, limit: usize) -> Result<EntityGraph, ModelError> {
    let names = names(model)?;
    let mut builder = Builder::new();
    if names.len() < 2 {
        return Ok(builder.graph);
    }
    for (id, name) in &names {
        builder.node(&format!("ch:{id}"), name.clone(), "channel");
    }
    for item in channel_overlaps(model, limit)? {
        let path = format!("sh:{}:{}", item.kind, item.value);
        let note_type = match item.kind.as_str() {
            "site" => "site",
            "mention" => "mention",
            "text" => "source",
            _ => "channel",
        };
        let short: String = item.value.chars().take(40).collect();
        builder.node(
            &path,
            format!("{short} ({})", item.channels.len()),
            note_type,
        );
        for channel in &item.channels {
            let centre = format!("ch:{}", channel.channel_id);
            let count = channel.count.to_string();
            if item.kind == "forward" {
                builder.line(&path, &centre, "forward", count);
            } else {
                builder.line(&centre, &path, &item.kind, count);
            }
        }
    }
    // direct references between channels of the base
    let conn = model.connection();
    for (id, name) in &names {
        let mut stmt = conn.prepare(
            "SELECT channel_id, COUNT(*) FROM telegram_post \
             WHERE forwarded_from = ?1 AND channel_id <> ?2 GROUP BY channel_id",
        )?;
        let forwards = stmt
            .query_map(rusqlite::params![name, id], |r| {
                Ok((r.get::<_, i64>(0)?, r.get::<_, i64>(1)?))
            })?
            .collect::<Result<Vec<_>, _>>()?;
        for (to, count) in forwards {
            builder.line(
                &format!("ch:{id}"),
                &format!("ch:{to}"),
                "forward",
                count.to_string(),
            );
        }
        let mut stmt = conn.prepare(
            "SELECT p.channel_id, COUNT(DISTINCT p.source_id) \
             FROM source_link l JOIN telegram_post p ON p.source_id = l.source_id \
             WHERE l.kind = 'tg' AND l.value = ?1 AND p.channel_id <> ?2 \
             GROUP BY p.channel_id",
        )?;
        let links = stmt
            .query_map(rusqlite::params![format!("c/{id}"), id], |r| {
                Ok((r.get::<_, i64>(0)?, r.get::<_, i64>(1)?))
            })?
            .collect::<Result<Vec<_>, _>>()?;
        for (from, count) in links {
            builder.line(
                &format!("ch:{from}"),
                &format!("ch:{id}"),
                "link",
                count.to_string(),
            );
        }
    }
    Ok(builder.graph)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{NewSource, Reliability, SourceKind};

    fn post(
        model: &Model,
        channel: i64,
        msg: i64,
        text: &str,
        forwarded: Option<&str>,
        links: &[(&str, &str)],
    ) {
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
        let links: Vec<(String, String)> = links
            .iter()
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect();
        model.add_source_links(id, &links).unwrap();
    }

    const LONG: &str = "Це досить довгий текст, який однаково з'являється в двох різних каналах, \
                        і тому порівняння за змістом має його знайти.";

    #[test]
    fn what_two_channels_share_is_found_and_drawn() {
        let model = Model::open_in_memory("tester").unwrap();
        model.set_telegram_channel(1, "Один").unwrap();
        model.set_telegram_channel(2, "Два").unwrap();
        post(
            &model,
            1,
            1,
            LONG,
            Some("Спільний"),
            &[("site", "example.org"), ("site", "youtube.com")],
        );
        post(
            &model,
            2,
            1,
            &format!("Переслано з: Хтось\n\n{LONG}"),
            Some("Спільний"),
            &[
                ("site", "example.org"),
                ("site", "m.youtube.com"),
                ("site", "youtube.com"),
                ("tg", "c/1"),
            ],
        );
        post(&model, 2, 2, "Короткий", Some("Один"), &[]);

        let shared = channel_overlaps(&model, 50).unwrap();
        let has = |kind: &str, value: &str| {
            shared
                .iter()
                .any(|s| s.kind == kind && s.value == value && s.channels.len() == 2)
        };
        assert!(has("forward", "Спільний"));
        assert!(has("site", "example.org"));
        // a platform every channel links to is left out
        assert!(!shared.iter().any(|s| s.value.contains("youtube")));
        assert!(shared.iter().any(|s| s.kind == "text"));
        // a link nobody else has is not shared
        assert!(!shared.iter().any(|s| s.value == "c/1"));

        let graph = overlaps_graph(&model, 50).unwrap();
        let line = |source: &str, target: &str, kind: &str| {
            graph
                .relations
                .iter()
                .any(|r| r.source == source && r.target == target && r.kind == kind)
        };
        // a direct link of channel 2 to channel 1 and a forward from channel 1 into channel 2
        assert!(line("ch:2", "ch:1", "link"));
        assert!(line("ch:1", "ch:2", "forward"));
    }

    #[test]
    fn one_channel_has_nothing_to_compare() {
        let model = Model::open_in_memory("tester").unwrap();
        model.set_telegram_channel(1, "Один").unwrap();
        post(&model, 1, 1, LONG, Some("Спільний"), &[]);
        post(&model, 1, 2, LONG, Some("Спільний"), &[]);
        assert!(channel_overlaps(&model, 50).unwrap().is_empty());
        assert!(overlaps_graph(&model, 50).unwrap().nodes.is_empty());
    }
}
