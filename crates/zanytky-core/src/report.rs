//! Investigator report (ADR-0005): the data of the model with the persons revealed,
//! as Markdown or Word, protected by the investigator's passphrase.
//!
//! Building a report needs the investigator's private identity (to reveal
//! persons). The result must be written outside the repositories.

use std::collections::BTreeSet;
use std::fmt;
use std::io::Cursor;

use docx_rs::{Docx, Paragraph, Run, Table, TableCell, TableRow};

use crate::model::{EntityKind, Model, ModelError};
use crate::registry::{Registry, RegistryError};
use crate::seal::{Identity, SealError};
use crate::wrap::wrap;

/// A piece of the report, independent of the output format.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Block {
    Heading(u8, String),
    Paragraph(String),
    Table {
        header: Vec<String>,
        rows: Vec<Vec<String>>,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Report {
    pub title: String,
    pub blocks: Vec<Block>,
}

#[derive(Debug, PartialEq, Eq)]
pub enum ReportError {
    Model(ModelError),
    Registry(RegistryError),
    Seal(SealError),
    Docx(String),
}

impl fmt::Display for ReportError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ReportError::Model(error) => write!(f, "{error}"),
            ReportError::Registry(error) => write!(f, "{error}"),
            ReportError::Seal(error) => write!(f, "{error}"),
            ReportError::Docx(message) => write!(f, "cannot build the Word file: {message}"),
        }
    }
}

impl std::error::Error for ReportError {}

impl From<ModelError> for ReportError {
    fn from(error: ModelError) -> Self {
        ReportError::Model(error)
    }
}

impl From<RegistryError> for ReportError {
    fn from(error: RegistryError) -> Self {
        ReportError::Registry(error)
    }
}

impl From<SealError> for ReportError {
    fn from(error: SealError) -> Self {
        ReportError::Seal(error)
    }
}

fn entity_name(
    model: &Model,
    registry: &Registry,
    identity: &Identity,
    id: i64,
) -> Result<String, ReportError> {
    let entity = model.entity(id)?;
    if entity.kind == EntityKind::Person {
        let token = entity.token.unwrap_or_default();
        return match registry.reveal(&token, identity) {
            Ok(sealed) => Ok(sealed.value),
            Err(RegistryError::NotFound) => Ok(token),
            Err(error) => Err(error.into()),
        };
    }
    let label = entity.label.unwrap_or_default();
    Ok(match entity.public_code {
        Some(code) => format!("{label} ({code})"),
        None => label,
    })
}

/// The report of everything in the model, persons revealed.
pub fn build_report(
    title: &str,
    model: &Model,
    registry: &Registry,
    identity: &Identity,
) -> Result<Report, ReportError> {
    let statements = model.statements()?;
    let mut rows = Vec::new();
    let mut source_ids = BTreeSet::new();
    for statement in &statements {
        let mut parties = Vec::new();
        for (entity, role) in &statement.parties {
            parties.push(format!(
                "{} ({role})",
                entity_name(model, registry, identity, *entity)?
            ));
        }
        source_ids.extend(statement.sources.iter().copied());
        let period = match (&statement.valid_from, &statement.valid_to) {
            (Some(from), Some(to)) => format!("{from} - {to}"),
            (Some(from), None) => format!("з {from}"),
            (None, Some(to)) => format!("до {to}"),
            (None, None) => String::new(),
        };
        rows.push(vec![
            statement.id.to_string(),
            statement.kind.as_str().to_string(),
            parties.join("; "),
            statement.value.clone().unwrap_or_default(),
            period,
            model.grade(statement.id)?.to_string(),
            statement.status.as_str().to_string(),
            statement.note.clone().unwrap_or_default(),
        ]);
    }

    let mut source_rows = Vec::new();
    for id in source_ids {
        let source = model.source(id)?;
        let mut chain = Vec::new();
        for ancestor in model.provenance(id)?.into_iter().skip(1) {
            chain.push(model.source(ancestor)?.title);
        }
        source_rows.push(vec![
            source.id.to_string(),
            source.kind.as_str().to_string(),
            source.title,
            source.url.unwrap_or_default(),
            source.reliability.as_str().to_string(),
            chain.join(" <- "),
        ]);
    }

    let strings = |items: &[&str]| items.iter().map(|s| s.to_string()).collect::<Vec<_>>();
    Ok(Report {
        title: title.to_string(),
        blocks: vec![
            Block::Heading(1, title.to_string()),
            Block::Paragraph(
                "Документ містить відкриті персональні дані. Відповідальність за його \
                 зберігання і використання несе розслідувач."
                    .to_string(),
            ),
            Block::Heading(2, "Твердження".to_string()),
            Block::Table {
                header: strings(&[
                    "№",
                    "Тип",
                    "Учасники",
                    "Значення",
                    "Період",
                    "Оцінка",
                    "Статус",
                    "Цитата і примітка",
                ]),
                rows,
            },
            Block::Heading(2, "Джерела".to_string()),
            Block::Table {
                header: strings(&["№", "Вид", "Назва", "Адреса", "Надійність", "Отримано від"]),
                rows: source_rows,
            },
        ],
    })
}

