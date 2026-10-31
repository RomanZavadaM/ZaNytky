//! A small fictional case, so the application can be tried (the "Database" and "Report" tabs,
//! statuses, source chains) without an AI step and without any real data.
//!
//! It is created only in an empty or new folder: a system key, a demo investigator key,
//! pseudonyms for three invented people and one imported result with sources of different
//! reliability, ownership, a procurement and a rumour. Nothing in it is real.

use std::fs;
use std::path::{Path, PathBuf};

use serde_json::json;

use crate::model::Model;
use crate::normalize::Kind;
use crate::pseudonym::new_key;
use crate::registry::Registry;
use crate::seal::{generate_identity, parse_recipient};
use crate::{import_result, parse_result};

/// Where the demo was made.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DemoInfo {
    pub home: PathBuf,
    /// The demo investigator's private key file.
    pub identity: PathBuf,
    pub statements: usize,
}

const PACKAGE: &str = "demo-1";

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}

/// Create the fictional case in `home` (must be empty or missing).
pub fn create_demo(home: &Path) -> Result<DemoInfo, String> {
    if home.exists() && fs::read_dir(home).map_err(|e| e.to_string())?.next().is_some() {
        return Err("папка не порожня: демонстрація створюється лише в порожній або новій папці".to_string());
    }
    fs::create_dir_all(home).map_err(|e| e.to_string())?;
    let key = new_key().map_err(|e| e.to_string())?;
    fs::write(home.join("system.key"), hex(&key)).map_err(|e| e.to_string())?;
    let (secret, public) = generate_identity();
    let identity = home.join("demo-investigator-key.txt");
    fs::write(
        &identity,
        format!("# демонстраційний ключ розслідувача (вигадані дані)\n{secret}\n"),
    )
    .map_err(|e| e.to_string())?;
    fs::write(
        home.join("recipients.txt"),
        format!("# демонстраційний розслідувач\n{public}\n"),
    )
    .map_err(|e| e.to_string())?;

    let recipient = parse_recipient(&public).map_err(|e| e.to_string())?;
    let registry =
        Registry::open(&home.join("registry.db"), &key, vec![recipient]).map_err(|e| e.to_string())?;
    let model = Model::open(&home.join("model.db"), "demo").map_err(|e| e.to_string())?;
    let person = |name: &str| registry.register(Kind::Name, name).map_err(|e| e.to_string());
    let t1 = person("Петро Іваненко")?;
    let t2 = person("Марія Шевченко")?;
    let t3 = person("Олексій Коваль")?;

    let result = json!({
        "package": PACKAGE,
        "sources": [
            {"ref": "s1", "kind": "registry", "title": "Вигаданий реєстр юридичних осіб", "url": null,
             "published_at": "2026-01-10", "reliability": "A", "received_from": null},
            {"ref": "s2", "kind": "media", "title": "Вигадане місцеве видання", "url": "https://example.org/demo-news",
             "published_at": "2026-09-02", "reliability": "C", "received_from": null},
            {"ref": "s3", "kind": "rumor", "title": "Чутка в коментарях", "url": null,
             "published_at": null, "reliability": "F", "received_from": "s2"}
        ],
        "entities": [
            {"ref": "p1", "kind": "person", "label": null, "token": t1, "public_code": null},
            {"ref": "p2", "kind": "person", "label": null, "token": t2, "public_code": null},
            {"ref": "p3", "kind": "person", "label": null, "token": t3, "public_code": null},
            {"ref": "o1", "kind": "org", "label": "ТОВ «Приклад»", "token": null, "public_code": "12345678"},
            {"ref": "o2", "kind": "org", "label": "ТОВ «Будмайстер»", "token": null, "public_code": "87654321"},
            {"ref": "o3", "kind": "org", "label": "Міська рада", "token": null, "public_code": null},
            {"ref": "pr1", "kind": "procurement", "label": "Закупівля «Ремонт дороги»", "token": null, "public_code": null}
        ],
        "statements": [
            {"kind": "founder", "value": null, "valid_from": null, "valid_to": null, "as_of": "2026-01-10",
             "parties": [{"entity": "p1", "role": "person"}, {"entity": "o1", "role": "company"}],
             "sources": ["s1"], "credibility": 2, "quote": format!("Засновник: {t1}"), "note": null},
            {"kind": "owns", "value": "40%", "valid_from": null, "valid_to": null, "as_of": "2026-01-10",
             "parties": [{"entity": "p1", "role": "owner"}, {"entity": "o1", "role": "owned"}],
             "sources": ["s1"], "credibility": 2, "quote": format!("{t1} володіє 40% статутного капіталу"), "note": null},
            {"kind": "director", "value": null, "valid_from": null, "valid_to": null, "as_of": "2026-01-10",
             "parties": [{"entity": "p2", "role": "person"}, {"entity": "o1", "role": "company"}],
             "sources": ["s1"], "credibility": 2, "quote": format!("Директор: {t2}"), "note": null},
            {"kind": "owns", "value": "60%", "valid_from": null, "valid_to": null, "as_of": "2026-09-02",
             "parties": [{"entity": "p3", "role": "owner"}, {"entity": "o1", "role": "owned"}],
             "sources": ["s2"], "credibility": 3, "quote": format!("решта 60% належить {t3}"), "note": "лише за повідомленням видання"},
            {"kind": "beneficiary", "value": null, "valid_from": null, "valid_to": null, "as_of": "2026-09-02",
             "parties": [{"entity": "p3", "role": "person"}, {"entity": "o2", "role": "company"}],
             "sources": ["s2"], "credibility": 4, "quote": format!("кінцевий бенефіціар - {t3}"), "note": null},
            {"kind": "supplies", "value": null, "valid_from": null, "valid_to": null, "as_of": "2026-09-02",
             "parties": [{"entity": "o2", "role": "supplier"}, {"entity": "o1", "role": "customer"}],
             "sources": ["s2"], "credibility": 3, "quote": "ТОВ «Будмайстер» постачає матеріали ТОВ «Приклад»", "note": null},
            {"kind": "won", "value": "1 500 000 грн", "valid_from": null, "valid_to": null, "as_of": "2026-09-02",
             "parties": [{"entity": "o1", "role": "company"}, {"entity": "pr1", "role": "procurement"}],
             "sources": ["s2"], "credibility": 3, "quote": "компанія виграла закупівлю на суму 1 500 000 грн", "note": null},
            {"kind": "customer", "value": null, "valid_from": null, "valid_to": null, "as_of": "2026-09-02",
             "parties": [{"entity": "o3", "role": "customer"}, {"entity": "pr1", "role": "procurement"}],
             "sources": ["s2"], "credibility": 3, "quote": "замовник - міська рада", "note": null},
            {"kind": "reported_link", "value": "нібито родичі", "valid_from": null, "valid_to": null, "as_of": null,
             "parties": [{"entity": "p2", "role": "person"}, {"entity": "p3", "role": "person"}],
             "sources": ["s3"], "credibility": 6, "quote": format!("{t2} і {t3} нібито родичі"), "note": "чутка, не підтверджено"}
        ]
    });
    let file = parse_result(&result.to_string(), PACKAGE).map_err(|e| e.to_string())?;
    let summary = import_result(&model, &file).map_err(|e| e.to_string())?;
    Ok(DemoInfo {
        home: home.to_path_buf(),
        identity,
        statements: summary.statements,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_demo_is_made_in_a_new_folder_with_statements() {
        let dir = tempfile::tempdir().unwrap();
        let home = dir.path().join("demo");
        let info = create_demo(&home).unwrap();
        assert_eq!(info.statements, 9);
        assert!(home.join("system.key").exists());
        assert!(info.identity.exists());
        let model = Model::open(&home.join("model.db"), "test").unwrap();
        assert_eq!(model.statements().unwrap().len(), 9);
    }

    #[test]
    fn a_folder_with_files_is_refused_and_left_untouched() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(dir.path().join("mine.txt"), "x").unwrap();
        assert!(create_demo(dir.path()).is_err());
        assert!(!dir.path().join("system.key").exists());
    }
}
