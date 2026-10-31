//! Canonical form of a value before pseudonymisation (rules version `norm_v1`).
//!
//! The same real-world value written in different ways must normalise to the
//! same string, otherwise its pseudonym differs.

use std::fmt;

use unicode_normalization::UnicodeNormalization;

/// Version of the normalisation rules; part of every pseudonym.
pub const NORM_VERSION: &str = "norm_v1";

/// What kind of value is being normalised.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    Name,
    Ipn,
    Edrpou,
    Phone,
    Email,
    Text,
    /// Ukrainian bank account number (IBAN).
    Iban,
    /// Vehicle registration plate.
    Plate,
    /// Street address (street, house, flat).
    Address,
    /// Payment card number (13-19 digits, valid Luhn check).
    Card,
    /// Date of birth in any common spelling; the canonical form is ISO (YYYY-MM-DD).
    Birth,
}

impl Kind {
    pub fn as_str(self) -> &'static str {
        match self {
            Kind::Name => "name",
            Kind::Ipn => "ipn",
            Kind::Edrpou => "edrpou",
            Kind::Phone => "phone",
            Kind::Email => "email",
            Kind::Text => "text",
            Kind::Iban => "iban",
            Kind::Plate => "plate",
            Kind::Address => "address",
            Kind::Card => "card",
            Kind::Birth => "birth",
        }
    }

    /// The kind named by `as_str`, if any.
    pub fn parse(text: &str) -> Option<Kind> {
        match text {
            "name" => Some(Kind::Name),
            "ipn" => Some(Kind::Ipn),
            "edrpou" => Some(Kind::Edrpou),
            "phone" => Some(Kind::Phone),
            "email" => Some(Kind::Email),
            "text" => Some(Kind::Text),
            "iban" => Some(Kind::Iban),
            "plate" => Some(Kind::Plate),
            "address" => Some(Kind::Address),
            "card" => Some(Kind::Card),
            "birth" => Some(Kind::Birth),
            _ => None,
        }
    }

    /// One-letter prefix of the token.
    pub fn prefix(self) -> char {
        match self {
            Kind::Name => 'N',
            Kind::Ipn => 'I',
            Kind::Edrpou => 'E',
            Kind::Phone => 'T',
            Kind::Email => 'M',
            Kind::Text => 'X',
            Kind::Iban => 'B',
            Kind::Plate => 'V',
            Kind::Address => 'A',
            Kind::Card => 'K',
            Kind::Birth => 'D',
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NormalizeError {
    Empty,
    /// The value does not have the digits a code of this kind needs.
    BadFormat(Kind),
}

impl fmt::Display for NormalizeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            NormalizeError::Empty => write!(f, "empty value"),
            NormalizeError::BadFormat(kind) => write!(f, "bad format for {}", kind.as_str()),
        }
    }
}

impl std::error::Error for NormalizeError {}

pub(crate) const APOSTROPHES: [char; 6] = [
    '\u{2019}', '\u{02bc}', '\u{2018}', '`', '\u{00b4}', '\u{2032}',
];
pub(crate) const DASHES: [char; 6] = [
    '\u{2010}', '\u{2011}', '\u{2012}', '\u{2013}', '\u{2014}', '\u{2212}',
];

fn normalize_text(value: &str) -> String {
    let unified: String = value
        .nfc()
        .map(|ch| {
            if APOSTROPHES.contains(&ch) {
                '\''
            } else if DASHES.contains(&ch) {
                '-'
            } else {
                ch
            }
        })
        .collect();
    let collapsed = unified.split_whitespace().collect::<Vec<_>>().join(" ");
    collapsed
        .replace(" - ", "-")
        .replace(" -", "-")
        .replace("- ", "-")
        .to_lowercase()
}

/// Street-level markers of an address in common spellings, with their canonical form.
pub(crate) fn address_marker(word: &str) -> Option<&'static str> {
    Some(match word {
        "вул" | "вулиця" | "вулиці" => "вул",
        "просп" | "проспект" | "проспекту" => "просп",
        "пров" | "провулок" | "провулку" => "пров",
        "бульв" | "бульвар" | "бульвару" => "бульв",
        "пл" | "площа" | "площі" => "пл",
        "шосе" => "шосе",
        "узв" | "узвіз" | "узвозу" => "узв",
        "наб" | "набережна" | "набережній" => "наб",
        "мкр" | "мікрорайон" | "мікрорайоні" => "мкр",
        "тупик" | "тупику" => "тупик",
        "алея" | "алеї" => "алея",
        _ => return None,
    })
}

