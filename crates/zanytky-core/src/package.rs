//! The parse package for the AI (ADR-0007): de-identified text, prompt, metadata.
//!
//! The original text and the table of replacements never go into the package.

use serde_json::json;

use crate::deidentify::{deidentify, Candidate, DictEntry, Replacement};
use crate::model::{Reliability, SourceKind};
use crate::pseudonym::TokenError;

/// What the investigator knows about the document.
#[derive(Debug, Clone)]
pub struct SourceMeta {
    pub kind: SourceKind,
    pub title: String,
    pub url: Option<String>,
    pub published_at: Option<String>,
    /// The investigator's own estimate; the AI only proposes.
    pub reliability: Reliability,
    pub received_from: Option<String>,
}

/// Files for the AI plus what the investigator needs to review.
#[derive(Debug, Clone)]
pub struct Package {
    pub id: String,
    /// `deidentified.md`
    pub deidentified: String,
    /// `prompt.md`
    pub prompt: String,
    /// `package.json`
    pub meta_json: String,
    pub replacements: Vec<Replacement>,
    /// Names the system did not recognise (step 2a: the investigator decides).
    pub candidates: Vec<Candidate>,
}

const PROMPT: &str = r#"# Завдання: розбір документа

Пакет: {package}

Ти отримуєш знеособлений документ (`deidentified.md`) і метадані (`package.json`). Витягни з нього **джерела, суб'єктів і твердження** та поверни результат лише у вигляді JSON (`result.json`) за схемою нижче.

## Правила
1. Лише те, що сказано в тексті. Нічого не вигадуй і не домислюй.
2. Факти, а не оцінки: не ставити ярликів і не робити висновків про мотиви чи провину. Кожне твердження супроводжуй дослівною цитатою з тексту (`quote`).
3. Псевдоніми виду `N-...`, `I-...`, `T-...`, `M-...` залишай як є. Не намагайся здогадатися, хто це. Особа в `entities` має лише `token`, без `label`.
4. Організації, закупівлі, ринки тощо мають `label`; публічний код (наприклад, ЄДРПОУ) пиши в `public_code`.
5. Нерозбірливе й неповне позначай у `note` словами «читається погано»; числа не вигадуй.
6. Розділ «Контекст від розслідувача» (якщо є) - пояснення, а не доказ. Твердження, що спираються лише на нього, мають джерело виду `own`.
7. Чутки й свідчення позначай видом джерела `rumor` чи `testimony`. Не підвищуй їхню надійність. Якщо видно, від кого джерело отримало інформацію, вкажи це в `received_from`.
8. `reliability` (A-F) і `credibility` (1-6) - лише пропозиція, з коротким поясненням у `note`. Статусу «підтверджено» не існує: це вирішує людина.
9. Відповідай лише JSON, без пояснень поза ним.

## Схема `result.json`
```json
{
  "package": "{package}",
  "sources": [
    {"ref": "s1", "kind": "media", "title": "...", "url": null,
     "published_at": "2026-10-01", "reliability": "C", "received_from": null}
  ],
  "entities": [
    {"ref": "e1", "kind": "org", "label": "...", "token": null, "public_code": "12345678"},
    {"ref": "e2", "kind": "person", "label": null, "token": "N-XXXXXXXXXXXXXXXXXXXXXXXXXX", "public_code": null}
  ],
  "statements": [
    {"kind": "owns", "value": "40%", "valid_from": null, "valid_to": null, "as_of": null,
     "parties": [{"entity": "e2", "role": "owner"}, {"entity": "e1", "role": "owned"}],
     "sources": ["s1"], "credibility": 3, "quote": "...", "note": null}
  ]
}
```
Дозволені значення:
- `sources[].kind`: registry, authority, media, testimony, rumor, own; `reliability`: A, B, C, D, E, F;
- `entities[].kind`: person, org, group, asset, procurement, market, event, document;
- `statements[].kind`: owns, beneficiary, director, founder, won, customer, supplies, uses, coordinated, member_of_group, operates_in_market, reported_link;
- ролі (`role`): owner, owned, person, company, customer, supplier, member, group та інші короткі слова;
- `credibility`: ціле 1-6 (1 - підтверджено незалежно, 6 - перевірити неможливо).
"#;

