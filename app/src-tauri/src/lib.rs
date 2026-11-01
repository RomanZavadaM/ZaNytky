//! ZaNytky desktop application: thin Tauri commands over `zanytky-core`.
//!
//! Secrets stay in files the user chooses: the system key in the data folder,
//! the investigator's private key wherever the investigator saves it.

use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use zanytky_core::model::{EntityKind, Model, Reliability, SourceKind, Status};
use zanytky_core::normalize::Kind;
use zanytky_core::pseudonym::{new_key, MIN_KEY_BYTES};
use zanytky_core::registry::{Registry, Resolution};
use zanytky_core::seal::{generate_identity, parse_identity, parse_recipient, Identity, Recipient};
use zanytky_core::wrap::wrap;
use zanytky_core::{
    backup_home, build_package_with_links, build_report, create_demo, docx_to_text, entity_card,
    entity_graph, entity_list, find_paths, import_result, parse_result, pdf, restore_home, review_with_links,
    to_docx, to_markdown, EntityCard, EntityGraph, EntityListRow, PathResult, SourceMeta,
};

type Res<T> = Result<T, String>;

fn io_error(path: &Path, error: std::io::Error) -> String {
    format!("{}: {error}", path.display())
}

fn read_text(path: &Path) -> Res<String> {
    fs::read_to_string(path).map_err(|e| io_error(path, e))
}

fn write_bytes(path: &Path, bytes: &[u8]) -> Res<()> {
    if let Some(parent) = path.parent() {
        if !parent.as_os_str().is_empty() {
            fs::create_dir_all(parent).map_err(|e| io_error(parent, e))?;
        }
    }
    fs::write(path, bytes).map_err(|e| io_error(path, e))
}

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

fn unhex(text: &str) -> Res<Vec<u8>> {
    let text = text.trim();
    if !text.is_ascii() || !text.len().is_multiple_of(2) {
        return Err("system key is damaged".to_string());
    }
    (0..text.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&text[i..i + 2], 16).map_err(|_| "system key is damaged".to_string()))
        .collect()
}

fn home_path(home: &str) -> Res<PathBuf> {
    if home.trim().is_empty() {
        return Err("data folder is not chosen".to_string());
    }
    Ok(PathBuf::from(home))
}

fn load_key(home: &Path) -> Res<Vec<u8>> {
    let key = unhex(&read_text(&home.join("system.key"))?)?;
    if key.len() < MIN_KEY_BYTES {
        return Err("system key is too short".to_string());
    }
    Ok(key)
}

fn load_recipients(home: &Path) -> Res<Vec<Recipient>> {
    let text = read_text(&home.join("recipients.txt"))?;
    let mut out = Vec::new();
    for line in text.lines().map(str::trim) {
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        out.push(parse_recipient(line).map_err(|e| e.to_string())?);
    }
    if out.is_empty() {
        return Err("no investigator public keys in recipients.txt".to_string());
    }
    Ok(out)
}

fn open_registry(home: &Path) -> Res<Registry> {
    Registry::open(&home.join("registry.db"), &load_key(home)?, load_recipients(home)?)
        .map_err(|e| e.to_string())
}

fn open_model(home: &Path) -> Res<Model> {
    Model::open(&home.join("model.db"), "app").map_err(|e| e.to_string())
}

fn read_identity(path: &str) -> Res<Identity> {
    let text = read_text(Path::new(path))?;
    let line = text
        .lines()
        .map(str::trim)
        .find(|l| l.starts_with("AGE-SECRET-KEY-"))
        .ok_or_else(|| "no private key in the file".to_string())?;
    parse_identity(line).map_err(|e| e.to_string())
}

