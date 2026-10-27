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
    Text,
}

impl Kind {
    pub fn as_str(self) -> &'static str {
        match self {
            Kind::Name => "name",
            Kind::Ipn => "ipn",
            Kind::Edrpou => "edrpou",
            Kind::Phone => "phone",
            Kind::Text => "text",
        }
    }

    /// One-letter prefix of the token.
    pub fn prefix(self) -> char {
        match self {
            Kind::Name => 'N',
            Kind::Ipn => 'I',
            Kind::Edrpou => 'E',
            Kind::Phone => 'T',
            Kind::Text => 'X',
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

const APOSTROPHES: [char; 6] = [
    '\u{2019}', '\u{02bc}', '\u{2018}', '`', '\u{00b4}', '\u{2032}',
];
const DASHES: [char; 6] = [
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
    fn empty_text_is_rejected() {
        assert_eq!(normalize(Kind::Name, "   "), Err(NormalizeError::Empty));
    }
}