/// Lower case, no punctuation, markers in one canonical spelling, house words dropped,
/// so "вул. Шевченка, буд. 15" and "вулиця Шевченка 15" give the same form.
fn normalize_address(value: &str) -> String {
    let lowered = normalize_text(value);
    let cleaned: String = lowered
        .chars()
        .map(|c| {
            if matches!(c, ',' | '.' | ';' | '\u{2116}') {
                ' '
            } else {
                c
            }
        })
        .collect();
    let mut words: Vec<&str> = Vec::new();
    for word in cleaned.split_whitespace() {
        let canonical = match word {
            "місто" | "місті" => "м",
            "квартира" | "квартири" => "кв",
            "офіс" | "офісу" => "оф",
            "буд" | "будинок" | "будинку" => continue,
            other => address_marker(other).unwrap_or(other),
        };
        words.push(canonical);
    }
    words.join(" ")
}

/// A letter that is used on Ukrainian plates, as its Cyrillic form (Latin look-alikes
/// are accepted and mapped).
pub(crate) fn plate_letter(c: char) -> Option<char> {
    let upper = c.to_uppercase().next().unwrap_or(c);
    Some(match upper {
        '\u{410}' | 'A' => '\u{410}',
        '\u{412}' | 'B' => '\u{412}',
        '\u{415}' | 'E' => '\u{415}',
        '\u{406}' | 'I' => '\u{406}',
        '\u{41a}' | 'K' => '\u{41a}',
        '\u{41c}' | 'M' => '\u{41c}',
        '\u{41d}' | 'H' => '\u{41d}',
        '\u{41e}' | 'O' => '\u{41e}',
        '\u{420}' | 'P' => '\u{420}',
        '\u{421}' | 'C' => '\u{421}',
        '\u{422}' | 'T' => '\u{422}',
        '\u{425}' | 'X' => '\u{425}',
        _ => return None,
    })
}

/// `UA` and 27 digits with a valid ISO 7064 check (mod 97).
fn valid_ua_iban(text: &str) -> bool {
    if text.len() != 29 || !text.starts_with("UA") || !text[2..].chars().all(|c| c.is_ascii_digit())
    {
        return false;
    }
    let rearranged = format!("{}{}", &text[4..], &text[..4]);
    let mut remainder: u32 = 0;
    for c in rearranged.chars() {
        let value = c.to_digit(36).unwrap_or(0);
        remainder = if value >= 10 {
            (remainder * 100 + value) % 97
        } else {
            (remainder * 10 + value) % 97
        };
    }
    remainder == 1
}

/// Month number of a Ukrainian month name in the genitive ("12 березня 1985").
pub(crate) fn month_number(word: &str) -> Option<u32> {
    Some(match word {
        "січня" => 1,
        "лютого" => 2,
        "березня" => 3,
        "квітня" => 4,
        "травня" => 5,
        "червня" => 6,
        "липня" => 7,
        "серпня" => 8,
        "вересня" => 9,
        "жовтня" => 10,
        "листопада" => 11,
        "грудня" => 12,
        _ => return None,
    })
}

fn valid_date(year: u32, month: u32, day: u32) -> bool {
    if !(1900..=2099).contains(&year) || !(1..=12).contains(&month) {
        return false;
    }
    let leap = year.is_multiple_of(4) && (!year.is_multiple_of(100) || year.is_multiple_of(400));
    let days = match month {
        2 if leap => 29,
        2 => 28,
        4 | 6 | 9 | 11 => 30,
        _ => 31,
    };
    (1..=days).contains(&day)
}

/// ISO form of a date written "12.03.1985", "12/03/1985", "1985-03-12" or "12 березня 1985".
fn normalize_birth(value: &str) -> Option<String> {
    let lowered = value.trim().to_lowercase();
    let words: Vec<&str> = lowered.split_whitespace().collect();
    let (year, month, day) = if words.len() >= 3 {
        let day: u32 = words[0].parse().ok()?;
        let month = month_number(words[1])?;
        let year: u32 = words[2].parse().ok()?;
        (year, month, day)
    } else {
        let parts: Vec<&str> = lowered.split(['.', '/', '-']).collect();
        let all_digits = parts
            .iter()
            .all(|p| !p.is_empty() && p.chars().all(|c| c.is_ascii_digit()));
        if parts.len() != 3 || !all_digits {
            return None;
        }
        let numbers: Vec<u32> = parts
            .iter()
            .map(|p| p.parse().ok())
            .collect::<Option<Vec<u32>>>()?;
        if parts[0].len() == 4 {
            (numbers[0], numbers[1], numbers[2])
        } else if parts[2].len() == 4 {
            (numbers[2], numbers[1], numbers[0])
        } else {
            return None;
        }
    };
    valid_date(year, month, day).then(|| format!("{year:04}-{month:02}-{day:02}"))
}