fn clean(value: Option<String>) -> Option<String> {
    value.map(|v| v.trim().to_string()).filter(|v| !v.is_empty())
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct HomeInput {
    home: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct HomeStatus {
    initialised: bool,
    recipients: usize,
}

#[tauri::command]
fn home_status(input: HomeInput) -> Res<HomeStatus> {
    let home = home_path(&input.home)?;
    let initialised = home.join("system.key").exists();
    let recipients = match fs::read_to_string(home.join("recipients.txt")) {
        Ok(text) => text
            .lines()
            .map(str::trim)
            .filter(|l| !l.is_empty() && !l.starts_with('#'))
            .count(),
        Err(_) => 0,
    };
    Ok(HomeStatus {
        initialised,
        recipients,
    })
}

#[tauri::command]
fn init_home(input: HomeInput) -> Res<String> {
    let home = home_path(&input.home)?;
    fs::create_dir_all(&home).map_err(|e| io_error(&home, e))?;
    let key_path = home.join("system.key");
    if !key_path.exists() {
        let key = new_key().map_err(|e| e.to_string())?;
        write_bytes(&key_path, hex(&key).as_bytes())?;
    }
    let recipients = home.join("recipients.txt");
    if !recipients.exists() {
        write_bytes(&recipients, b"# investigator public keys (age1...), one per line\n")?;
    }
    Ok(home.display().to_string())
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct IdentityInput {
    home: String,
    path: String,
}

/// Create the investigator's key pair: the private key goes to `path`, the
/// public key is added to the recipients of the data folder.
#[tauri::command]
fn new_identity(input: IdentityInput) -> Res<String> {
    let home = home_path(&input.home)?;
    if !home.join("system.key").exists() {
        return Err("initialise the data folder first".to_string());
    }
    let path = PathBuf::from(&input.path);
    if path.exists() {
        return Err("the file already exists, it is not overwritten".to_string());
    }
    let (secret, public) = generate_identity();
    write_bytes(
        &path,
        format!("# investigator private key - keep it in a safe place\n{secret}\n").as_bytes(),
    )?;
    let recipients = home.join("recipients.txt");
    let mut text = read_text(&recipients)?;
    if !text.ends_with('\n') {
        text.push('\n');
    }
    text.push_str(&format!("{public}\n"));
    write_bytes(&recipients, text.as_bytes())?;
    Ok(public)
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct PathInput {
    path: String,
}

/// Check that a file holds an investigator's private key.
#[tauri::command]
fn check_identity(input: PathInput) -> Res<()> {
    read_identity(&input.path).map(|_| ())
}

#[tauri::command]
fn read_text_file(input: PathInput) -> Res<String> {
    let path = Path::new(&input.path);
    let is_docx = path
        .extension()
        .and_then(|e| e.to_str())
        .is_some_and(|e| e.eq_ignore_ascii_case("docx"));
    if is_docx {
        let bytes = fs::read(path).map_err(|e| io_error(path, e))?;
        docx_to_text(&bytes).map_err(|e| e.to_string())
    } else {
        read_text(path)
    }
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct PrepareInput {
    home: String,
    text: String,
    id: String,
    title: String,
    kind: String,
    url: Option<String>,
    date: Option<String>,
    context: Option<String>,
    identity_path: Option<String>,
}

#[derive(Serialize)]
struct CandidateOut {
    text: String,
    count: u32,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct PrepareOutput {
    deidentified: String,
    prompt: String,
    meta_json: String,
    replacements: u32,
    candidates: Vec<CandidateOut>,
}

#[tauri::command]
fn prepare_package(input: PrepareInput) -> Res<PrepareOutput> {
    let home = home_path(&input.home)?;
    if input.id.trim().is_empty() {
        return Err("package id is required".to_string());
    }
    let registry = open_registry(&home)?;
    let key = load_key(&home)?;
    let dictionary = match clean(input.identity_path) {
        Some(path) => registry
            .dictionary(&read_identity(&path)?)
            .map_err(|e| e.to_string())?,
        None => Vec::new(),
    };
    let links = registry.links().map_err(|e| e.to_string())?;
    let meta = SourceMeta {
        kind: SourceKind::parse(&input.kind).ok_or("unknown source kind")?,
        title: input.title,
        url: clean(input.url),
        published_at: clean(input.date),
        reliability: Reliability::C,
        received_from: None,
    };
    let context = clean(input.context);
    let package = build_package_with_links(
        input.id.trim(),
        &meta,
        &input.text,
        context.as_deref(),
        &key,
        &dictionary,
        &links,
    )
    .map_err(|e| e.to_string())?;
    Ok(PrepareOutput {
        deidentified: package.deidentified,
        prompt: package.prompt,
        meta_json: package.meta_json,
        replacements: package.replacements.iter().map(|r| r.count).sum(),
        candidates: package
            .candidates
            .into_iter()
            .map(|c| CandidateOut {
                text: c.text,
                count: c.count,
            })
            .collect(),
    })
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct ResolveInput {
    home: String,
    mention: String,
    /// "same", "new" or "not_person"
    action: String,
    person: Option<String>,
    name: Option<String>,
    ipn: Option<String>,
    phone: Option<String>,
    email: Option<String>,
}

#[tauri::command]
fn resolve_mention(input: ResolveInput) -> Res<Option<String>> {
    let registry = open_registry(&home_path(&input.home)?)?;
    let resolution = match input.action.as_str() {
        "not_person" => Resolution::NotAPerson,
        "same" => Resolution::SamePerson(
            clean(input.person).ok_or("the person's pseudonym is required")?,
        ),
        "new" => {
            let mut codes = Vec::new();
            for (kind, value) in [
                (Kind::Ipn, input.ipn),
                (Kind::Phone, input.phone),
                (Kind::Email, input.email),
            ] {
                if let Some(value) = clean(value) {
                    codes.push((kind, value));
                }
            }
            Resolution::NewPerson {
                name: clean(input.name).ok_or("the full name is required")?,
                codes,
            }
        }
        _ => return Err("unknown action".to_string()),
    };
    registry
        .resolve_mention(&input.mention, &resolution)
        .map_err(|e| e.to_string())
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct ResultInput {
    home: String,
    json: String,
    package: String,
}

#[tauri::command]
fn check_result(input: ResultInput) -> Res<String> {
    let file = parse_result(&input.json, input.package.trim()).map_err(|e| e.to_string())?;
    let model = open_model(&home_path(&input.home)?)?;
    review_with_links(&file, &model).map_err(|e| e.to_string())
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct ImportOut {
    sources: usize,
    entities_new: usize,
    entities_existing: usize,
    statements: usize,
}

#[tauri::command]
fn import_result_cmd(input: ResultInput) -> Res<ImportOut> {
    let home = home_path(&input.home)?;
    let file = parse_result(&input.json, input.package.trim()).map_err(|e| e.to_string())?;
    let summary = import_result(&open_model(&home)?, &file).map_err(|e| e.to_string())?;
    Ok(ImportOut {
        sources: summary.sources,
        entities_new: summary.entities_new,
        entities_existing: summary.entities_existing,
        statements: summary.statements,
    })
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct ReportInput {
    home: String,
    identity_path: String,
    title: String,
    /// "md", "docx" or "pdf"
    format: String,
    passphrase: Option<String>,
    /// Allow a report without a passphrase (open personal data).
    plain: bool,
    out_path: String,
}

#[tauri::command]
fn make_report(input: ReportInput) -> Res<String> {
    let home = home_path(&input.home)?;
    let registry = open_registry(&home)?;
    let model = open_model(&home)?;
    let identity = read_identity(&input.identity_path)?;
    let report = build_report(&input.title, &model, &registry, &identity).map_err(|e| e.to_string())?;
    let bytes = match input.format.as_str() {
        "md" => to_markdown(&report).into_bytes(),
        "docx" => to_docx(&report).map_err(|e| e.to_string())?,
        "pdf" => pdf::to_pdf(&report).map_err(|e| e.to_string())?,
        _ => return Err("unknown format".to_string()),
    };
    let mut target = PathBuf::from(&input.out_path);
    let bytes = match clean(input.passphrase) {
        Some(passphrase) => {
            let mut name = target.as_os_str().to_owned();
            name.push(".age");
            target = PathBuf::from(name);
            wrap(&bytes, &passphrase).map_err(|e| e.to_string())?
        }
        None if input.plain => bytes,
        None => return Err("a passphrase is required (or confirm a report without protection)".to_string()),
    };
    write_bytes(&target, &bytes)?;
    Ok(target.display().to_string())
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct CardInput {
    home: String,
    id: Option<i64>,
    /// With the investigator's key the names of persons are shown (on screen only).
    identity_path: Option<String>,
}

fn names_for(home: &Path, identity_path: &Option<String>) -> Res<std::collections::HashMap<String, String>> {
    let Some(path) = identity_path.as_deref().filter(|p| !p.trim().is_empty()) else {
        return Ok(std::collections::HashMap::new());
    };
    let identity = read_identity(path)?;
    let registry = open_registry(home)?;
    let dictionary = registry.dictionary(&identity).map_err(|e| e.to_string())?;
    let mut map: std::collections::HashMap<String, String> = dictionary
        .into_iter()
        .filter_map(|entry| entry.token.map(|token| (token, entry.text)))
        .collect();
    map.extend(
        registry
            .identifier_values(&identity)
            .map_err(|e| e.to_string())?,
    );
    Ok(map)
}

#[tauri::command]
fn list_entities(input: CardInput) -> Res<Vec<EntityListRow>> {
    let home = home_path(&input.home)?;
    let names = names_for(&home, &input.identity_path)?;
    entity_list(&open_model(&home)?, &names).map_err(|e| e.to_string())
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct LinkPathInput {
    home: String,
    from: i64,
    to: i64,
    max_depth: Option<u32>,
    include_refuted: Option<bool>,
    identity_path: Option<String>,
}

/// The shortest chains between two entities through shared statements.
#[tauri::command]
fn find_entity_paths(input: LinkPathInput) -> Res<PathResult> {
    let home = home_path(&input.home)?;
    let names = names_for(&home, &input.identity_path)?;
    let depth = input.max_depth.unwrap_or(6).clamp(1, 10);
    find_paths(
        &open_model(&home)?,
        input.from,
        input.to,
        depth,
        input.include_refuted.unwrap_or(false),
        &names,
    )
    .map_err(|e| e.to_string())
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct MergeInput {
    home: String,
    keep: i64,
    drop: i64,
}

/// Fold one entity into another (they are the same one). Can be undone.
#[tauri::command]
fn merge_two_entities(input: MergeInput) -> Res<()> {
    let home = home_path(&input.home)?;
    open_model(&home)?
        .merge_entities(input.keep, input.drop)
        .map_err(|e| e.to_string())
}

/// Split a folded entity off again.
#[tauri::command]
fn unmerge_one_entity(input: MergeInput) -> Res<()> {
    let home = home_path(&input.home)?;
    open_model(&home)?
        .unmerge_entity(input.drop)
        .map_err(|e| e.to_string())
}

/// The whole base as a graph for the "Graph" tab.
#[tauri::command]
fn get_entity_graph(input: CardInput) -> Res<EntityGraph> {
    let home = home_path(&input.home)?;
    let names = names_for(&home, &input.identity_path)?;
    entity_graph(&open_model(&home)?, &names).map_err(|e| e.to_string())
}

#[tauri::command]
fn get_entity_card(input: CardInput) -> Res<EntityCard> {
    let home = home_path(&input.home)?;
    let names = names_for(&home, &input.identity_path)?;
    let id = input.id.ok_or_else(|| "an entity id is required".to_string())?;
    entity_card(&open_model(&home)?, id, &names).map_err(|e| e.to_string())
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct DemoInput {
    folder: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct DemoOutput {
    home: String,
    identity: String,
    statements: usize,
}

/// A fictional case in an empty or new folder, to try the application without an AI.
#[tauri::command]
fn make_demo(input: DemoInput) -> Res<DemoOutput> {
    let info = create_demo(Path::new(&input.folder))?;
    Ok(DemoOutput {
        home: info.home.display().to_string(),
        identity: info.identity.display().to_string(),
        statements: info.statements,
    })
}

const MIN_BACKUP_PASSPHRASE: usize = 8;

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct BackupInput {
    home: String,
    out_path: String,
    passphrase: String,
}

/// Encrypted copy of the whole data folder; an existing file is never overwritten.
#[tauri::command]
fn make_backup(input: BackupInput) -> Res<String> {
    let home = home_path(&input.home)?;
    if input.passphrase.chars().count() < MIN_BACKUP_PASSPHRASE {
        return Err(format!("passphrase must have at least {MIN_BACKUP_PASSPHRASE} characters"));
    }
    let target = PathBuf::from(&input.out_path);
    if target.exists() {
        return Err("the backup file already exists, it is not overwritten".to_string());
    }
    let bytes = backup_home(&home, &input.passphrase).map_err(|e| e.to_string())?;
    write_bytes(&target, &bytes)?;
    Ok(target.display().to_string())
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct RestoreInput {
    file_path: String,
    dest: String,
    passphrase: String,
}

/// Restores a copy into an empty or new folder; returns the number of files.
#[tauri::command]
fn restore_backup(input: RestoreInput) -> Res<usize> {
    let source = PathBuf::from(&input.file_path);
    let data = fs::read(&source).map_err(|e| io_error(&source, e))?;
    restore_home(&data, &input.passphrase, Path::new(&input.dest)).map_err(|e| e.to_string())
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct StatementOut {
    id: i64,
    kind: String,
    parties: Vec<String>,
    value: String,
    grade: String,
    status: String,
    note: String,
}

fn party_label(model: &Model, id: i64, role: &str) -> Res<String> {
    let entity = model.entity(id).map_err(|e| e.to_string())?;
    let name = match entity.kind {
        // persons are shown only by pseudonym here; revealing needs the report
        EntityKind::Person => entity.token.unwrap_or_default(),
        _ => entity.label.unwrap_or_default(),
    };
    Ok(format!("{name} ({role})"))
}

#[tauri::command]
fn list_statements(input: HomeInput) -> Res<Vec<StatementOut>> {
    let model = open_model(&home_path(&input.home)?)?;
    let mut out = Vec::new();
    for statement in model.statements().map_err(|e| e.to_string())? {
        let mut parties = Vec::new();
        for (entity, role) in &statement.parties {
            parties.push(party_label(&model, *entity, role)?);
        }
        out.push(StatementOut {
            id: statement.id,
            kind: statement.kind.as_str().to_string(),
            parties,
            value: statement.value.clone().unwrap_or_default(),
            grade: model.grade(statement.id).map_err(|e| e.to_string())?.to_string(),
            status: statement.status.as_str().to_string(),
            note: statement.note.clone().unwrap_or_default(),
        });
    }
    Ok(out)
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct StatusInput {
    home: String,
    id: i64,
    /// "confirmed", "refuted" or "unverified"
    status: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct SourceTextInput {
    home: String,
    source_id: i64,
    text: String,
}

#[tauri::command]
fn set_source_text_cmd(input: SourceTextInput) -> Res<()> {
    let model = open_model(&home_path(&input.home)?)?;
    model
        .set_source_text(input.source_id, &input.text)
        .map_err(|e| e.to_string())
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct SourceFileInput {
    home: String,
    source_id: i64,
    path: String,
}

fn mime_of(path: &Path) -> Option<&'static str> {
    let extension = path.extension()?.to_str()?.to_lowercase();
    Some(match extension.as_str() {
        "pdf" => "application/pdf",
        "png" => "image/png",
        "jpg" | "jpeg" => "image/jpeg",
        "gif" => "image/gif",
        "webp" => "image/webp",
        "mp4" => "video/mp4",
        "mov" => "video/quicktime",
        "mp3" => "audio/mpeg",
        "ogg" | "oga" => "audio/ogg",
        "txt" => "text/plain",
        "html" | "htm" => "text/html",
        "json" => "application/json",
        "docx" => "application/vnd.openxmlformats-officedocument.wordprocessingml.document",
        _ => return None,
    })
}

#[tauri::command]
fn add_source_file_cmd(input: SourceFileInput) -> Res<()> {
    let home = home_path(&input.home)?;
    let model = open_model(&home)?;
    let file = PathBuf::from(&input.path);
    model
        .add_source_file(input.source_id, &file, &home.join("attachments"), mime_of(&file))
        .map(|_| ())
        .map_err(|e| e.to_string())
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct RevealInput {
    home: String,
    rel_path: String,
}

/// Shows an attachment in the system file manager (no extra plugin: the standard commands).
#[tauri::command]
fn reveal_source_file(input: RevealInput) -> Res<()> {
    let home = home_path(&input.home)?;
    let relative = Path::new(&input.rel_path);
    if relative
        .components()
        .any(|c| !matches!(c, std::path::Component::Normal(_)))
    {
        return Err("invalid attachment path".to_string());
    }
    let target = home.join("attachments").join(relative);
    if !target.is_file() {
        return Err("the attachment file is missing".to_string());
    }
    #[cfg(target_os = "windows")]
    let spawned = std::process::Command::new("explorer")
        .arg(format!("/select,{}", target.display()))
        .spawn();
    #[cfg(target_os = "macos")]
    let spawned = std::process::Command::new("open").arg("-R").arg(&target).spawn();
    #[cfg(all(not(target_os = "windows"), not(target_os = "macos")))]
    let spawned = std::process::Command::new("xdg-open")
        .arg(target.parent().unwrap_or(&home))
        .spawn();
    spawned.map(|_| ()).map_err(|e| e.to_string())
}

#[tauri::command]
fn set_statement_status(input: StatusInput) -> Res<()> {
    let model = open_model(&home_path(&input.home)?)?;
    let status = Status::parse(&input.status).ok_or("unknown status")?;
    model.set_status(input.id, status).map_err(|e| e.to_string())
}

/// Uses the web view runtime shipped next to the executable (folder `webview2`),
/// so the portable build does not depend on anything installed in Windows.
/// Without that folder the system runtime is used as before.
#[cfg(target_os = "windows")]
fn use_bundled_webview() {
    if let Ok(exe) = std::env::current_exe() {
        if let Some(dir) = exe.parent() {
            let runtime = dir.join("webview2");
            if runtime.join("msedgewebview2.exe").is_file() {
                std::env::set_var("WEBVIEW2_BROWSER_EXECUTABLE_FOLDER", &runtime);
            }
        }
    }
}

pub fn run() {
    #[cfg(target_os = "windows")]
    use_bundled_webview();
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![
            home_status,
            init_home,
            new_identity,
            read_text_file,
            check_identity,
            prepare_package,
            resolve_mention,
            check_result,
            import_result_cmd,
            make_report,
            list_statements,
            make_backup,
            make_demo,
            list_entities,
            get_entity_card,
            get_entity_graph,
            merge_two_entities,
            unmerge_one_entity,
            find_entity_paths,
            restore_backup,
            set_statement_status,
            set_source_text_cmd,
            add_source_file_cmd,
            reveal_source_file,
        ])
        .run(tauri::generate_context!())
        .expect("error while running ZaNytky");
}
