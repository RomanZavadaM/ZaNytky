//! Import of a Telegram Desktop export (JSON) of a channel into the database.
//!
//! Every post becomes a source (kind `media`): the title is the first line, the date is the
//! date of the post, the address is `https://t.me/c/<channel id>/<post id>`, the full text is
//! kept as the text of the source, photos, videos and files become attachments (copied next to
//! the database). The import can be repeated: a post that is already known (same address)
//! is not added twice and its files are not copied twice.
//! Everything stays in the local database; nothing is sent anywhere.

use std::fmt;
use std::path::{Component, Path};

use serde_json::Value;

use crate::model::{Model, ModelError, NewSource, Reliability, SourceKind};

#[derive(Debug, Clone, Default, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TelegramSummary {
    pub channel: String,
    /// Posts looked at (service messages are not counted).
    pub messages: u32,
    pub sources_new: u32,
    pub sources_existing: u32,
    pub texts_added: u32,
    pub files_added: u32,
    /// Files named in the export that are not in the folder (not downloaded).
    pub files_missing: u32,
    /// Posts without text and without a usable file.
    pub skipped: u32,
}

#[derive(Debug)]
pub enum TelegramError {
    Read(String),
    Format(&'static str),
    Model(ModelError),
}

impl fmt::Display for TelegramError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TelegramError::Read(message) => write!(f, "не вдалося прочитати експорт: {message}"),
            TelegramError::Format(message) => write!(f, "unexpected export format: {message}"),
            TelegramError::Model(error) => write!(f, "{error}"),
        }
    }
}

impl std::error::Error for TelegramError {}

impl From<ModelError> for TelegramError {
    fn from(error: ModelError) -> Self {
        TelegramError::Model(error)
    }
}

/// The text of a post: a plain string, or a list of pieces (strings and objects with `text`;
/// the address of a link whose visible text differs is kept in brackets).
fn flatten(value: &Value) -> String {
    match value {
        Value::String(text) => text.clone(),
        Value::Array(pieces) => pieces
            .iter()
            .map(|piece| match piece {
                Value::String(text) => text.clone(),
                Value::Object(object) => {
                    let text = object.get("text").and_then(Value::as_str).unwrap_or("");
                    match object.get("href").and_then(Value::as_str) {
                        Some(href) if href != text && !href.is_empty() => {
                            format!("{text} ({href})")
                        }
                        _ => text.to_string(),
                    }
                }
                _ => String::new(),
            })
            .collect(),
        _ => String::new(),
    }
}

fn title_of(channel: &str, id: i64, text: &str) -> String {
    let first = text.lines().map(str::trim).find(|l| !l.is_empty());
    match first {
        Some(line) => {
            let short: String = line.chars().take(120).collect();
            format!("{channel}: {short}")
        }
        None => format!("{channel}: допис №{id}"),
    }
}

/// A path from the export that stays inside the export folder.
fn safe_relative(path: &str) -> Option<&Path> {
    let relative = Path::new(path);
    if relative.as_os_str().is_empty()
        || relative
            .components()
            .any(|c| !matches!(c, Component::Normal(_)))
    {
        return None;
    }
    Some(relative)
}

/// (relative path, mime) of the files of a post worth keeping (not thumbnails or stickers).
fn files_of(message: &Value) -> Vec<(String, Option<String>)> {
    let mut files = Vec::new();
    let media = message.get("media_type").and_then(Value::as_str);
    if let Some(photo) = message.get("photo").and_then(Value::as_str) {
        files.push((photo.to_string(), Some("image/jpeg".to_string())));
    }
    if media != Some("sticker") {
        if let Some(file) = message.get("file").and_then(Value::as_str) {
            let mime = message
                .get("mime_type")
                .and_then(Value::as_str)
                .map(str::to_string);
            files.push((file.to_string(), mime));
        }
    }
    files
}

