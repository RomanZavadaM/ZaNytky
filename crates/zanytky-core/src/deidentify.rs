//! De-identification of a text before it goes to the AI (ADR-0007).
//!
//! Known values are replaced by their pseudonyms: names and aliases from the
//! investigator's dictionary, national tax numbers (10 digits), phone numbers
//! and e-mail addresses. Codes of organisations are public and stay. Names the
//! system does not know are reported as candidates so the investigator can
//! decide what they are (step 2a of the parse package).

use std::collections::BTreeMap;

use crate::normalize::{Kind, APOSTROPHES, DASHES};
use crate::pseudonym::{token, TokenError};

/// A name or alias the investigator knows.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DictEntry {
    pub text: String,
    /// Pseudonym of the person this spelling stands for. When absent the
    /// pseudonym is computed from the text itself.
    pub token: Option<String>,
}

impl DictEntry {
    pub fn name(text: &str) -> Self {
        Self {
            text: text.to_string(),
            token: None,
        }
    }

    pub fn alias(text: &str, token: &str) -> Self {
        Self {
            text: text.to_string(),
            token: Some(token.to_string()),
        }
    }
}

/// How many times a pseudonym was substituted.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Replacement {
    pub token: String,
    pub kind: Kind,
    pub count: u32,
}

/// Something that looks like a person's name but is not in the dictionary.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Candidate {
    pub text: String,
    pub count: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Deidentified {
    pub text: String,
    pub replacements: Vec<Replacement>,
    pub candidates: Vec<Candidate>,
}

struct Span {
    start: usize,
    end: usize,
    token: String,
    kind: Kind,
}

fn norm_char(c: char) -> char {
    if APOSTROPHES.contains(&c) {
        '\''
    } else if DASHES.contains(&c) {
        '-'
    } else {
        c.to_lowercase().next().unwrap_or(c)
    }
}

fn entry_chars(text: &str) -> Vec<char> {
    text.split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .chars()
        .map(|c| if c == ' ' { c } else { norm_char(c) })
        .collect()
}

fn match_at(chars: &[char], start: usize, entry: &[char]) -> Option<usize> {
    let mut i = start;
    let mut j = 0;
    while j < entry.len() {
        if i >= chars.len() {
            return None;
        }
        if entry[j] == ' ' {
            if !chars[i].is_whitespace() {
                return None;
            }
            while i < chars.len() && chars[i].is_whitespace() {
                i += 1;
            }
        } else {
            if norm_char(chars[i]) != entry[j] {
                return None;
            }
            i += 1;
        }
        j += 1;
    }
    Some(i)
}

fn boundary_before(chars: &[char], start: usize) -> bool {
    start == 0 || !chars[start - 1].is_alphanumeric()
}

fn boundary_after(chars: &[char], end: usize) -> bool {
    end >= chars.len() || !chars[end].is_alphanumeric()
}

fn dictionary_spans(
    chars: &[char],
    dictionary: &[DictEntry],
    key: &[u8],
) -> Result<Vec<Span>, TokenError> {
    let mut entries: Vec<(Vec<char>, String)> = Vec::new();
    for entry in dictionary {
        let normalized = entry_chars(&entry.text);
        if normalized.is_empty() {
            continue;
        }
        let tok = match &entry.token {
            Some(tok) => tok.clone(),
            None => token(Kind::Name, &entry.text, key)?,
        };
        entries.push((normalized, tok));
    }
    entries.sort_by_key(|(chars, _)| std::cmp::Reverse(chars.len()));
    let mut spans = Vec::new();
    for start in 0..chars.len() {
        if !boundary_before(chars, start) {
            continue;
        }
        for (entry, tok) in &entries {
            if let Some(end) = match_at(chars, start, entry) {
                if boundary_after(chars, end) {
                    spans.push(Span {
                        start,
                        end,
                        token: tok.clone(),
                        kind: Kind::Name,
                    });
                    break;
                }
            }
        }
    }
    Ok(spans)
}

/// Collect digits from `start` (an optional `+` first) through spaces, hyphens
/// and parentheses until exactly `limit` digits are read.
fn scan_digits(chars: &[char], start: usize, limit: usize) -> Option<(String, usize)> {
    let mut i = start;
    if chars.get(i) == Some(&'+') {
        i += 1;
    }
    let mut digits = String::new();
    let mut end = i;
    while i < chars.len() && digits.len() < limit {
        let c = chars[i];
        if c.is_ascii_digit() {
            digits.push(c);
            end = i + 1;
        } else if !matches!(c, ' ' | '-' | '(' | ')') {
            break;
        }
        i += 1;
    }
    if digits.len() == limit && !chars.get(end).is_some_and(char::is_ascii_digit) {
        Some((digits, end))
    } else {
        None
    }
}

