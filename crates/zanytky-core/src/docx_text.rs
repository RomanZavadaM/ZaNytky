//! Plain text from a Word (`.docx`) file, so a document can be de-identified
//! without copying its text by hand.
//!
//! Only the visible text of the main document part is read (paragraphs, tabs,
//! line breaks, table cells). Deleted text of tracked changes, comments,
//! headers and footers are not included. Nothing is executed or fetched.

use std::fmt;
use std::io::{Cursor, Read};

use zip::ZipArchive;

/// The unpacked document part may not be larger than this (protection
/// against compressed files that expand enormously).
const MAX_XML_BYTES: u64 = 50 * 1024 * 1024;

#[derive(Debug)]
pub enum DocxError {
    /// Not a ZIP container.
    NotDocx(String),
    /// The container has no main document part.
    NoDocument,
    TooLarge,
    Xml(String),
}

impl fmt::Display for DocxError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DocxError::NotDocx(m) => write!(f, "це не файл Word (.docx): {m}"),
            DocxError::NoDocument => write!(f, "у файлі немає основної частини документа"),
            DocxError::TooLarge => write!(f, "документ завеликий для читання"),
            DocxError::Xml(m) => write!(f, "пошкоджений вміст документа: {m}"),
        }
    }
}

impl std::error::Error for DocxError {}

/// Extract the text; paragraphs are separated by a line break.
pub fn docx_to_text(bytes: &[u8]) -> Result<String, DocxError> {
    let mut archive =
        ZipArchive::new(Cursor::new(bytes)).map_err(|e| DocxError::NotDocx(e.to_string()))?;
    let file = archive
        .by_name("word/document.xml")
        .map_err(|_| DocxError::NoDocument)?;
    if file.size() > MAX_XML_BYTES {
        return Err(DocxError::TooLarge);
    }
    let mut xml = String::new();
    file.take(MAX_XML_BYTES + 1)
        .read_to_string(&mut xml)
        .map_err(|e| DocxError::Xml(e.to_string()))?;
    if xml.len() as u64 > MAX_XML_BYTES {
        return Err(DocxError::TooLarge);
    }
    let document = roxmltree::Document::parse(&xml).map_err(|e| DocxError::Xml(e.to_string()))?;

    let mut out = String::new();
    for paragraph in document
        .descendants()
        .filter(|n| n.is_element() && n.tag_name().name() == "p")
    {
        let mut line = String::new();
        for node in paragraph.descendants().filter(|n| n.is_element()) {
            match node.tag_name().name() {
                "t" => line.push_str(node.text().unwrap_or("")),
                "tab" => line.push('\t'),
                "br" | "cr" => line.push('\n'),
                _ => {}
            }
        }
        out.push_str(line.trim_end());
        out.push('\n');
    }
    Ok(out.trim().to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use docx_rs::{Docx, Paragraph, Run, Table, TableCell, TableRow};

    fn build(docx: Docx) -> Vec<u8> {
        let mut cursor = Cursor::new(Vec::new());
        docx.build().pack(&mut cursor).unwrap();
        cursor.into_inner()
    }

    #[test]
    fn reads_paragraphs_in_order() {
        let docx = Docx::new()
            .add_paragraph(Paragraph::new().add_run(Run::new().add_text("Перший абзац")))
            .add_paragraph(Paragraph::new().add_run(Run::new().add_text("Другий: Іваненко Петро")));
        let text = docx_to_text(&build(docx)).unwrap();
        assert_eq!(text, "Перший абзац\nДругий: Іваненко Петро");
    }

    #[test]
    fn joins_runs_of_one_paragraph() {
        let paragraph = Paragraph::new()
            .add_run(Run::new().add_text("Іван"))
            .add_run(Run::new().add_text("енко"));
        let text = docx_to_text(&build(Docx::new().add_paragraph(paragraph))).unwrap();
        assert_eq!(text, "Іваненко");
    }

    #[test]
    fn reads_table_cells() {
        let cell = |t: &str| {
            TableCell::new().add_paragraph(Paragraph::new().add_run(Run::new().add_text(t)))
        };
        let table = Table::new(vec![TableRow::new(vec![cell("ЄДРПОУ"), cell("12345678")])]);
        let text = docx_to_text(&build(Docx::new().add_table(table))).unwrap();
        assert!(text.contains("ЄДРПОУ"));
        assert!(text.contains("12345678"));
    }

    #[test]
    fn a_plain_text_file_is_not_a_docx() {
        let err = docx_to_text("просто текст".as_bytes()).unwrap_err();
        assert!(matches!(err, DocxError::NotDocx(_)));
    }

    #[test]
    fn a_zip_without_a_document_part_is_refused() {
        use std::io::Write;
        let mut cursor = Cursor::new(Vec::new());
        {
            let mut writer = zip::ZipWriter::new(&mut cursor);
            writer
                .start_file("other.txt", zip::write::SimpleFileOptions::default())
                .unwrap();
            writer.write_all(b"x").unwrap();
            writer.finish().unwrap();
        }
        let err = docx_to_text(&cursor.into_inner()).unwrap_err();
        assert!(matches!(err, DocxError::NoDocument));
    }
}