fn escape_cell(text: &str) -> String {
    text.replace('|', "\\|").replace(['\r', '\n'], " ")
}

/// The report as Markdown.
pub fn to_markdown(report: &Report) -> String {
    let mut out = String::new();
    for block in &report.blocks {
        match block {
            Block::Heading(level, text) => {
                out.push_str(&format!("{} {text}\n\n", "#".repeat(usize::from(*level))));
            }
            Block::Paragraph(text) => out.push_str(&format!("{text}\n\n")),
            Block::Table { header, rows } => {
                out.push_str(&format!("| {} |\n", header.join(" | ")));
                out.push_str(&format!("|{}\n", " --- |".repeat(header.len())));
                for row in rows {
                    let cells: Vec<String> = row.iter().map(|c| escape_cell(c)).collect();
                    out.push_str(&format!("| {} |\n", cells.join(" | ")));
                }
                out.push('\n');
            }
        }
    }
    out
}

fn cell(text: &str, bold: bool) -> TableCell {
    let mut run = Run::new().add_text(text);
    if bold {
        run = run.bold();
    }
    TableCell::new().add_paragraph(Paragraph::new().add_run(run))
}

/// The report as a Word file (`.docx`).
pub fn to_docx(report: &Report) -> Result<Vec<u8>, ReportError> {
    let mut docx = Docx::new();
    for block in &report.blocks {
        docx = match block {
            Block::Heading(level, text) => {
                let size = match level {
                    1 => 40,
                    2 => 32,
                    _ => 28,
                };
                docx.add_paragraph(
                    Paragraph::new().add_run(Run::new().add_text(text).bold().size(size)),
                )
            }
            Block::Paragraph(text) => {
                docx.add_paragraph(Paragraph::new().add_run(Run::new().add_text(text)))
            }
            Block::Table { header, rows } => {
                let mut table_rows = vec![TableRow::new(
                    header.iter().map(|h| cell(h, true)).collect(),
                )];
                for row in rows {
                    table_rows.push(TableRow::new(row.iter().map(|c| cell(c, false)).collect()));
                }
                docx.add_table(Table::new(table_rows))
            }
        };
    }
    let mut cursor = Cursor::new(Vec::new());
    docx.build()
        .pack(&mut cursor)
        .map_err(|error| ReportError::Docx(error.to_string()))?;
    Ok(cursor.into_inner())
}

/// The Markdown report, encrypted with the investigator's passphrase.
pub fn markdown_protected(report: &Report, passphrase: &str) -> Result<Vec<u8>, ReportError> {
    Ok(wrap(to_markdown(report).as_bytes(), passphrase)?)
}