fn number_spans(chars: &[char], key: &[u8]) -> Result<Vec<Span>, TokenError> {
    let mut spans = Vec::new();
    let mut i = 0;
    while i < chars.len() {
        let c = chars[i];
        let starts = c.is_ascii_digit() || c == '+';
        if !starts || (i > 0 && (chars[i - 1].is_alphanumeric() || chars[i - 1] == '+')) {
            i += 1;
            continue;
        }
        let international = scan_digits(chars, i, 12).filter(|(d, _)| d.starts_with("380"));
        let national = scan_digits(chars, i, 10).filter(|(d, _)| d.starts_with('0'));
        let found = if let Some((digits, end)) = international.or(national) {
            Some((Kind::Phone, digits, end))
        } else if let Some((digits, end)) = scan_digits(chars, i, 10) {
            let contiguous = chars[i..end].iter().all(char::is_ascii_digit);
            contiguous.then_some((Kind::Ipn, digits, end))
        } else {
            None
        };
        match found {
            Some((kind, digits, end)) if boundary_after(chars, end) => {
                spans.push(Span {
                    start: i,
                    end,
                    token: token(kind, &digits, key)?,
                    kind,
                });
                i = end;
            }
            _ => {
                while i < chars.len() && (chars[i].is_ascii_digit() || chars[i] == '+') {
                    i += 1;
                }
                i += 1;
            }
        }
    }
    Ok(spans)
}

fn email_spans(chars: &[char], key: &[u8]) -> Result<Vec<Span>, TokenError> {
    let local = |c: char| c.is_ascii_alphanumeric() || matches!(c, '.' | '_' | '%' | '+' | '-');
    let domain = |c: char| c.is_ascii_alphanumeric() || matches!(c, '.' | '-');
    let mut spans = Vec::new();
    for (at, _) in chars.iter().enumerate().filter(|(_, c)| **c == '@') {
        let mut start = at;
        while start > 0 && local(chars[start - 1]) {
            start -= 1;
        }
        let mut end = at + 1;
        while end < chars.len() && domain(chars[end]) {
            end += 1;
        }
        while end > at + 1 && chars[end - 1] == '.' {
            end -= 1;
        }
        let text: String = chars[start..end].iter().collect();
        if start < at && end > at + 1 && text[text.find('@').unwrap_or(0)..].contains('.') {
            spans.push(Span {
                start,
                end,
                token: token(Kind::Email, &text, key)?,
                kind: Kind::Email,
            });
        }
    }
    Ok(spans)
}

fn is_cap_word(word: &str) -> bool {
    let mut chars = word.chars();
    match chars.next() {
        Some(first) if first.is_uppercase() => {}
        _ => return false,
    }
    let rest: Vec<char> = chars.collect();
    !rest.is_empty()
        && rest.iter().all(|c| {
            c.is_lowercase()
                || APOSTROPHES.contains(c)
                || *c == '\''
                || DASHES.contains(c)
                || *c == '-'
        })
}

fn is_initials(word: &str) -> bool {
    let chars: Vec<char> = word.chars().collect();
    !chars.is_empty()
        && chars.len() % 2 == 0
        && chars
            .chunks(2)
            .all(|pair| pair[0].is_uppercase() && pair[1] == '.')
}

#[derive(Default)]
struct Run {
    words: Vec<String>,
    caps: u32,
    inits: u32,
}

impl Run {
    fn flush(&mut self, counts: &mut BTreeMap<String, u32>) {
        if self.caps >= 2 || (self.caps >= 1 && self.inits >= 1) {
            *counts.entry(self.words.join(" ")).or_insert(0) += 1;
        }
        *self = Run::default();
    }
}

fn find_candidates(text: &str) -> Vec<Candidate> {
    let mut counts: BTreeMap<String, u32> = BTreeMap::new();
    let mut run = Run::default();
    for raw in text.split_whitespace() {
        let mut word = raw.trim_start_matches(['(', '\u{ab}', '"']);
        let mut brk = false;
        let trimmed = word.trim_end_matches([',', ';', ':', '!', '?', ')', '\u{bb}', '"']);
        if trimmed.len() != word.len() {
            brk = true;
        }
        word = trimmed;
        if word.ends_with('.') && !is_initials(word) {
            word = &word[..word.len() - 1];
            brk = true;
        }
        if is_cap_word(word) {
            run.words.push(word.to_string());
            run.caps += 1;
        } else if is_initials(word) {
            run.words.push(word.to_string());
            run.inits += 1;
        } else {
            run.flush(&mut counts);
        }
        if brk {
            run.flush(&mut counts);
        }
    }
    run.flush(&mut counts);
    let mut out: Vec<Candidate> = counts
        .into_iter()
        .map(|(text, count)| Candidate { text, count })
        .collect();
    out.sort_by(|a, b| b.count.cmp(&a.count).then_with(|| a.text.cmp(&b.text)));
    out
}

