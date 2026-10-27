//! The report as a PDF (ADR-0005). The embedded Noto Sans font (SIL OFL 1.1,
//! see `assets/OFL.txt`) provides the Ukrainian letters.

use genpdf::elements::{FrameCellDecorator, Paragraph, TableLayout};
use genpdf::fonts::{FontData, FontFamily};
use genpdf::style::Style;
use genpdf::{Document, Element, SimplePageDecorator, Size};

use crate::report::{Block, Report, ReportError};
use crate::wrap::wrap;

const FONT: &[u8] = include_bytes!("../assets/NotoSans-Regular.ttf");

fn pdf_error<E: std::fmt::Display>(error: E) -> ReportError {
    ReportError::Docx(format!("pdf: {error}"))
}

fn weights(columns: usize) -> Vec<usize> {
    match columns {
        8 => vec![1, 2, 6, 2, 3, 2, 2, 6],
        6 => vec![1, 2, 5, 6, 2, 5],
        n => vec![1; n.max(1)],
    }
}

/// The report as a PDF file (landscape A4, tables in small type).
pub fn to_pdf(report: &Report) -> Result<Vec<u8>, ReportError> {
    let font = FontData::new(FONT.to_vec(), None).map_err(pdf_error)?;
    let family = FontFamily {
        regular: font.clone(),
        bold: font.clone(),
        italic: font.clone(),
        bold_italic: font,
    };
    let mut doc = Document::new(family);
    doc.set_title(report.title.clone());
    doc.set_paper_size(Size::new(297, 210));
    let mut decorator = SimplePageDecorator::new();
    decorator.set_margins(10);
    doc.set_page_decorator(decorator);

    let small = Style::new().with_font_size(8);
    for block in &report.blocks {
        match block {
            Block::Heading(level, text) => {
                let size = match level {
                    1 => 18,
                    2 => 14,
                    _ => 12,
                };
                doc.push(Paragraph::new(text.clone()).styled(Style::new().with_font_size(size)));
            }
            Block::Paragraph(text) => doc.push(Paragraph::new(text.clone())),
            Block::Table { header, rows } => {
                let mut table = TableLayout::new(weights(header.len()));
                table.set_cell_decorator(FrameCellDecorator::new(true, true, false));
                let mut head = table.row();
                for cell in header {
                    head = head.element(Paragraph::new(cell.clone()).styled(small));
                }
                head.push().map_err(pdf_error)?;
                for row in rows {
                    let mut line = table.row();
                    for cell in row {
                        line = line.element(Paragraph::new(cell.clone()).styled(small));
                    }
                    line.push().map_err(pdf_error)?;
                }
                doc.push(table);
            }
        }
    }
    let mut out = Vec::new();
    doc.render(&mut out).map_err(pdf_error)?;
    Ok(out)
}

/// The PDF report, encrypted with the investigator's passphrase (a `.pdf.age` file).
pub fn pdf_protected(report: &Report, passphrase: &str) -> Result<Vec<u8>, ReportError> {
    Ok(wrap(&to_pdf(report)?, passphrase)?)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::wrap::unwrap;

    fn report() -> Report {
        Report {
            title: "Справа".to_string(),
            blocks: vec![
                Block::Heading(1, "Справа".to_string()),
                Block::Paragraph("Іваненко Петро - ТОВ Приклад".to_string()),
                Block::Table {
                    header: vec!["№".to_string(), "Тип".to_string()],
                    rows: vec![vec!["1".to_string(), "owns".to_string()]],
                },
            ],
        }
    }

    #[test]
    fn the_pdf_is_a_pdf_with_cyrillic_text() {
        let pdf = to_pdf(&report()).unwrap();
        assert!(pdf.starts_with(b"%PDF-"));
        assert!(pdf.len() > 5000);
    }

    #[test]
    fn a_protected_pdf_needs_the_passphrase() {
        let protected = pdf_protected(&report(), "пароль").unwrap();
        assert!(!protected.starts_with(b"%PDF-"));
        assert!(unwrap(&protected, "пароль").unwrap().starts_with(b"%PDF-"));
        assert!(unwrap(&protected, "інший").is_err());
    }

    #[test]
    fn many_rows_do_not_fail() {
        let mut big = report();
        if let Block::Table { rows, .. } = &mut big.blocks[2] {
            for i in 0..200 {
                rows.push(vec![
                    i.to_string(),
                    "довгий текст для переносу рядка".to_string(),
                ]);
            }
        }
        assert!(to_pdf(&big).is_ok());
    }
}