/// The Word report, encrypted with the investigator's passphrase (a `.docx.age` file).
pub fn docx_protected(report: &Report, passphrase: &str) -> Result<Vec<u8>, ReportError> {
    Ok(wrap(&to_docx(report)?, passphrase)?)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::import::import_result;
    use crate::normalize::Kind;
    use crate::result::parse_result;
    use crate::wrap::unwrap;

    fn key() -> Vec<u8> {
        (0u8..32).collect()
    }

    /// A registry with one person and a model with one imported statement about them.
    fn setup() -> (Model, Registry, Identity) {
        let identity = Identity::generate();
        let registry = Registry::open_in_memory(&key(), vec![identity.to_public()]).unwrap();
        let token = registry.register(Kind::Name, "Іваненко Петро").unwrap();
        let json = format!(
            r#"{{
  "package": "pkg-1",
  "sources": [
    {{"ref": "s0", "kind": "registry", "title": "Реєстр юросіб", "url": "https://example.org/reg",
      "published_at": null, "reliability": "A", "received_from": null}}
  ],
  "entities": [
    {{"ref": "e1", "kind": "org", "label": "ТОВ Приклад", "token": null, "public_code": "12345678"}},
    {{"ref": "e2", "kind": "person", "label": null, "token": "{token}", "public_code": null}}
  ],
  "statements": [
    {{"kind": "owns", "value": "40%", "valid_from": null, "valid_to": null, "as_of": null,
      "parties": [{{"entity": "e2", "role": "owner"}}, {{"entity": "e1", "role": "owned"}}],
      "sources": ["s0"], "credibility": 2, "quote": "а|б у реєстрі", "note": null}}
  ]
}}"#
        );
        let model = Model::open_in_memory("tester").unwrap();
        import_result(&model, &parse_result(&json, "pkg-1").unwrap()).unwrap();
        (model, registry, identity)
    }

    #[test]
    fn the_report_reveals_persons_and_shows_grades() {
        let (model, registry, identity) = setup();
        let report = build_report("Справа 1", &model, &registry, &identity).unwrap();
        let markdown = to_markdown(&report);
        assert!(markdown.starts_with("# Справа 1"));
        assert!(markdown.contains("Іваненко Петро (owner)"));
        assert!(markdown.contains("ТОВ Приклад (12345678) (owned)"));
        assert!(markdown.contains("| A2 |"));
        assert!(markdown.contains("Реєстр юросіб"));
    }

    #[test]
    fn pipes_do_not_break_the_markdown_table() {
        let (model, registry, identity) = setup();
        let report = build_report("Справа", &model, &registry, &identity).unwrap();
        assert!(to_markdown(&report).contains("а\\|б у реєстрі"));
    }

    #[test]
    fn the_wrong_identity_cannot_build_the_report() {
        let (model, registry, _) = setup();
        let stranger = Identity::generate();
        assert!(build_report("Справа", &model, &registry, &stranger).is_err());
    }

    #[test]
    fn the_word_file_is_a_zip_package() {
        let (model, registry, identity) = setup();
        let report = build_report("Справа", &model, &registry, &identity).unwrap();
        let docx = to_docx(&report).unwrap();
        assert!(docx.starts_with(b"PK"));
        assert!(docx.len() > 1000);
    }

    #[test]
    fn protected_exports_need_the_passphrase() {
        let (model, registry, identity) = setup();
        let report = build_report("Справа", &model, &registry, &identity).unwrap();
        let markdown = markdown_protected(&report, "пароль").unwrap();
        assert!(!markdown
            .windows("Іваненко".len())
            .any(|w| w == "Іваненко".as_bytes()));
        let opened = unwrap(&markdown, "пароль").unwrap();
        assert!(String::from_utf8(opened)
            .unwrap()
            .contains("Іваненко Петро"));
        assert!(unwrap(&markdown, "інший").is_err());

        let docx = docx_protected(&report, "пароль").unwrap();
        assert!(!docx.starts_with(b"PK"));
        assert!(unwrap(&docx, "пароль").unwrap().starts_with(b"PK"));
    }
}