/// Replace known identifiers in `text` by pseudonyms and list unknown name candidates.
pub fn deidentify(
    text: &str,
    key: &[u8],
    dictionary: &[DictEntry],
) -> Result<Deidentified, TokenError> {
    let chars: Vec<char> = text.chars().collect();
    let mut spans = dictionary_spans(&chars, dictionary, key)?;
    spans.extend(email_spans(&chars, key)?);
    spans.extend(number_spans(&chars, key)?);
    spans.sort_by_key(|s| (s.start, std::cmp::Reverse(s.end)));

    let mut out = String::new();
    let mut counts: BTreeMap<(String, &'static str), (Kind, u32)> = BTreeMap::new();
    let mut cursor = 0;
    for span in &spans {
        if span.start < cursor {
            continue;
        }
        out.extend(&chars[cursor..span.start]);
        out.push_str(&span.token);
        cursor = span.end;
        let entry = counts
            .entry((span.token.clone(), span.kind.as_str()))
            .or_insert((span.kind, 0));
        entry.1 += 1;
    }
    out.extend(&chars[cursor..]);

    let replacements = counts
        .into_iter()
        .map(|((token, _), (kind, count))| Replacement { token, kind, count })
        .collect();
    let candidates = find_candidates(&out);
    Ok(Deidentified {
        text: out,
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

    fn run(text: &str, dictionary: &[DictEntry]) -> Deidentified {
        deidentify(text, &key(), dictionary).unwrap()
    }

    #[test]
    fn dictionary_names_are_replaced_in_any_spelling() {
        let dict = [DictEntry::name("Мар\u{2019}яна Іваненко")];
        let result = run("Вчора мар'яна   іваненко підписала акт.", &dict);
        let tok = token(Kind::Name, "Мар'яна Іваненко", &key()).unwrap();
        assert_eq!(result.text, format!("Вчора {tok} підписала акт."));
        assert_eq!(result.replacements.len(), 1);
        assert_eq!(result.replacements[0].count, 1);
    }

    #[test]
    fn an_alias_uses_the_given_pseudonym() {
        let dict = [DictEntry::alias("Іванов І.", "N-PERSON")];
        let result = run("Звернувся Іванов І. з заявою", &dict);
        assert_eq!(result.text, "Звернувся N-PERSON з заявою");
    }

    #[test]
    fn longer_entries_win_and_boundaries_hold() {
        let dict = [
            DictEntry::name("Петро"),
            DictEntry::name("Петро Іваненко"),
            DictEntry::name("Іван"),
        ];
        let result = run("Петро Іваненко та Іванович", &dict);
        let long = token(Kind::Name, "Петро Іваненко", &key()).unwrap();
        assert_eq!(result.text, format!("{long} та Іванович"));
    }

    #[test]
    fn tax_numbers_are_replaced_but_organisation_codes_stay() {
        let result = run("ІПН 1234567890, ЄДРПОУ 12345678.", &[]);
        let tok = token(Kind::Ipn, "1234567890", &key()).unwrap();
        assert_eq!(result.text, format!("ІПН {tok}, ЄДРПОУ 12345678."));
    }

    #[test]
    fn phone_formats_give_one_pseudonym() {
        let result = run("Тел. +38 (050) 123-45-67 або 0501234567.", &[]);
        let tok = token(Kind::Phone, "380501234567", &key()).unwrap();
        assert_eq!(result.text, format!("Тел. {tok} або {tok}."));
        assert_eq!(result.replacements.len(), 1);
        assert_eq!(result.replacements[0].count, 2);
    }

    #[test]
    fn emails_are_replaced() {
        let result = run("Пишіть на Ivan.P@Example.org, дякую.", &[]);
        let tok = token(Kind::Email, "ivan.p@example.org", &key()).unwrap();
        assert_eq!(result.text, format!("Пишіть на {tok}, дякую."));
    }

    #[test]
    fn unknown_names_are_reported_as_candidates() {
        let result = run(
            "заступник Олексій Коваль підписав; звернувся Мельник І. П.",
            &[],
        );
        let texts: Vec<&str> = result.candidates.iter().map(|c| c.text.as_str()).collect();
        assert!(texts.contains(&"Олексій Коваль"));
        assert!(texts.contains(&"Мельник І. П."));
    }

    #[test]
    fn known_names_are_not_candidates() {
        let dict = [DictEntry::name("Олексій Коваль")];
        let result = run("Заступник Олексій Коваль підписав.", &dict);
        assert!(result.candidates.is_empty());
    }

    #[test]
    fn the_output_has_no_original_values() {
        let dict = [DictEntry::name("Іваненко Петро")];
        let result = run(
            "Іваненко Петро, ІПН 1234567890, тел. 0501234567, ivan@example.org.",
            &dict,
        );
        for secret in ["Іваненко", "1234567890", "0501234567", "ivan@example.org"] {
            assert!(!result.text.contains(secret), "{secret} leaked");
        }
    }

    #[test]
    fn text_without_identifiers_is_unchanged() {
        let text = "Тендер на 2026 рік, сума 1500000 грн.";
        let result = run(text, &[]);
        assert_eq!(result.text, text);
        assert!(result.replacements.is_empty());
    }
}