/// The Luhn check used by payment card numbers.
fn luhn_valid(digits: &str) -> bool {
    let mut sum = 0;
    for (index, c) in digits.chars().rev().enumerate() {
        let Some(mut value) = c.to_digit(10) else {
            return false;
        };
        if index % 2 == 1 {
            value *= 2;
            if value > 9 {
                value -= 9;
            }
        }
        sum += value;
    }
    sum % 10 == 0
}

fn digits(value: &str) -> String {
    value.nfkc().filter(char::is_ascii_digit).collect()
}

/// Canonical form of `value` for `kind`.
///
/// `Name`/`Text`: Unicode NFC, unified apostrophes and hyphens, collapsed
/// spaces, lower-cased. `Ipn` (10 digits) and `Edrpou` (8 digits): digits only.
/// `Phone`: digits only; a national `0XXXXXXXXX` becomes `380XXXXXXXXX`.
pub fn normalize(kind: Kind, value: &str) -> Result<String, NormalizeError> {
    let result = match kind {
        Kind::Name | Kind::Text => normalize_text(value),
        Kind::Ipn => {
            let d = digits(value);
            if d.len() != 10 {
                return Err(NormalizeError::BadFormat(kind));
            }
            d
        }
        Kind::Edrpou => {
            let d = digits(value);
            if d.len() != 8 {
                return Err(NormalizeError::BadFormat(kind));
            }
            d
        }
        Kind::Email => {
            let lowered = value.trim().to_lowercase();
            let valid = lowered.contains('@')
                && !lowered.contains(char::is_whitespace)
                && lowered.rsplit('@').next().is_some_and(|d| d.contains('.'));
            if !valid {
                return Err(NormalizeError::BadFormat(kind));
            }
            lowered
        }
        Kind::Iban => {
            let compact: String = value
                .chars()
                .filter(|c| !c.is_whitespace() && *c != '-')
                .collect::<String>()
                .to_uppercase();
            if !valid_ua_iban(&compact) {
                return Err(NormalizeError::BadFormat(kind));
            }
            compact
        }
        Kind::Plate => {
            let chars: Vec<char> = value
                .chars()
                .filter(|c| !c.is_whitespace() && *c != '-')
                .collect();
            let shaped = chars.len() == 8
                && chars[..2]
                    .iter()
                    .chain(&chars[6..])
                    .all(|c| plate_letter(*c).is_some())
                && chars[2..6].iter().all(char::is_ascii_digit);
            if !shaped {
                return Err(NormalizeError::BadFormat(kind));
            }
            chars
                .iter()
                .enumerate()
                .map(|(i, c)| {
                    if (2..6).contains(&i) {
                        *c
                    } else {
                        plate_letter(*c).unwrap_or(*c)
                    }
                })
                .collect()
        }
        Kind::Address => normalize_address(value),
        Kind::Birth => match normalize_birth(value) {
            Some(date) => date,
            None => return Err(NormalizeError::BadFormat(kind)),
        },
        Kind::Card => {
            let d = digits(value);
            if !(13..=19).contains(&d.len()) || !luhn_valid(&d) {
                return Err(NormalizeError::BadFormat(kind));
            }
            d
        }
        Kind::Phone => {
            let mut d = digits(value);
            if d.len() == 10 && d.starts_with('0') {
                d = format!("38{d}");
            }
            if d.len() != 12 || !d.starts_with("380") {
                return Err(NormalizeError::BadFormat(kind));
            }
            d
        }
    };
    if result.is_empty() {
        return Err(NormalizeError::Empty);
    }
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn name_variants_are_equal() {
        let a = normalize(Kind::Name, "  Іваненко   Петро-Олексійович ").unwrap();
        let b = normalize(Kind::Name, "іваненко петро \u{2013} олексійович").unwrap();
        assert_eq!(a, "іваненко петро-олексійович");
        assert_eq!(a, b);
    }

    #[test]
    fn apostrophes_are_unified() {
        let a = normalize(Kind::Name, "Мар\u{2019}яна").unwrap();
        let b = normalize(Kind::Name, "Мар'яна").unwrap();
        let c = normalize(Kind::Name, "Мар\u{02bc}яна").unwrap();
        assert_eq!(a, b);
        assert_eq!(b, c);
    }

    #[test]
    fn iban_needs_a_valid_check_and_ignores_spaces() {
        let a = normalize(Kind::Iban, "UA21 3223 1300 0002 6007 2335 6600 1").unwrap();
        let b = normalize(Kind::Iban, "ua213223130000026007233566001").unwrap();
        assert_eq!(a, "UA213223130000026007233566001");
        assert_eq!(a, b);
        assert_eq!(
            normalize(Kind::Iban, "UA213223130000026007233566000"),
            Err(NormalizeError::BadFormat(Kind::Iban))
        );
        assert!(normalize(Kind::Iban, "UA21").is_err());
    }

    #[test]
    fn birth_dates_in_all_spellings_give_one_form() {
        let a = normalize(Kind::Birth, "12.03.1985").unwrap();
        let b = normalize(Kind::Birth, "12/03/1985").unwrap();
        let c = normalize(Kind::Birth, "1985-03-12").unwrap();
        let d = normalize(Kind::Birth, "12 березня 1985").unwrap();
        assert_eq!(a, "1985-03-12");
        assert_eq!(a, b);
        assert_eq!(a, c);
        assert_eq!(a, d);
    }

    #[test]
    fn impossible_dates_are_refused() {
        assert!(normalize(Kind::Birth, "31.02.1985").is_err());
        assert!(normalize(Kind::Birth, "12.13.1985").is_err());
        assert!(normalize(Kind::Birth, "29.02.1900").is_err());
        assert!(normalize(Kind::Birth, "29.02.2000").is_ok());
        assert!(normalize(Kind::Birth, "12.03.85").is_err());
    }

    #[test]
    fn card_numbers_need_the_luhn_check() {
        let a = normalize(Kind::Card, "4111 1111 1111 1111").unwrap();
        let b = normalize(Kind::Card, "4111-1111-1111-1111").unwrap();
        assert_eq!(a, "4111111111111111");
        assert_eq!(a, b);
        assert!(normalize(Kind::Card, "4111 1111 1111 1112").is_err());
        assert!(normalize(Kind::Card, "4111 1111").is_err());
    }

    #[test]
    fn plate_spellings_give_one_form() {
        let a = normalize(Kind::Plate, "АА 1234 ВВ").unwrap();
        let b = normalize(Kind::Plate, "AA1234BB").unwrap();
        let c = normalize(Kind::Plate, "аа-1234-вв").unwrap();
        assert_eq!(a, b);
        assert_eq!(a, c);
        assert!(normalize(Kind::Plate, "ЖЖ 1234 ВВ").is_err());
        assert!(normalize(Kind::Plate, "АА 123 ВВ").is_err());
    }

    #[test]
    fn address_spellings_give_one_form() {
        let a = normalize(Kind::Address, "вул. Шевченка, буд. 15, кв. 7").unwrap();
        let b = normalize(Kind::Address, "Вулиця  Шевченка 15 квартира 7").unwrap();
        assert_eq!(a, "вул шевченка 15 кв 7");
        assert_eq!(a, b);
    }

    #[test]
    fn codes_keep_digits_only() {
        assert_eq!(normalize(Kind::Ipn, "1234-567 890").unwrap(), "1234567890");
        assert_eq!(normalize(Kind::Edrpou, "12 345 678").unwrap(), "12345678");
    }

    #[test]
    fn codes_check_length() {
        assert_eq!(
            normalize(Kind::Ipn, "12345"),
            Err(NormalizeError::BadFormat(Kind::Ipn))
        );
        assert_eq!(
            normalize(Kind::Edrpou, "123456789"),
            Err(NormalizeError::BadFormat(Kind::Edrpou))
        );
    }

    #[test]
    fn phone_forms_are_equal() {
        let a = normalize(Kind::Phone, "+38 (050) 123-45-67").unwrap();
        let b = normalize(Kind::Phone, "0501234567").unwrap();
        assert_eq!(a, "380501234567");
        assert_eq!(a, b);
    }

    #[test]
    fn phone_must_be_ukrainian() {
        assert_eq!(
            normalize(Kind::Phone, "+1 202 555 0100"),
            Err(NormalizeError::BadFormat(Kind::Phone))
        );
    }

    #[test]
    fn email_is_lowercased_and_checked() {
        assert_eq!(
            normalize(Kind::Email, " Ivan@Example.ORG ").unwrap(),
            "ivan@example.org"
        );
        assert_eq!(
            normalize(Kind::Email, "not an email"),
            Err(NormalizeError::BadFormat(Kind::Email))
        );
    }

    #[test]
    fn empty_text_is_rejected() {
        assert_eq!(normalize(Kind::Name, "   "), Err(NormalizeError::Empty));
    }
}
