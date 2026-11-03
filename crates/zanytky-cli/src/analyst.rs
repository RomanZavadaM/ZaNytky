//! Commands for an analyst, a person or a program: search the sources, read one, discover the
//! subjects in a text, look at channels and what they share, add a document, make subjects and
//! statements in a batch (the quotes are checked against the texts). Every reading command can
//! print JSON (`--json`) for a program to read.

use std::collections::HashMap;
use std::path::Path;

use zanytky_core::model::{EntityKind, NewSource, Reliability, SourceKind};
use zanytky_core::{
    channel_gaps, channel_list, channel_overlaps, channel_timeline, discover_everywhere,
    discover_in_text, entity_card, json_pretty, parse_statement_requests, source_list, source_view,
    subject_pairs,
};

use super::{home_dir, open_model, open_registry, read_text, Args, Out};

fn number<T: std::str::FromStr>(args: &Args, name: &str, default: T) -> Result<T, String> {
    match args.flag(name) {
        Some(text) => text
            .parse()
            .map_err(|_| format!("--{name}: потрібне число")),
        None => Ok(default),
    }
}

fn line(out: Out, text: &str) -> Result<(), String> {
    writeln!(out, "{text}").map_err(|e| e.to_string())
}

fn normalized(text: &str) -> String {
    text.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// `search ЗАПИТ [--limit N] [--offset N] [--json]`
pub(crate) fn cmd_search(args: &Args, out: Out) -> Result<(), String> {
    let home = home_dir(args)?;
    let model = open_model(&home)?;
    let query = args.pos(
        0,
        "запит (слова через пробіл; початки слів, будь-який регістр)",
    )?;
    let limit: u32 = number(args, "limit", 20)?;
    let offset: u32 = number(args, "offset", 0)?;
    let page = source_list(&model, query, offset, limit).map_err(|e| e.to_string())?;
    if args.has("json") {
        return line(out, &json_pretty(&page)?);
    }
    line(
        out,
        &format!("Знайдено: {} (показано з {})", page.total, offset),
    )?;
    for row in &page.rows {
        line(
            out,
            &format!(
                "#{} {} [{}] {}",
                row.id,
                row.published_at.as_deref().unwrap_or("?"),
                row.reliability,
                row.title
            ),
        )?;
        if let Some(url) = &row.url {
            line(out, &format!("    {url}"))?;
        }
        if !row.excerpt.is_empty() {
            line(out, &format!("    {}", row.excerpt))?;
        }
    }
    Ok(())
}

/// `source НОМЕР [--full] [--json]`
pub(crate) fn cmd_source(args: &Args, out: Out) -> Result<(), String> {
    let home = home_dir(args)?;
    let model = open_model(&home)?;
    let id: i64 = args
        .pos(0, "номер джерела")?
        .parse()
        .map_err(|_| "номер джерела: потрібне число".to_string())?;
    let view = source_view(&model, id).map_err(|e| e.to_string())?;
    let row = model.source(id).map_err(|e| e.to_string())?;
    let used: Vec<i64> = model
        .statements()
        .map_err(|e| e.to_string())?
        .into_iter()
        .filter(|s| s.sources.contains(&id))
        .map(|s| s.id)
        .collect();
    if args.has("json") {
        return line(out, &json_pretty(&view)?);
    }
    line(
        out,
        &format!(
            "#{id} [{}] надійність {} | {}",
            row.kind.as_str(),
            row.reliability.as_str(),
            row.title
        ),
    )?;
    if let Some(date) = &row.published_at {
        line(out, &format!("дата: {date}"))?;
    }
    if let Some(url) = &row.url {
        line(out, &format!("адреса: {url}"))?;
    }
    if !used.is_empty() {
        let list: Vec<String> = used.iter().map(|n| format!("#{n}")).collect();
        line(
            out,
            &format!("використано в твердженнях: {}", list.join(", ")),
        )?;
    }
    for file in &view.files {
        line(
            out,
            &format!(
                "файл: {} ({} байт, {})",
                file.name, file.size, file.rel_path
            ),
        )?;
    }
    match &view.text {
        Some(text) if args.has("full") => line(out, &format!("\n{text}")),
        Some(text) => {
            let head: String = text.chars().take(2000).collect();
            let more = if text.chars().count() > 2000 {
                "\n... (повний текст: --full)"
            } else {
                ""
            };
            line(out, &format!("\n{head}{more}"))
        }
        None => line(out, "(тексту немає)"),
    }
}

/// `discover (--source НОМЕР | --all) [--limit N] [--json]`
pub(crate) fn cmd_discover(args: &Args, out: Out) -> Result<(), String> {
    let home = home_dir(args)?;
    let model = open_model(&home)?;
    let registry = open_registry(&home).ok();
    let found = if let Some(source) = args.flag("source") {
        let id: i64 = source
            .parse()
            .map_err(|_| "--source: потрібне число".to_string())?;
        let text = source_view(&model, id)
            .map_err(|e| e.to_string())?
            .text
            .unwrap_or_default();
        discover_in_text(&model, registry.as_ref(), &text)
    } else if args.has("all") {
        discover_everywhere(&model, registry.as_ref(), number(args, "limit", 60)?)?
    } else {
        return Err("потрібне --source НОМЕР або --all".to_string());
    };
    if args.has("json") {
        return line(out, &json_pretty(&found)?);
    }
    for item in &found {
        let known = match item.known {
            Some(id) => format!("є в базі #{id}"),
            None => "новий".to_string(),
        };
        line(
            out,
            &format!(
                "{} | {} | разів {}, джерел {} | {known}",
                item.kind, item.text, item.count, item.sources
            ),
        )?;
    }
    Ok(())
}

/// `channels [--channel НОМЕР] [--json]`: the channels, or the timeline and gaps of one.
pub(crate) fn cmd_channels(args: &Args, out: Out) -> Result<(), String> {
    let home = home_dir(args)?;
    let model = open_model(&home)?;
    if let Some(id) = args.flag("channel") {
        let id: i64 = id
            .parse()
            .map_err(|_| "--channel: потрібне число".to_string())?;
        let months = channel_timeline(&model, id).map_err(|e| e.to_string())?;
        let gaps = channel_gaps(&model, id).map_err(|e| e.to_string())?;
        if args.has("json") {
            return line(out, &json_pretty(&(&months, &gaps))?);
        }
        for m in &months {
            let origin = m
                .top_origin
                .as_deref()
                .map(|o| format!(" (найбільше з «{o}»: {})", m.top_count))
                .unwrap_or_default();
            line(
                out,
                &format!(
                    "{} дописів {}, пересланих {}{origin}",
                    m.month, m.posts, m.forwarded
                ),
            )?;
        }
        let ranges: Vec<String> = gaps
            .ranges
            .iter()
            .map(|r| format!("{}-{}", r.from, r.to))
            .collect();
        return line(
            out,
            &format!(
                "немає в експорті номерів: {} ({})",
                gaps.missing,
                ranges.join(", ")
            ),
        );
    }
    let channels = channel_list(&model).map_err(|e| e.to_string())?;
    if args.has("json") {
        return line(out, &json_pretty(&channels)?);
    }
    for c in &channels {
        line(
            out,
            &format!(
                "{} | {} | дописів {} | {} - {} | немає номерів {}",
                c.channel_id,
                c.name,
                c.posts,
                c.first_date.as_deref().unwrap_or("?"),
                c.last_date.as_deref().unwrap_or("?"),
                c.missing
            ),
        )?;
    }
    Ok(())
}

/// `overlaps [--limit N] [--json]`
pub(crate) fn cmd_overlaps(args: &Args, out: Out) -> Result<(), String> {
    let home = home_dir(args)?;
    let model = open_model(&home)?;
    let shared = channel_overlaps(&model, number(args, "limit", 50)?).map_err(|e| e.to_string())?;
    if args.has("json") {
        return line(out, &json_pretty(&shared)?);
    }
    for item in &shared {
        let who: Vec<String> = item
            .channels
            .iter()
            .map(|c| format!("{} ({})", c.name, c.count))
            .collect();
        line(
            out,
            &format!("{} | {} | {}", item.kind, item.value, who.join(", ")),
        )?;
    }
    Ok(())
}

/// `pairs [--limit N] [--json]`: likely links, two subjects named in one paragraph.
pub(crate) fn cmd_pairs(args: &Args, out: Out) -> Result<(), String> {
    let home = home_dir(args)?;
    let model = open_model(&home)?;
    let registry = open_registry(&home).ok();
    let pairs = subject_pairs(&model, registry.as_ref(), number(args, "limit", 50)?)?;
    if args.has("json") {
        return line(out, &json_pretty(&pairs)?);
    }
    for pair in &pairs {
        line(
            out,
            &format!(
                "{} <-> {} | дописів {}, каналів {}\n    {} {}",
                pair.a, pair.b, pair.posts, pair.channels, pair.example, pair.url
            ),
        )?;
    }
    Ok(())
}

/// `entity НОМЕР`: the card of a subject as JSON (persons by pseudonym).
pub(crate) fn cmd_entity(args: &Args, out: Out) -> Result<(), String> {
    let home = home_dir(args)?;
    let model = open_model(&home)?;
    let id: i64 = args
        .pos(0, "номер суб'єкта")?
        .parse()
        .map_err(|_| "номер суб'єкта: потрібне число".to_string())?;
    let card = entity_card(&model, id, &HashMap::new()).map_err(|e| e.to_string())?;
    line(out, &json_pretty(&card)?)
}

/// `add-entity --kind person|org|group|asset|procurement|market|event|document --name НАЗВА
/// [--code ЄДРПОУ]`: a subject (a person goes to the registry by name, the base keeps a pseudonym).
pub(crate) fn cmd_add_entity(args: &Args, out: Out) -> Result<(), String> {
    let home = home_dir(args)?;
    let model = open_model(&home)?;
    let registry = open_registry(&home).ok();
    let kind = EntityKind::parse(args.need("kind")?).ok_or("невідомий вид суб'єкта")?;
    let id = zanytky_core::make::ensure_entity(
        &model,
        registry.as_ref(),
        kind,
        args.need("name")?,
        args.flag("code"),
    )?;
    line(out, &format!("Суб'єкт №{id}"))
}

/// `add-source --title НАЗВА [--kind К] [--reliability A-F] [--url У] [--date Д]
/// (--text-file ФАЙЛ | --text ТЕКСТ)`: a document, a page or a note as a source with its text.
pub(crate) fn cmd_add_source(args: &Args, out: Out) -> Result<(), String> {
    let home = home_dir(args)?;
    let model = open_model(&home)?;
    let kind =
        SourceKind::parse(args.flag("kind").unwrap_or("media")).ok_or("невідомий вид джерела")?;
    let reliability =
        Reliability::parse(args.flag("reliability").unwrap_or("C")).ok_or("невідома надійність")?;
    let url = args.flag("url");
    let existing = match url {
        Some(address) => model
            .find_source_by_url(address)
            .map_err(|e| e.to_string())?,
        None => None,
    };
    let id = match existing {
        Some(id) => id,
        None => model
            .add_source(&NewSource {
                kind,
                title: args.need("title")?,
                url,
                published_at: args.flag("date"),
                reliability,
                parent_id: None,
            })
            .map_err(|e| e.to_string())?,
    };
    let text = match args.flag("text-file") {
        Some(path) => Some(read_text(Path::new(path))?),
        None => args.flag("text").map(str::to_string),
    };
    if let Some(text) = text {
        if model.source_text(id).map_err(|e| e.to_string())?.is_none() {
            model
                .set_source_text(id, &text)
                .map_err(|e| e.to_string())?;
        }
    }
    line(out, &format!("Джерело №{id}"))
}

/// `make-statements --file ФАЙЛ.json [--no-verify]`: statements in a batch. The file is a list
/// of `{sourceId, kind, value, credibility, quote, note, parties: [{role, entityId} or
/// {role, kind, label, publicCode}]}`. Every quote must be in the text of its source.
pub(crate) fn cmd_make_statements(args: &Args, out: Out) -> Result<(), String> {
    let home = home_dir(args)?;
    let model = open_model(&home)?;
    let registry = open_registry(&home).ok();
    let requests = parse_statement_requests(&read_text(Path::new(args.need("file")?))?)?;
    if !args.has("no-verify") {
        for (index, request) in requests.iter().enumerate() {
            let text = source_view(&model, request.source_id)
                .map_err(|e| format!("запис {}: {e}", index + 1))?
                .text
                .unwrap_or_default();
            if !normalized(&text).contains(&normalized(&request.quote)) {
                return Err(format!(
                    "запис {}: цитати немає в тексті джерела №{} (нічого не записано)",
                    index + 1,
                    request.source_id
                ));
            }
        }
    }
    for request in &requests {
        let id = zanytky_core::create_statement(&model, registry.as_ref(), request)?;
        line(
            out,
            &format!(
                "Твердження №{id} створено ({}, джерело №{})",
                request.kind, request.source_id
            ),
        )?;
    }
    Ok(())
}
