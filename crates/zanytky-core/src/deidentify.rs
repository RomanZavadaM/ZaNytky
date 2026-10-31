//! De-identification of a text before it goes to the AI (ADR-0007).
//!
//! Known values are replaced by their pseudonyms: names and aliases from the
//! investigator's dictionary, national tax numbers (10 digits), phone numbers,
//! e-mail addresses, bank account numbers (IBAN), payment card numbers, vehicle plates and street
//! addresses with a house number. Codes of organisations are public and stay. Names the
//! system does not know are reported as candidates so the investigator can
//! decide what they are (step 2a of the parse package).

use std::collections::{BTreeMap, HashMap, HashSet};

use crate::normalize::{
    address_marker, month_number, normalize, plate_letter, Kind, APOSTROPHES, DASHES,
};
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

/// Other word orders of a full name written as "Surname Name [Patronymic]": the name first,
/// without the patronymic, name and patronymic. Forms with initials are not generated:
/// "Іваненко П." can be Петро or Павло, so such mentions stay for the investigator to decide.
fn name_variants(text: &str) -> Vec<String> {
    let parts: Vec<&str> = text.split_whitespace().collect();
    let plain = |part: &&str| {
        part.chars().count() >= 2
            && part.chars().all(|c| {
                c.is_alphabetic()
                    || APOSTROPHES.contains(&c)
                    || DASHES.contains(&c)
                    || c == '\''
                    || c == '-'
            })
    };
    if !(2..=3).contains(&parts.len()) || !parts.iter().all(plain) {
        return Vec::new();
    }
    match parts.as_slice() {
        [surname, name] => vec![format!("{name} {surname}")],
        [surname, name, patronymic] => vec![
            format!("{name} {patronymic} {surname}"),
            format!("{surname} {name}"),
            format!("{name} {surname}"),
            format!("{name} {patronymic}"),
        ],
        _ => Vec::new(),
    }
}

