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
/// How many nodes an expanded node adds.
const EXPAND_TOP: usize = 15;

/// One connection of a channel: a node around it and the line to it.
struct Item {
    path: String,
    label: String,
    note_type: &'static str,
    count: i64,
    kind: &'static str,
    /// The line runs from the node to the channel (a forward), otherwise the other way.
    inward: bool,
}

/// The most frequent connections of a channel: where its posts were forwarded from, the
/// channels and sites it links to, the accounts it mentions.
fn connections(
    conn: &rusqlite::Connection,
    channel_id: i64,
    top: usize,
) -> Result<Vec<Item>, ModelError> {
    let mut items = Vec::new();
    let mut stmt = conn.prepare(
        "SELECT forwarded_from, COUNT(*) AS n FROM telegram_post \
         WHERE channel_id = ?1 AND forwarded_from IS NOT NULL AND forwarded_from <> '' \
         GROUP BY forwarded_from ORDER BY n DESC, forwarded_from LIMIT ?2",
    )?;
    let forwarded = stmt
        .query_map(rusqlite::params![channel_id, top as i64], |r| {
            Ok((r.get::<_, String>(0)?, r.get::<_, i64>(1)?))
        })?
        .collect::<Result<Vec<_>, _>>()?;
    for (origin, count) in forwarded {
        items.push(Item {
            path: format!("fw:{origin}"),
            label: origin,
            note_type: "channel",
            count,
            kind: "forward",
            inward: true,
        });
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
            .query_map(rusqlite::params![channel_id, kind, own, top as i64], |r| {
                Ok((r.get::<_, String>(0)?, r.get::<_, i64>(1)?))
            })?
            .collect::<Result<Vec<_>, _>>()?;
        for (value, count) in found {
            items.push(Item {
                path: format!("{kind}:{value}"),
                label: value,
                note_type,
                count,
                kind: relation,
                inward: false,
            });
        }
    }
    Ok(items)
}

/// The nodes and lines of a graph, without repeats.
pub(crate) struct Builder {
    pub(crate) graph: EntityGraph,
    seen: std::collections::HashSet<String>,
}

impl Builder {
    pub(crate) fn new() -> Self {
        Builder {
            graph: EntityGraph {
                nodes: Vec::new(),
                edges: Vec::new(),
                relations: Vec::new(),
            },
            seen: std::collections::HashSet::new(),
        }
    }

    pub(crate) fn node(&mut self, path: &str, title: String, note_type: &str) {
        if self.seen.insert(path.to_string()) {
            self.graph.nodes.push(GraphNodeOut {
                path: path.to_string(),
                title,
                note_type: Some(note_type.to_string()),
            });
        }
    }

    pub(crate) fn line(&mut self, source: &str, target: &str, kind: &str, evidence: String) {
        let exists = self
            .graph
            .relations
            .iter()
            .any(|r| r.source == source && r.target == target && r.kind == kind);
        if exists {
            return;
        }
        self.graph.edges.push(GraphEdgeOut {
            source: source.to_string(),
            target: target.to_string(),
        });
        self.graph.relations.push(GraphRelationOut {
            source: source.to_string(),
            target: target.to_string(),
            kind: kind.to_string(),
            grade: None,
            origin: None,
            status: None,
            evidence: Some(evidence),
        });
    }

    fn around(&mut self, centre: &str, items: Vec<Item>) {
        for item in items {
            self.node(
                &item.path,
                format!("{} ({})", item.label, item.count),
                item.note_type,
            );
            let (source, target) = if item.inward {
                (item.path.as_str(), centre)
            } else {
                (centre, item.path.as_str())
            };
            self.line(source, target, item.kind, format!("{}", item.count));
        }
    }
}

fn channel_name(conn: &rusqlite::Connection, channel_id: i64) -> Result<String, ModelError> {
    Ok(conn.query_row(
        "SELECT name FROM telegram_channel WHERE channel_id = ?1",
        [channel_id],
        |r| r.get(0),
    )?)
}

/// The most recent posts of the channel that match `condition` (a WHERE part on `p`, `l`).
fn recent_posts(
    conn: &rusqlite::Connection,
    sql_from_where: &str,
    params: &[&dyn rusqlite::ToSql],
) -> Result<Vec<(i64, String)>, ModelError> {
    let mut stmt = conn.prepare(&format!(
        "SELECT DISTINCT s.id, COALESCE(s.published_at, ''), s.title {sql_from_where} \
         ORDER BY s.published_at DESC, s.id DESC LIMIT {EXPAND_TOP}"
    ))?;
    let rows = stmt
        .query_map(params, |r| {
            let id: i64 = r.get(0)?;
            let date: String = r.get(1)?;
            let title: String = r.get(2)?;
            let short: String = title.chars().take(60).collect();
            Ok((id, format!("{date} {short}").trim().to_string()))
        })?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(rows)
}

