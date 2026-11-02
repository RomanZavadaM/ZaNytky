//! Analysis of an imported Telegram channel (the "Channels" tab): the list of channels, the
//! numbers of posts missing in the export (a gap in the sequence of ids), and the graph of
//! the channel's connections: where its posts were forwarded from, which channels and sites
//! it links to, whom it mentions. Everything comes from the data of the export; nothing is
//! asked from Telegram.

use serde::Serialize;

use crate::card::{EntityGraph, GraphEdgeOut, GraphNodeOut, GraphRelationOut};
use crate::model::{Model, ModelError};

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ChannelRow {
    pub channel_id: i64,
    pub name: String,
    /// Posts that became sources.
    pub posts: u32,
    /// Ids present in the export (posts and service messages).
    pub seen: u32,
    pub first_id: i64,
    pub last_id: i64,
    /// Ids inside the range that the export does not contain.
    pub missing: u32,
    pub first_date: Option<String>,
    pub last_date: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct GapRange {
    pub from: i64,
    pub to: i64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ChannelGaps {
    pub missing: u32,
    /// The first ranges (at most `MAX_RANGES`); `more` is how many ranges are not listed.
    pub ranges: Vec<GapRange>,
    pub more: u32,
}

const MAX_RANGES: usize = 200;

/// Ranges of ids that are absent between the smallest and the largest seen id.
fn ranges_of(ids: &[i64]) -> Vec<GapRange> {
    let mut ranges = Vec::new();
    for pair in ids.windows(2) {
        if pair[1] - pair[0] > 1 {
            ranges.push(GapRange {
                from: pair[0] + 1,
                to: pair[1] - 1,
            });
        }
    }
    ranges
}

fn seen_ids(model: &Model, channel_id: i64) -> Result<Vec<i64>, ModelError> {
    let mut stmt = model
        .connection()
        .prepare("SELECT msg_id FROM telegram_seen WHERE channel_id = ?1 ORDER BY msg_id")?;
    let rows = stmt.query_map([channel_id], |r| r.get::<_, i64>(0))?;
    Ok(rows.collect::<Result<Vec<_>, _>>()?)
}

pub fn channel_list(model: &Model) -> Result<Vec<ChannelRow>, ModelError> {
    let conn = model.connection();
    let mut stmt = conn.prepare("SELECT channel_id, name FROM telegram_channel ORDER BY name")?;
    let channels = stmt
        .query_map([], |r| Ok((r.get::<_, i64>(0)?, r.get::<_, String>(1)?)))?
        .collect::<Result<Vec<_>, _>>()?;
    let mut out = Vec::new();
    for (channel_id, name) in channels {
        let ids = seen_ids(model, channel_id)?;
        let missing: i64 = ranges_of(&ids).iter().map(|g| g.to - g.from + 1).sum();
        let (posts, first_date, last_date): (i64, Option<String>, Option<String>) = conn
            .query_row(
                "SELECT COUNT(*), MIN(s.published_at), MAX(s.published_at) \
                 FROM telegram_post p JOIN source s ON s.id = p.source_id \
                 WHERE p.channel_id = ?1",
                [channel_id],
                |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
            )?;
        out.push(ChannelRow {
            channel_id,
            name,
            posts: posts as u32,
            seen: ids.len() as u32,
            first_id: ids.first().copied().unwrap_or(0),
            last_id: ids.last().copied().unwrap_or(0),
            missing: missing as u32,
            first_date,
            last_date,
        });
    }
    Ok(out)
}

pub fn channel_gaps(model: &Model, channel_id: i64) -> Result<ChannelGaps, ModelError> {
    let ranges = ranges_of(&seen_ids(model, channel_id)?);
    let missing: i64 = ranges.iter().map(|g| g.to - g.from + 1).sum();
    let more = ranges.len().saturating_sub(MAX_RANGES) as u32;
    Ok(ChannelGaps {
        missing: missing as u32,
        ranges: ranges.into_iter().take(MAX_RANGES).collect(),
        more,
    })
}

/// How many nodes of each kind (forwarded-from, channels, sites, mentions) the graph shows.
const TOP: usize = 30;

/// The connections of one channel as a graph: the channel in the middle; around it the
/// sources of its forwarded posts, the channels and sites it links to, and the accounts it
/// mentions (the most frequent ones). A line says how many posts it comes from.
pub fn channel_graph(model: &Model, channel_id: i64) -> Result<EntityGraph, ModelError> {
    let conn = model.connection();
    let name: String = conn.query_row(
        "SELECT name FROM telegram_channel WHERE channel_id = ?1",
        [channel_id],
        |r| r.get(0),
    )?;
    let centre = format!("ch:{channel_id}");
    let mut nodes = vec![GraphNodeOut {
        path: centre.clone(),
        title: name.clone(),
        note_type: Some("channel".to_string()),
    }];
    let mut edges = Vec::new();
    let mut relations = Vec::new();

    let mut add = |path: String, label: &str, note_type: &str, count: i64, kind: &str, inward: bool| {
        nodes.push(GraphNodeOut {
            path: path.clone(),
            title: format!("{label} ({count})"),
            note_type: Some(note_type.to_string()),
        });
        let (source, target) = if inward {
            (path.clone(), centre.clone())
        } else {
            (centre.clone(), path.clone())
        };
        edges.push(GraphEdgeOut {
            source: source.clone(),
            target: target.clone(),
        });
        relations.push(GraphRelationOut {
            source,
            target,
            kind: kind.to_string(),
            grade: None,
            origin: None,
            status: None,
            evidence: Some(format!("{count}")),
        });
    };

    let mut stmt = conn.prepare(
        "SELECT forwarded_from, COUNT(*) AS n FROM telegram_post \
         WHERE channel_id = ?1 AND forwarded_from IS NOT NULL AND forwarded_from <> '' \
         GROUP BY forwarded_from ORDER BY n DESC, forwarded_from LIMIT ?2",
    )?;
    let forwarded = stmt
        .query_map(rusqlite::params![channel_id, TOP as i64], |r| {
            Ok((r.get::<_, String>(0)?, r.get::<_, i64>(1)?))
        })?
        .collect::<Result<Vec<_>, _>>()?;
    for (origin, count) in forwarded {
        add(format!("fw:{origin}"), &origin, "channel", count, "forward", true);
    }

    for (kind, note_type, relation) in [
        ("tg", "channel", "link"),
        ("site", "site", "link"),
        ("mention", "mention", "mention"),
    ] {
        let mut stmt = conn.prepare(
            "SELECT l.value, COUNT(DISTINCT l.source_id) AS n \
             FROM source_link l JOIN telegram_post p ON p.source_id = l.source_id \
             WHERE p.channel_id = ?1 AND l.kind = ?2 AND l.value NOT LIKE ?3 \
             GROUP BY l.value ORDER BY n DESC, l.value LIMIT ?4",
        )?;
        let own = format!("c/{channel_id}%");
        let found = stmt
            .query_map(
                rusqlite::params![channel_id, kind, own, TOP as i64],
                |r| Ok((r.get::<_, String>(0)?, r.get::<_, i64>(1)?)),
            )?
            .collect::<Result<Vec<_>, _>>()?;
        for (value, count) in found {
            add(
                format!("{kind}:{value}"),
                &value,
                note_type,
                count,
                relation,
                false,
            );
        }
    }
    Ok(EntityGraph {
        nodes,
        edges,
        relations,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn gaps_are_the_ids_missing_between_the_smallest_and_the_largest() {
        let ranges = ranges_of(&[1, 2, 3, 6, 7, 10]);
        assert_eq!(
            ranges,
            vec![GapRange { from: 4, to: 5 }, GapRange { from: 8, to: 9 }]
        );
        assert!(ranges_of(&[5]).is_empty());
        assert!(ranges_of(&[]).is_empty());
    }
}