fn dictionary_spans(
    chars: &[char],
    dictionary: &[DictEntry],
    key: &[u8],
) -> Result<Vec<Span>, TokenError> {
    let mut entries: Vec<(Vec<char>, String)> = Vec::new();
    let mut originals: Vec<(String, String)> = Vec::new();
    for entry in dictionary {
        let normalized = entry_chars(&entry.text);
        if normalized.is_empty() {
            continue;
        }
        let tok = match &entry.token {
            Some(tok) => tok.clone(),
            None => token(Kind::Name, &entry.text, key)?,
        };
        entries.push((normalized, tok.clone()));
        originals.push((entry.text.clone(), tok));
    }
    // Other word orders of the same name. An explicit entry always wins, and a form that
    // would belong to two different people is dropped (left for the investigator).
    let explicit: HashSet<Vec<char>> = entries.iter().map(|(chars, _)| chars.clone()).collect();
    let mut generated: HashMap<Vec<char>, Option<String>> = HashMap::new();
    for (text, tok) in &originals {
        for variant in name_variants(text) {
            let variant_chars = entry_chars(&variant);
            if explicit.contains(&variant_chars) {
                continue;
            }
            match generated.get(&variant_chars) {
                Some(Some(existing)) if existing != tok => {
                    generated.insert(variant_chars, None);
                }
                Some(_) => {}
                None => {
                    generated.insert(variant_chars, Some(tok.clone()));
                }
            }
        }
    }
    for (variant_chars, tok) in generated {
        if let Some(tok) = tok {
            entries.push((variant_chars, tok));
        }
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

/// Ukrainian bank account number (IBAN): `UA` and 27 digits, spaces allowed, valid check.
fn iban_spans(chars: &[char], key: &[u8]) -> Result<Vec<Span>, TokenError> {
    let mut spans = Vec::new();
    for start in 0..chars.len().saturating_sub(1) {
        let first = chars[start].to_ascii_uppercase();
        let second = chars[start + 1].to_ascii_uppercase();
        if first != 'U' || second != 'A' || !boundary_before(chars, start) {
            continue;
        }
        let mut digits = 0;
        let mut end = start + 2;
        let mut i = start + 2;
        while i < chars.len() && digits < 27 {
            let c = chars[i];
            if c.is_ascii_digit() {
                digits += 1;
                end = i + 1;
            } else if !matches!(c, ' ' | '-') {
                break;
            }
            i += 1;
        }
        if digits != 27 || !boundary_after(chars, end) {
            continue;
        }
        let text: String = chars[start..end].iter().collect();
        if normalize(Kind::Iban, &text).is_ok() {
            spans.push(Span {
                start,
                end,
                token: token(Kind::Iban, &text, key)?,
                kind: Kind::Iban,
            });
        }
    }
    Ok(spans)
}

/// Words before a date that mark it as a date of birth.
const BIRTH_BEFORE: &[&str] = &[
    "народження",
    "народився",
    "народилася",
    "народилось",
    "народжений",
    "народжена",
    "нар",
    "н",
    "дн",
];

/// Bounds of a word without the punctuation around it.
fn trimmed_bounds(chars: &[char], (start, end): (usize, usize)) -> (usize, usize) {
    let trim = |c: char| matches!(c, ',' | ';' | '.' | ':' | '(' | ')' | '"' | '\u{ab}' | '\u{bb}');
    let (mut first, mut last) = (start, end);
    while first < last && trim(chars[first]) {
        first += 1;
    }
    while last > first && trim(chars[last - 1]) {
        last -= 1;
    }
    (first, last)
}

fn all_digits(text: &str, min: usize, max: usize) -> bool {
    (min..=max).contains(&text.chars().count()) && text.chars().all(|c| c.is_ascii_digit())
}

/// A full date in the text that is marked as a date of birth ("дата народження 12.03.1985",
/// "народився 12 березня 1985", "(12.03.1985 р.н.)"). Other dates (contracts, events, publication)
/// are facts the analysis needs and stay; a year alone stays too.
fn birth_date_spans(chars: &[char], key: &[u8]) -> Result<Vec<Span>, TokenError> {
    let bounds = word_bounds(chars);
    let texts: Vec<String> = bounds
        .iter()
        .map(|(start, end)| chars[*start..*end].iter().collect())
        .collect();
    let lower = |index: usize| {
        if index < bounds.len() {
            strip_word(&texts[index]).to_lowercase()
        } else {
            String::new()
        }
    };
    let mut spans = Vec::new();
    for w in 0..bounds.len() {
        let word = strip_word(&texts[w]);
        let (start, end, last_word) = if all_digits(word, 1, 2)
            && w + 2 < bounds.len()
            && month_number(&lower(w + 1)).is_some()
            && all_digits(strip_word(&texts[w + 2]), 4, 4)
        {
            let start = trimmed_bounds(chars, bounds[w]).0;
            let end = trimmed_bounds(chars, bounds[w + 2]).1;
            (start, end, w + 2)
        } else if (8..=10).contains(&word.chars().count())
            && word.chars().all(|c| c.is_ascii_digit() || matches!(c, '.' | '/' | '-'))
        {
            let (start, end) = trimmed_bounds(chars, bounds[w]);
            (start, end, w)
        } else {
            continue;
        };
        let text: String = chars[start..end].iter().collect();
        if normalize(Kind::Birth, &text).is_err() {
            continue;
        }
        let before = (w.saturating_sub(4)..w).any(|i| BIRTH_BEFORE.contains(&lower(i).as_str()));
        let next = last_word + 1;
        let after = matches!(lower(next).as_str(), "р.н" | "р/н" | "рн")
            || (lower(next) == "р" && lower(next + 1) == "н")
            || (lower(next) == "року" && lower(next + 1) == "народження");
        if before || after {
            spans.push(Span {
                start,
                end,
                token: token(Kind::Birth, &text, key)?,
                kind: Kind::Birth,
            });
        }
    }
    Ok(spans)
}

/// Payment card number: 13-19 digits, grouped by four (or not grouped), valid Luhn check.
fn card_spans(chars: &[char], key: &[u8]) -> Result<Vec<Span>, TokenError> {
    let mut spans = Vec::new();
    for start in 0..chars.len() {
        if !chars[start].is_ascii_digit() || !boundary_before(chars, start) {
            continue;
        }
        let mut digits = 0;
        let mut end = start;
        let mut aligned = true;
        let mut i = start;
        while i < chars.len() && digits < 19 {
            let c = chars[i];
            if c.is_ascii_digit() {
                digits += 1;
                end = i + 1;
            } else if matches!(c, ' ' | '-') {
                if digits % 4 != 0 {
                    aligned = false;
                    break;
                }
            } else {
                break;
            }
            i += 1;
        }
        if !aligned || !(13..=19).contains(&digits) || !boundary_after(chars, end) {
            continue;
        }
        let text: String = chars[start..end].iter().collect();
        if normalize(Kind::Card, &text).is_ok() {
            spans.push(Span {
                start,
                end,
                token: token(Kind::Card, &text, key)?,
                kind: Kind::Card,
            });
        }
    }
    Ok(spans)
}

/// End of a vehicle plate that starts at `start`: two letters, four digits, two letters
/// (Cyrillic or Latin look-alikes, optional spaces between the groups).
fn plate_end(chars: &[char], start: usize) -> Option<usize> {
    fn letter(chars: &[char], i: usize) -> Option<char> {
        chars.get(i).copied().and_then(plate_letter)
    }
    let mut i = start;
    letter(chars, i)?;
    letter(chars, i + 1)?;
    i += 2;
    if chars.get(i).is_some_and(|c| c.is_whitespace()) {
        i += 1;
    }
    for _ in 0..4 {
        if !chars.get(i).is_some_and(char::is_ascii_digit) {
            return None;
        }
        i += 1;
    }
    if chars.get(i).is_some_and(|c| c.is_whitespace()) {
        i += 1;
    }
    letter(chars, i)?;
    letter(chars, i + 1)?;
    Some(i + 2)
}

fn plate_spans(chars: &[char], key: &[u8]) -> Result<Vec<Span>, TokenError> {
    let mut spans = Vec::new();
    for start in 0..chars.len() {
        if !boundary_before(chars, start) {
            continue;
        }
        if let Some(end) = plate_end(chars, start) {
            if boundary_after(chars, end) {
                let text: String = chars[start..end].iter().collect();
                spans.push(Span {
                    start,
                    end,
                    token: token(Kind::Plate, &text, key)?,
                    kind: Kind::Plate,
                });
            }
        }
    }
    Ok(spans)
}

fn strip_word(word: &str) -> &str {
    word.trim_matches(|c: char| {
        matches!(
            c,
            ',' | ';' | '.' | ':' | '(' | ')' | '"' | '\u{ab}' | '\u{bb}'
        )
    })
}

/// A house or flat number: 15, 15А, 15/2, 15-Б.
fn house_number(word: &str) -> bool {
    let word = strip_word(word);
    let digits = word.chars().take_while(char::is_ascii_digit).count();
    if digits == 0 || digits > 4 {
        return false;
    }
    let rest: Vec<char> = word.chars().skip(digits).collect();
    match rest.as_slice() {
        [] => true,
        [c] => c.is_alphabetic(),
        ['/' | '-', tail @ ..] => {
            !tail.is_empty()
                && (tail.iter().all(char::is_ascii_digit)
                    || (tail.len() == 1 && tail[0].is_alphabetic()))
        }
        _ => false,
    }
}

/// Positions of whitespace separated words.
fn word_bounds(chars: &[char]) -> Vec<(usize, usize)> {
    let mut words = Vec::new();
    let mut i = 0;
    while i < chars.len() {
        if chars[i].is_whitespace() {
            i += 1;
            continue;
        }
        let start = i;
        while i < chars.len() && !chars[i].is_whitespace() {
            i += 1;
        }
        words.push((start, i));
    }
    words
}

/// A street address with a house number: marker (вул., проспект, ...), one to three
/// capitalised words, the house number, optionally a flat or office and a city before.
/// A street without a house number is not an address of a person and stays.
fn address_spans(chars: &[char], key: &[u8]) -> Result<Vec<Span>, TokenError> {
    let bounds = word_bounds(chars);
    let texts: Vec<String> = bounds
        .iter()
        .map(|(start, end)| chars[*start..*end].iter().collect())
        .collect();
    let lower = |index: usize| strip_word(&texts[index]).to_lowercase();
    let mut spans = Vec::new();
    let mut w = 0;
    while w < bounds.len() {
        if address_marker(&lower(w)).is_none() {
            w += 1;
            continue;
        }
        let mut j = w + 1;
        let mut names = 0;
        while j < bounds.len() && names < 3 && is_cap_word(strip_word(&texts[j])) {
            names += 1;
            j += 1;
            if texts[j - 1].ends_with(',') {
                break;
            }
        }
        if names == 0 {
            w += 1;
            continue;
        }
        if j < bounds.len() && matches!(lower(j).as_str(), "буд" | "будинок" | "\u{2116}")
        {
            j += 1;
        }
        if j >= bounds.len() || !house_number(&texts[j]) {
            w += 1;
            continue;
        }
        let mut last = j;
        j += 1;
        if j + 1 < bounds.len()
            && matches!(lower(j).as_str(), "кв" | "квартира" | "оф" | "офіс")
            && house_number(&texts[j + 1])
        {
            last = j + 1;
        }
        let mut first = w;
        if w >= 2
            && matches!(lower(w - 2).as_str(), "м" | "місто")
            && is_cap_word(strip_word(&texts[w - 1]))
        {
            first = w - 2;
        }
        let start = bounds[first].0;
        let mut end = bounds[last].1;
        while end > start && matches!(chars[end - 1], ',' | ';' | '.' | ':' | ')') {
            end -= 1;
        }
        let text: String = chars[start..end].iter().collect();
        spans.push(Span {
            start,
            end,
            token: token(Kind::Address, &text, key)?,
            kind: Kind::Address,
        });
        w = last + 1;
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

/// Abbreviations and short function words that are written in capitals but are not names.
#[rustfmt::skip]
const ABBREVIATIONS: &[&str] = &[
    "ТОВ", "ПАТ", "ПРАТ", "ТДВ", "ДП", "КП", "КНП", "ФОП", "ПП", "ОСББ", "ДПС", "ДМС", "СБУ", "НАБУ",
    "САП", "ВАКС", "ЗСУ", "МВС", "ДСНС", "ОВА", "ОДА", "ЄДРПОУ", "ЄДР", "ІПН", "ПДВ", "ДСТУ", "ТЕЦ",
    "НБУ", "НКРЕ", "ПРО", "ЩОДО", "ДЛЯ", "ПІД", "НАД", "ПРИ", "АБО", "ПЕРЕД", "ПІСЛЯ", "МІЖ", "ВІД",
];

/// Endings that are typical of Ukrainian surnames and rarely belong to other words,
/// with the common case forms (nominative, genitive, dative, instrumental).
#[rustfmt::skip]
const SURNAME_ENDINGS: &[&str] = &[
    "енко", "енка", "енку", "енком", "енкові", "чук", "чука", "чуку", "чуком", "чукові", "юк", "юка",
    "юку", "юком", "юкові",
];

fn is_cyrillic_upper(c: char) -> bool {
    matches!(
        c,
        '\u{410}'..='\u{42f}' | '\u{401}' | '\u{404}' | '\u{406}' | '\u{407}' | '\u{490}'
    )
}

/// A word written entirely in Cyrillic capitals (registers and court decisions
/// often print surnames this way), at least three letters, not an abbreviation.
fn is_upper_word(word: &str) -> bool {
    let separator =
        |c: char| APOSTROPHES.contains(&c) || DASHES.contains(&c) || c == '\'' || c == '-';
    let letters = word.chars().filter(|c| !separator(*c)).count();
    letters >= 3
        && word.chars().all(|c| is_cyrillic_upper(c) || separator(c))
        && !ABBREVIATIONS.contains(&word)
}

/// A single word that looks like a surname by its ending (in any listed case form).
fn is_surname_form(word: &str) -> bool {
    let lower = word.to_lowercase();
    lower.chars().count() >= 5 && SURNAME_ENDINGS.iter().any(|ending| lower.ends_with(ending))
}

fn is_initials(word: &str) -> bool {
    let chars: Vec<char> = word.chars().collect();
    !chars.is_empty()
        && chars.len().is_multiple_of(2)
        && chars
            .chunks(2)
            .all(|pair| pair[0].is_uppercase() && pair[1] == '.')
}

/// Titles and roles that often start a sentence or a phrase before a name.
const TITLE_WORDS: [&str; 20] = [
    "заступник",
    "директор",
    "голова",
    "начальник",
    "керівник",
    "міністр",
    "депутат",
    "мер",
    "прокурор",
    "суддя",
    "пан",
    "пані",
    "громадянин",
    "громадянка",
    "власник",
    "засновник",
    "бенефіціар",
    "підприємець",
    "радник",
    "секретар",
];

#[derive(Default)]
struct Run {
    words: Vec<String>,
    caps: u32,
    inits: u32,
    /// The run contains a word written in capitals.
    upper: bool,
}

impl Run {
    fn flush(&mut self, counts: &mut BTreeMap<String, u32>) {
        if self.caps >= 2 || (self.caps >= 1 && self.inits >= 1) {
            // a long run in capitals is a heading, not a name
            if !(self.upper && self.words.len() > 4) {
                *counts.entry(self.words.join(" ")).or_insert(0) += 1;
            }
        } else if self.words.len() == 1 && self.caps == 1 && is_surname_form(&self.words[0]) {
            *counts.entry(self.words[0].clone()).or_insert(0) += 1;
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
        let title_case = is_cap_word(word);
        let upper_case = !title_case && is_upper_word(word);
        if title_case || upper_case {
            if run.words.is_empty() && TITLE_WORDS.contains(&word.to_lowercase().as_str()) {
                // a job title at the start of a phrase is not part of the name
                continue;
            }
            run.words.push(word.to_string());
            run.caps += 1;
            run.upper |= upper_case;
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
    deidentify_with_links(text, key, dictionary, &HashMap::new())
}

/// Like [`deidentify`], and a code (tax number, phone, e-mail) whose pseudonym
/// is linked to a person is replaced by that person's pseudonym.
pub fn deidentify_with_links(
    text: &str,
    key: &[u8],
    dictionary: &[DictEntry],
    links: &HashMap<String, String>,
) -> Result<Deidentified, TokenError> {
    let chars: Vec<char> = text.chars().collect();
    let mut spans = dictionary_spans(&chars, dictionary, key)?;
    spans.extend(email_spans(&chars, key)?);
    spans.extend(number_spans(&chars, key)?);
    spans.extend(iban_spans(&chars, key)?);
    spans.extend(card_spans(&chars, key)?);
    spans.extend(birth_date_spans(&chars, key)?);
    spans.extend(plate_spans(&chars, key)?);
    spans.extend(address_spans(&chars, key)?);
    for span in spans.iter_mut().filter(|s| s.kind != Kind::Name) {
        if let Some(person) = links.get(&span.token) {
            span.token = person.clone();
        }
    }
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
    fn a_job_title_before_a_name_is_not_part_of_the_candidate() {
        let result = run("Заступник Олексій Коваль підписав акт.", &[]);
        let texts: Vec<&str> = result.candidates.iter().map(|c| c.text.as_str()).collect();
        assert_eq!(texts, vec!["Олексій Коваль"]);
    }

    #[test]
    fn names_in_capitals_are_candidates() {
        let result = run("позов подав ІВАНЕНКО ПЕТРО ІВАНОВИЧ.", &[]);
        let texts: Vec<&str> = result.candidates.iter().map(|c| c.text.as_str()).collect();
        assert_eq!(texts, vec!["ІВАНЕНКО ПЕТРО ІВАНОВИЧ"]);
    }

    #[test]
    fn capital_abbreviations_are_not_candidates() {
        let result = run("оплатив ТОВ ЄДРПОУ 12345678 у ДПС.", &[]);
        assert!(result.candidates.is_empty());
    }

    #[test]
    fn a_long_heading_in_capitals_is_not_a_candidate() {
        let result = run(
            "ЗАТВЕРДЖЕННЯ ПОРЯДКУ ВИКОНАННЯ ЗАХОДІВ ПРОГРАМИ ЗБЕРЕЖЕННЯ",
            &[],
        );
        assert!(result.candidates.is_empty());
    }

    #[test]
    fn declined_surnames_are_reported_as_single_candidates() {
        let result = run("Звернулися до Іваненка та до Мельничука.", &[]);
        let texts: Vec<&str> = result.candidates.iter().map(|c| c.text.as_str()).collect();
        assert!(texts.contains(&"Іваненка"));
        assert!(texts.contains(&"Мельничука"));
    }

    #[test]
    fn ordinary_capitalised_words_are_not_single_candidates() {
        let result = run("жив у місті Київ і Львівській області.", &[]);
        assert!(result.candidates.is_empty());
    }

    #[test]
    fn an_alias_for_a_declined_form_removes_the_candidate() {
        let dict = [DictEntry::alias("Іваненка", "N-ALIAS")];
        let result = run("Звернулися до Іваненка.", &dict);
        assert_eq!(result.text, "Звернулися до N-ALIAS.");
        assert!(result.candidates.is_empty());
    }

    const IBAN: &str = "UA213223130000026007233566001";

    #[test]
    fn a_bank_account_number_is_replaced_in_any_grouping() {
        let tok = token(Kind::Iban, IBAN, &key()).unwrap();
        let result = run(
            "Рахунок UA21 3223 1300 0002 6007 2335 6600 1, або UA213223130000026007233566001.",
            &[],
        );
        assert_eq!(result.text, format!("Рахунок {tok}, або {tok}."));
        assert_eq!(result.replacements.len(), 1);
        assert_eq!(result.replacements[0].count, 2);
        assert_eq!(result.replacements[0].kind, Kind::Iban);
    }

    #[test]
    fn a_number_that_is_not_a_valid_iban_stays() {
        let text = "Рахунок UA213223130000026007233566000 не існує.";
        let result = run(text, &[]);
        assert!(result.text.contains("UA213223130000026007233566000"));
        assert!(result.replacements.iter().all(|r| r.kind != Kind::Iban));
    }

    #[test]
    fn a_date_of_birth_after_its_label_is_replaced() {
        let tok = token(Kind::Birth, "12.03.1985", &key()).unwrap();
        let result = run("дата народження: 12.03.1985, проживає у Львові", &[]);
        assert_eq!(result.text, format!("дата народження: {tok}, проживає у Львові"));
    }

    #[test]
    fn spelled_out_and_numeric_birth_dates_give_one_pseudonym() {
        let tok = token(Kind::Birth, "1985-03-12", &key()).unwrap();
        let result = run("народився 12 березня 1985 року; нар. 12/03/1985", &[]);
        assert_eq!(result.text, format!("народився {tok} року; нар. {tok}"));
    }

    #[test]
    fn a_date_before_the_birth_marker_is_replaced() {
        let tok = token(Kind::Birth, "12.03.1985", &key()).unwrap();
        let result = run("Іваненко П. (12.03.1985 р.н.) звернувся", &[]);
        assert_eq!(result.text, format!("Іваненко П. ({tok} р.н.) звернувся"));
    }

    #[test]
    fn other_dates_stay() {
        let text = "Договір від 12.03.1985 і тендер 01.10.2026, дата публікації 05.10.2026.";
        let result = run(text, &[]);
        assert_eq!(result.text, text);
    }

    #[test]
    fn an_impossible_date_and_a_bare_year_stay() {
        let text = "народився 31.02.1985; 1985 р.н.";
        let result = run(text, &[]);
        assert_eq!(result.text, text);
    }

    #[test]
    fn a_payment_card_number_is_replaced_in_any_grouping() {
        let tok = token(Kind::Card, "4111111111111111", &key()).unwrap();
        let result = run(
            "Картка 4111 1111 1111 1111 і 4111111111111111 та 4111-1111-1111-1111.",
            &[],
        );
        assert_eq!(result.text, format!("Картка {tok} і {tok} та {tok}."));
        assert_eq!(result.replacements.len(), 1);
        assert_eq!(result.replacements[0].count, 3);
        assert_eq!(result.replacements[0].kind, Kind::Card);
    }

    #[test]
    fn a_number_that_fails_the_luhn_check_stays() {
        let text = "Картка 4111 1111 1111 1112 не існує.";
        let result = run(text, &[]);
        assert_eq!(result.text, text);
    }

    #[test]
    fn tax_numbers_and_phones_are_not_cards() {
        let result = run("ІПН 1234567890, тел. 0501234567.", &[]);
        assert!(result.replacements.iter().all(|r| r.kind != Kind::Card));
    }

    #[test]
    fn two_long_numbers_side_by_side_are_not_a_card() {
        let text = "коди 12345678 87654321 у таблиці";
        let result = run(text, &[]);
        assert_eq!(result.text, text);
    }

    #[test]
    fn vehicle_plates_in_all_spellings_give_one_pseudonym() {
        let tok = token(Kind::Plate, "АА1234ВВ", &key()).unwrap();
        let result = run("авто АА 1234 ВВ і AA1234BB, не АА 123 ВВ.", &[]);
        assert_eq!(result.text, format!("авто {tok} і {tok}, не АА 123 ВВ."));
    }

    #[test]
    fn an_address_with_a_house_number_is_replaced() {
        let text = "Проживає за адресою: м. Львів, вул. Шевченка, 15, кв. 7. Працює вдома.";
        let tok = token(Kind::Address, "м. Львів, вул. Шевченка, 15, кв. 7", &key()).unwrap();
        let result = run(text, &[]);
        assert_eq!(
            result.text,
            format!("Проживає за адресою: {tok}. Працює вдома.")
        );
    }

    #[test]
    fn address_spellings_give_one_pseudonym() {
        let a = run("жив на вул. Шевченка, буд. 15", &[]);
        let b = run("жив на вулиця Шевченка 15", &[]);
        assert_eq!(a.text, b.text);
        assert!(!a.text.contains("Шевченка"));
    }

    #[test]
    fn a_full_name_is_found_in_other_word_orders() {
        let dict = [DictEntry::name("Іваненко Петро Олексійович")];
        let tok = token(Kind::Name, "Іваненко Петро Олексійович", &key()).unwrap();
        let result = run(
            "Петро Олексійович Іваненко та Іваненко Петро, Петро Іваненко і Петро Олексійович.",
            &dict,
        );
        assert_eq!(result.text, format!("{tok} та {tok}, {tok} і {tok}."));
    }

    #[test]
    fn a_two_word_name_is_found_in_the_other_order() {
        let dict = [DictEntry::name("Іваненко Петро")];
        let tok = token(Kind::Name, "Іваненко Петро", &key()).unwrap();
        let result = run("Петро Іваненко підписав.", &dict);
        assert_eq!(result.text, format!("{tok} підписав."));
    }

    #[test]
    fn a_form_that_fits_two_people_is_left_for_the_investigator() {
        let dict = [
            DictEntry::name("Іваненко Петро Олексійович"),
            DictEntry::name("Іваненко Петро Іванович"),
        ];
        let result = run("Іваненко Петро прийшов.", &dict);
        assert_eq!(result.text, "Іваненко Петро прийшов.");
        let texts: Vec<&str> = result.candidates.iter().map(|c| c.text.as_str()).collect();
        assert!(texts.contains(&"Іваненко Петро"));
        let first = token(Kind::Name, "Іваненко Петро Олексійович", &key()).unwrap();
        let full = run("Іваненко Петро Олексійович прийшов.", &dict);
        assert_eq!(full.text, format!("{first} прийшов."));
    }

    #[test]
    fn an_explicit_entry_wins_over_a_generated_form() {
        let dict = [
            DictEntry::name("Іваненко Петро"),
            DictEntry::alias("Петро Іваненко", "N-OTHER"),
        ];
        let result = run("Петро Іваненко підписав.", &dict);
        assert_eq!(result.text, "N-OTHER підписав.");
    }

    #[test]
    fn initials_are_not_expanded_automatically() {
        let dict = [DictEntry::name("Іваненко Петро")];
        let result = run("Іваненко П. підписав.", &dict);
        assert_eq!(result.text, "Іваненко П. підписав.");
    }

    #[test]
    fn a_street_without_a_house_number_stays() {
        let text = "на вул. Шевченка сталася аварія";
        let result = run(text, &[]);
        assert_eq!(result.text, text);
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