/// (kind, value) of the address of a link: `tg` for a channel or chat of Telegram, `site`
/// for the host of any other address.
fn classify_url(url: &str) -> Option<(String, String)> {
    let rest = url
        .trim()
        .trim_start_matches("https://")
        .trim_start_matches("http://");
    let end = rest.find(['/', '?', '#']).unwrap_or(rest.len());
    let host = rest[..end].to_lowercase();
    let host = host
        .trim_start_matches("www.")
        .split(':')
        .next()?
        .to_string();
    if host.is_empty() || !host.contains('.') {
        return None;
    }
    if host == "t.me" || host == "telegram.me" || host == "telegram.dog" {
        let mut parts = rest[end..].trim_start_matches('/').split(['/', '?', '#']);
        let first = parts.next().unwrap_or("");
        if first.is_empty() || first.starts_with('+') || first == "joinchat" {
            return None;
        }
        if first == "c" {
            let id = parts.next().unwrap_or("");
            return if id.is_empty() {
                None
            } else {
                Some(("tg".to_string(), format!("c/{id}")))
            };
        }
        return Some(("tg".to_string(), first.to_lowercase()));
    }
    Some(("site".to_string(), host))
}

/// Links, sites and mentions of a post, from the structured parts of its text.
fn links_of(message: &Value) -> Vec<(String, String)> {
    let mut out = Vec::new();
    let Some(entities) = message.get("text_entities").and_then(Value::as_array) else {
        return out;
    };
    for entity in entities {
        let kind = entity.get("type").and_then(Value::as_str).unwrap_or("");
        let text = entity.get("text").and_then(Value::as_str).unwrap_or("");
        let found = match kind {
            "link" => classify_url(text),
            "text_link" => entity
                .get("href")
                .and_then(Value::as_str)
                .and_then(classify_url),
            "mention" => {
                let name = text.trim().trim_start_matches('@').to_lowercase();
                if name.is_empty() {
                    None
                } else {
                    Some(("mention".to_string(), name))
                }
            }
            _ => None,
        };
        if let Some(pair) = found {
            if !out.contains(&pair) {
                out.push(pair);
            }
        }
    }
    out
}