/// Add what hangs on an expanded node: the posts it comes from, the other channels of the
/// base that have the same connection, and (for a channel) its own connections.
fn expand(
    builder: &mut Builder,
    conn: &rusqlite::Connection,
    channel_id: i64,
    node: &str,
) -> Result<(), ModelError> {
    if !builder.seen.contains(node) {
        return Ok(());
    }
    let Some((prefix, value)) = node.split_once(':') else {
        return Ok(());
    };
    let (posts, others): (Vec<(i64, String)>, Vec<i64>) = match prefix {
        "fw" => {
            let posts = recent_posts(
                conn,
                "FROM telegram_post p JOIN source s ON s.id = p.source_id \
                 WHERE p.channel_id = ?1 AND p.forwarded_from = ?2",
                &[&channel_id, &value],
            )?;
            let mut stmt = conn.prepare(
                "SELECT DISTINCT channel_id FROM telegram_post \
                 WHERE forwarded_from = ?1 AND channel_id <> ?2",
            )?;
            let others = stmt
                .query_map(rusqlite::params![value, channel_id], |r| r.get::<_, i64>(0))?
                .collect::<Result<Vec<_>, _>>()?;
            // the origin may itself be a channel of the base
            let mut same =
                conn.prepare("SELECT channel_id FROM telegram_channel WHERE name = ?1")?;
            let same = same
                .query_map([value], |r| r.get::<_, i64>(0))?
                .collect::<Result<Vec<_>, _>>()?;
            for id in same {
                let name = channel_name(conn, id)?;
                builder.node(&format!("ch:{id}"), name, "channel");
                builder.line(node, &format!("ch:{id}"), "same", "".to_string());
            }
            (posts, others)
        }
        "tg" | "site" | "mention" => {
            let posts = recent_posts(
                conn,
                "FROM source_link l JOIN telegram_post p ON p.source_id = l.source_id \
                 JOIN source s ON s.id = p.source_id \
                 WHERE p.channel_id = ?1 AND l.kind = ?2 AND l.value = ?3",
                &[&channel_id, &prefix, &value],
            )?;
            let mut stmt = conn.prepare(
                "SELECT DISTINCT p.channel_id FROM source_link l \
                 JOIN telegram_post p ON p.source_id = l.source_id \
                 WHERE l.kind = ?1 AND l.value = ?2 AND p.channel_id <> ?3",
            )?;
            let others = stmt
                .query_map(rusqlite::params![prefix, value, channel_id], |r| {
                    r.get::<_, i64>(0)
                })?
                .collect::<Result<Vec<_>, _>>()?;
            (posts, others)
        }
        "ch" => {
            let id: i64 = value.parse().unwrap_or(-1);
            let items = connections(conn, id, EXPAND_TOP)?;
            builder.around(node, items);
            return Ok(());
        }
        _ => return Ok(()),
    };
    for (id, title) in posts {
        let path = format!("post:{id}");
        builder.node(&path, title, "source");
        builder.line(node, &path, "post", String::new());
    }
    for id in others {
        let path = format!("ch:{id}");
        let name = channel_name(conn, id)?;
        builder.node(&path, name, "channel");
        builder.line(node, &path, "shared", String::new());
    }
    Ok(())
}

/// The connections of one channel as a graph: the channel in the middle; around it the
/// sources of its forwarded posts, the channels and sites it links to, and the accounts it
/// mentions (the most frequent ones). A line says how many posts it comes from. Every node in
/// `expanded` is opened: its posts, the other channels of the base with the same connection
/// (the hidden links between channels), and for a channel its own connections.
pub fn channel_graph(
    model: &Model,
    channel_id: i64,
    expanded: &[String],
) -> Result<EntityGraph, ModelError> {
    let conn = model.connection();
    let centre = format!("ch:{channel_id}");
    let mut builder = Builder::new();
    builder.node(&centre, channel_name(conn, channel_id)?, "channel");
    let items = connections(conn, channel_id, TOP)?;
    builder.around(&centre, items);
    for node in expanded {
        expand(&mut builder, conn, channel_id, node)?;
    }
    Ok(builder.graph)
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MonthRow {
    /// `YYYY-MM`.
    pub month: String,
    pub posts: u32,
    /// Of them forwarded from another channel.
    pub forwarded: u32,
    pub top_origin: Option<String>,
    pub top_count: u32,
}

/// Posts of the channel by month, with the most frequent origin of the forwarded ones.
pub fn channel_timeline(model: &Model, channel_id: i64) -> Result<Vec<MonthRow>, ModelError> {
    let conn = model.connection();
    let mut stmt = conn.prepare(
        "SELECT substr(s.published_at, 1, 7), COUNT(*), \
         SUM(p.forwarded_from IS NOT NULL AND p.forwarded_from <> '') \
         FROM telegram_post p JOIN source s ON s.id = p.source_id \
         WHERE p.channel_id = ?1 AND s.published_at IS NOT NULL \
         GROUP BY 1 ORDER BY 1",
    )?;
    let mut rows = stmt
        .query_map([channel_id], |r| {
            Ok(MonthRow {
                month: r.get(0)?,
                posts: r.get::<_, i64>(1)? as u32,
                forwarded: r.get::<_, i64>(2)? as u32,
                top_origin: None,
                top_count: 0,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;
    let mut stmt = conn.prepare(
        "SELECT substr(s.published_at, 1, 7) AS m, p.forwarded_from, COUNT(*) AS n \
         FROM telegram_post p JOIN source s ON s.id = p.source_id \
         WHERE p.channel_id = ?1 AND s.published_at IS NOT NULL \
         AND p.forwarded_from IS NOT NULL AND p.forwarded_from <> '' \
         GROUP BY m, p.forwarded_from ORDER BY m, n DESC, p.forwarded_from",
    )?;
    let origins = stmt
        .query_map([channel_id], |r| {
            Ok((
                r.get::<_, String>(0)?,
                r.get::<_, String>(1)?,
                r.get::<_, i64>(2)?,
            ))
        })?
        .collect::<Result<Vec<_>, _>>()?;
    for (month, origin, n) in origins {
        if let Some(row) = rows.iter_mut().find(|row| row.month == month) {
            if row.top_origin.is_none() {
                row.top_origin = Some(origin);
                row.top_count = n as u32;
            }
        }
    }
    Ok(rows)
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