/// Build the package: de-identify the text (and the investigator's context
/// note), fill the prompt, describe the source.
pub fn build_package(
    id: &str,
    meta: &SourceMeta,
    text: &str,
    context_note: Option<&str>,
    key: &[u8],
    dictionary: &[DictEntry],
) -> Result<Package, TokenError> {
    let body = deidentify(text, key, dictionary)?;
    let mut deidentified = body.text;
    let mut replacements = body.replacements;
    let mut candidates = body.candidates;
    if let Some(note) = context_note.filter(|n| !n.trim().is_empty()) {
        let context = deidentify(note, key, dictionary)?;
        deidentified.push_str("\n\n## Контекст від розслідувача\n\n");
        deidentified.push_str(&context.text);
        deidentified.push('\n');
        replacements.extend(context.replacements);
        candidates.extend(context.candidates);
    }
    let title = deidentify(&meta.title, key, dictionary)?.text;
    let received_from = match &meta.received_from {
        Some(text) => Some(deidentify(text, key, dictionary)?.text),
        None => None,
    };
    let meta_json = serde_json::to_string_pretty(&json!({
        "package": id,
        "source": {
            "kind": meta.kind.as_str(),
            "title": title,
            "url": meta.url,
            "published_at": meta.published_at,
            "reliability_estimate": meta.reliability.as_str(),
            "received_from": received_from,
        }
    }))
    .unwrap_or_default();
    Ok(Package {
        id: id.to_string(),
        deidentified,
        prompt: PROMPT.replace("{package}", id),
        meta_json,
        replacements,
        candidates,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn key() -> Vec<u8> {
        (0u8..32).collect()
    }

    fn meta() -> SourceMeta {
        SourceMeta {
            kind: SourceKind::Media,
            title: "Інтерв'ю Іваненка Петра".to_string(),
            url: Some("https://example.org/a".to_string()),
            published_at: Some("2026-10-01".to_string()),
            reliability: Reliability::C,
            received_from: None,
        }
    }

    #[test]
    fn the_package_has_no_original_values() {
        let dict = [DictEntry::name("Іваненко Петро")];
        let pkg = build_package(
            "pkg-1",
            &meta(),
            "Іваненко Петро, ІПН 1234567890, тел. 0501234567.",
            None,
            &key(),
            &dict,
        )
        .unwrap();
        for secret in ["Іваненко", "1234567890", "0501234567"] {
            assert!(!pkg.deidentified.contains(secret), "{secret} in text");
            assert!(!pkg.meta_json.contains(secret), "{secret} in meta");
            assert!(!pkg.prompt.contains(secret), "{secret} in prompt");
        }
    }

    #[test]
    fn the_title_is_de_identified_too() {
        let dict = [DictEntry::name("Іваненка Петра")];
        let pkg = build_package("pkg-1", &meta(), "Текст.", None, &key(), &dict).unwrap();
        assert!(!pkg.meta_json.contains("Іваненка"));
        assert!(pkg.meta_json.contains("pkg-1"));
        assert!(pkg.meta_json.contains("https://example.org/a"));
    }

    #[test]
    fn the_prompt_names_the_package_and_the_rules() {
        let pkg = build_package("pkg-42", &meta(), "Текст.", None, &key(), &[]).unwrap();
        assert!(pkg.prompt.contains("pkg-42"));
        assert!(!pkg.prompt.contains("{package}"));
        assert!(pkg.prompt.contains("result.json"));
        assert!(pkg.prompt.contains("quote"));
    }

    #[test]
    fn the_context_note_is_added_and_de_identified() {
        let dict = [DictEntry::alias("Іванов І.", "N-PERSON")];
        let pkg = build_package(
            "pkg-1",
            &meta(),
            "Текст.",
            Some("Іванов І. - керівник ТОВ Приклад."),
            &key(),
            &dict,
        )
        .unwrap();
        assert!(pkg.deidentified.contains("## Контекст від розслідувача"));
        assert!(pkg.deidentified.contains("N-PERSON - керівник"));
        assert!(!pkg.deidentified.contains("Іванов"));
    }

    #[test]
    fn candidates_from_text_and_note_are_collected() {
        let pkg = build_package(
            "pkg-1",
            &meta(),
            "заступник Олексій Коваль підписав.",
            Some("також згадується Марія Шевченко."),
            &key(),
            &[],
        )
        .unwrap();
        let texts: Vec<&str> = pkg.candidates.iter().map(|c| c.text.as_str()).collect();
        assert!(texts.contains(&"Олексій Коваль"));
        assert!(texts.contains(&"Марія Шевченко"));
    }
}