/// Read `result.json` of the export folder and write the posts into the model.
/// All of it is written, or none of it (files already copied stay: they are named by hash).
pub fn import_telegram_export(
    model: &Model,
    export_dir: &Path,
    attachments_dir: &Path,
    reliability: Reliability,
) -> Result<TelegramSummary, TelegramError> {
    let raw = std::fs::read_to_string(export_dir.join("result.json")).map_err(|e| {
        if export_dir.join("result.json").exists() {
            TelegramError::Read(e.to_string())
        } else {
            TelegramError::Read(
                "у папці немає result.json: оберіть саму папку експорту Telegram Desktop (JSON), де лежить result.json"
                    .to_string(),
            )
        }
    })?;
    let root: Value = serde_json::from_str(&raw).map_err(|e| TelegramError::Read(e.to_string()))?;
    drop(raw);
    let channel = root
        .get("name")
        .and_then(Value::as_str)
        .filter(|n| !n.trim().is_empty())
        .ok_or(TelegramError::Format("no channel name"))?
        .to_string();
    let channel_id = root
        .get("id")
        .and_then(Value::as_i64)
        .ok_or(TelegramError::Format("no channel id"))?;
    let messages = root
        .get("messages")
        .and_then(Value::as_array)
        .ok_or(TelegramError::Format("no messages"))?;

    let summary = model.in_transaction(|model| {
        let mut summary = TelegramSummary {
            channel: channel.clone(),
            ..TelegramSummary::default()
        };
        model.set_telegram_channel(channel_id, &channel)?;
        let seen: Vec<i64> = messages
            .iter()
            .filter_map(|m| m.get("id").and_then(Value::as_i64))
            .collect();
        model.add_telegram_seen(channel_id, &seen)?;
        for message in messages {
            if message.get("type").and_then(Value::as_str) != Some("message") {
                continue;
            }
            let Some(id) = message.get("id").and_then(Value::as_i64) else {
                continue;
            };
            summary.messages += 1;
            let mut body = flatten(message.get("text").unwrap_or(&Value::Null))
                .trim()
                .to_string();
            if let Some(from) = message.get("forwarded_from").and_then(Value::as_str) {
                body = format!("Переслано з: {from}\n\n{body}").trim().to_string();
            }
            let mut files = Vec::new();
            for (relative, mime) in files_of(message) {
                let path = safe_relative(&relative).map(|r| export_dir.join(r));
                match path {
                    Some(path) if path.is_file() => files.push((path, mime)),
                    _ => summary.files_missing += 1,
                }
            }
            if body.is_empty() && files.is_empty() {
                summary.skipped += 1;
                continue;
            }
            let url = format!("https://t.me/c/{channel_id}/{id}");
            let source_id = match model.find_source_by_url(&url)? {
                Some(existing) => {
                    summary.sources_existing += 1;
                    existing
                }
                None => {
                    summary.sources_new += 1;
                    let date = message.get("date").and_then(Value::as_str);
                    model.add_source(&NewSource {
                        kind: SourceKind::Media,
                        title: &title_of(&channel, id, &body),
                        url: Some(&url),
                        published_at: date.map(|d| d.get(..10).unwrap_or(d)),
                        reliability,
                        parent_id: None,
                    })?
                }
            };
            if !body.is_empty() && model.source_text(source_id)?.is_none() {
                model.set_source_text(source_id, &body)?;
                summary.texts_added += 1;
            }
            model.set_telegram_post(
                source_id,
                channel_id,
                id,
                message.get("forwarded_from").and_then(Value::as_str),
            )?;
            model.add_source_links(source_id, &links_of(message))?;
            for (path, mime) in files {
                let before = model.source_files(source_id)?.len();
                model.add_source_file(source_id, &path, attachments_dir, mime.as_deref())?;
                if model.source_files(source_id)?.len() > before {
                    summary.files_added += 1;
                }
            }
        }
        Ok(summary)
    })?;
    Ok(summary)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn export(dir: &Path) {
        std::fs::create_dir_all(dir.join("photos")).unwrap();
        std::fs::write(dir.join("photos").join("a.jpg"), b"photo-bytes").unwrap();
        let json = r#"{
  "name": "Канал", "type": "public_channel", "id": 777,
  "messages": [
    {"id": 1, "type": "service", "date": "2024-01-01T10:00:00", "action": "create_channel", "text": ""},
    {"id": 2, "type": "message", "date": "2024-01-02T10:00:00", "text": "Перший рядок\nдругий рядок"},
    {"id": 3, "type": "message", "date": "2024-01-03T10:00:00",
     "text": [{"type": "plain", "text": "Дивись "}, {"type": "text_link", "text": "тут", "href": "https://example.org/x"}, "."],
     "forwarded_from": "Інший"},
    {"id": 4, "type": "message", "date": "2024-01-04T10:00:00", "text": "", "photo": "photos/a.jpg"},
    {"id": 5, "type": "message", "date": "2024-01-05T10:00:00", "text": "", "photo": "photos/missing.jpg"},
    {"id": 6, "type": "message", "date": "2024-01-06T10:00:00", "text": "", "file": "stickers/s.webp", "media_type": "sticker"},
    {"id": 7, "type": "message", "date": "2024-01-07T10:00:00", "text": "Текст і ../ шлях", "photo": "../evil.jpg"}
  ]
}"#;
        std::fs::write(dir.join("result.json"), json).unwrap();
    }

    #[test]
    fn posts_become_sources_with_text_and_files_and_the_import_repeats_safely() {
        let model = Model::open_in_memory("tester").unwrap();
        let dir = tempfile::tempdir().unwrap();
        let export_dir = dir.path().join("export");
        export(&export_dir);
        let store = dir.path().join("attachments");

        let first = import_telegram_export(&model, &export_dir, &store, Reliability::F).unwrap();
        assert_eq!(first.channel, "Канал");
        assert_eq!(first.messages, 6);
        assert_eq!(first.sources_new, 4);
        assert_eq!(first.texts_added, 3);
        assert_eq!(first.files_added, 1);
        assert_eq!(first.files_missing, 2);
        assert_eq!(first.skipped, 2);

        let post2 = model
            .find_source_by_url("https://t.me/c/777/2")
            .unwrap()
            .unwrap();
        assert_eq!(model.source(post2).unwrap().title, "Канал: Перший рядок");
        assert_eq!(
            model.source_text(post2).unwrap().as_deref(),
            Some("Перший рядок\nдругий рядок")
        );
        let post3 = model
            .find_source_by_url("https://t.me/c/777/3")
            .unwrap()
            .unwrap();
        assert_eq!(
            model.source_text(post3).unwrap().as_deref(),
            Some("Переслано з: Інший\n\nДивись тут (https://example.org/x).")
        );
        let post4 = model
            .find_source_by_url("https://t.me/c/777/4")
            .unwrap()
            .unwrap();
        assert_eq!(model.source(post4).unwrap().title, "Канал: допис №4");
        assert_eq!(model.source_files(post4).unwrap().len(), 1);

        let second = import_telegram_export(&model, &export_dir, &store, Reliability::F).unwrap();
        assert_eq!(second.sources_new, 0);
        assert_eq!(second.sources_existing, 4);
        assert_eq!(second.texts_added, 0);
        assert_eq!(second.files_added, 0);
        assert_eq!(model.source_files(post4).unwrap().len(), 1);
    }

    #[test]
    fn a_folder_without_a_channel_is_refused() {
        let model = Model::open_in_memory("tester").unwrap();
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("result.json"), r#"{"messages": []}"#).unwrap();
        assert!(import_telegram_export(&model, dir.path(), dir.path(), Reliability::F).is_err());
    }

    #[test]
    fn addresses_are_sorted_into_channels_and_sites() {
        assert_eq!(
            classify_url("https://t.me/SomeChan/123"),
            Some(("tg".to_string(), "somechan".to_string()))
        );
        assert_eq!(
            classify_url("https://t.me/c/555/9"),
            Some(("tg".to_string(), "c/555".to_string()))
        );
        assert_eq!(classify_url("https://t.me/+secretinvite"), None);
        assert_eq!(
            classify_url("http://www.Example.org:8080/a?b=1"),
            Some(("site".to_string(), "example.org".to_string()))
        );
        assert_eq!(classify_url("not an address"), None);
    }

    #[test]
    fn the_import_records_forwards_links_and_the_ids_of_the_export() {
        use crate::channel::{channel_gaps, channel_graph, channel_list, channel_timeline};
        let model = Model::open_in_memory("tester").unwrap();
        let dir = tempfile::tempdir().unwrap();
        let json = r#"{
  "name": "Канал", "type": "public_channel", "id": 777,
  "messages": [
    {"id": 1, "type": "service", "date": "2024-01-01T10:00:00", "action": "create_channel", "text": ""},
    {"id": 2, "type": "message", "date": "2024-01-02T10:00:00", "text": "Один",
     "text_entities": [{"type": "link", "text": "https://example.org/a"},
                       {"type": "link", "text": "https://t.me/other/5"},
                       {"type": "mention", "text": "@Some_Account"}]},
    {"id": 3, "type": "message", "date": "2024-01-03T10:00:00", "text": "Два", "forwarded_from": "Інший канал"},
    {"id": 4, "type": "message", "date": "2024-01-04T10:00:00", "text": "Три", "forwarded_from": "Інший канал"},
    {"id": 8, "type": "message", "date": "2024-01-08T10:00:00", "text": "Вісім"}
  ]
}"#;
        std::fs::write(dir.path().join("result.json"), json).unwrap();
        let store = dir.path().join("attachments");
        import_telegram_export(&model, dir.path(), &store, Reliability::F).unwrap();

        let channels = channel_list(&model).unwrap();
        assert_eq!(channels.len(), 1);
        assert_eq!(channels[0].name, "Канал");
        assert_eq!(channels[0].posts, 4);
        assert_eq!((channels[0].first_id, channels[0].last_id), (1, 8));
        assert_eq!(channels[0].missing, 3);
        let gaps = channel_gaps(&model, 777).unwrap();
        assert_eq!(gaps.ranges.len(), 1);
        assert_eq!((gaps.ranges[0].from, gaps.ranges[0].to), (5, 7));

        let graph = channel_graph(&model, 777, &[]).unwrap();
        let titles: Vec<&str> = graph.nodes.iter().map(|n| n.title.as_str()).collect();
        assert!(titles.contains(&"Інший канал (2)"));
        assert!(titles.contains(&"example.org (1)"));
        assert!(titles.contains(&"other (1)"));
        assert!(titles.contains(&"some_account (1)"));

        // the same export again changes nothing
        import_telegram_export(&model, dir.path(), &store, Reliability::F).unwrap();
        assert_eq!(
            channel_graph(&model, 777, &[]).unwrap().nodes.len(),
            graph.nodes.len()
        );

        // expanding a node adds the posts it comes from
        let opened = channel_graph(&model, 777, &["fw:Інший канал".to_string()]).unwrap();
        let posts = opened
            .nodes
            .iter()
            .filter(|n| n.path.starts_with("post:"))
            .count();
        assert_eq!(posts, 2);

        // the timeline: one month, 4 posts, 2 forwarded from the same origin
        let months = channel_timeline(&model, 777).unwrap();
        assert_eq!(months.len(), 1);
        assert_eq!(months[0].month, "2024-01");
        assert_eq!((months[0].posts, months[0].forwarded), (4, 2));
        assert_eq!(months[0].top_origin.as_deref(), Some("Інший канал"));
    }
}
