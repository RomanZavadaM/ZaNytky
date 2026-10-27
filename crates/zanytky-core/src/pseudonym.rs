//! Keyed one-to-one pseudonyms.
//!
//! `token = PREFIX "-" base32(HMAC-SHA256(key, kind 0x1F norm_version 0x1F canonical)[..16])`
//!
//! The same value always gives the same token, so records can be matched and
//! grouped. Without the key the token cannot be recomputed, and the token alone
//! never reveals the value.

use hmac::{Hmac, Mac};
use sha2::Sha256;

use crate::normalize::{normalize, Kind, NormalizeError, NORM_VERSION};

/// Shortest accepted system key.
pub const MIN_KEY_BYTES: usize = 32;

const ALPHABET: &[u8; 32] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ234567";

/// A fresh random system key. The caller keeps it outside the repository.
pub fn new_key() -> Result<[u8; MIN_KEY_BYTES], getrandom::Error> {
    let mut key = [0u8; MIN_KEY_BYTES];
    getrandom::getrandom(&mut key)?;
    Ok(key)
}

#[derive(Debug, PartialEq, Eq)]
pub enum TokenError {
    KeyTooShort,
    Value(NormalizeError),
}

impl std::fmt::Display for TokenError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TokenError::KeyTooShort => write!(f, "system key must be at least 32 bytes"),
            TokenError::Value(error) => write!(f, "{error}"),
        }
    }
}

impl std::error::Error for TokenError {}

impl From<NormalizeError> for TokenError {
    fn from(error: NormalizeError) -> Self {
        TokenError::Value(error)
    }
}

fn base32(bytes: &[u8]) -> String {
    let mut out = String::new();
    let mut buffer: u32 = 0;
    let mut bits = 0;
    for &byte in bytes {
        buffer = (buffer << 8) | u32::from(byte);
        bits += 8;
        while bits >= 5 {
            bits -= 5;
            out.push(ALPHABET[((buffer >> bits) & 31) as usize] as char);
        }
        buffer &= (1 << bits) - 1;
    }
    if bits > 0 {
        out.push(ALPHABET[((buffer << (5 - bits)) & 31) as usize] as char);
    }
    out
}

/// Pseudonym of `value` for `kind` under `key`.
pub fn token(kind: Kind, value: &str, key: &[u8]) -> Result<String, TokenError> {
    if key.len() < MIN_KEY_BYTES {
        return Err(TokenError::KeyTooShort);
    }
    let canonical = normalize(kind, value)?;
    let message = format!("{}\u{1f}{}\u{1f}{}", kind.as_str(), NORM_VERSION, canonical);
    let mut mac = Hmac::<Sha256>::new_from_slice(key).map_err(|_| TokenError::KeyTooShort)?;
    mac.update(message.as_bytes());
    let digest = mac.finalize().into_bytes();
    Ok(format!("{}-{}", kind.prefix(), base32(&digest[..16])))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn key() -> Vec<u8> {
        (0u8..32).collect()
    }

    /// Vectors produced independently with Python (hmac + hashlib + base64) on fictional data.
    #[test]
    fn known_vectors() {
        let k = key();
        assert_eq!(
            token(Kind::Name, "Іваненко Петро-Олексійович", &k).unwrap(),
            "N-DNNKSHBLI2EYTBLZ3HCPEVW4OQ"
        );
        assert_eq!(
            token(Kind::Ipn, "1234567890", &k).unwrap(),
            "I-WWFIFCVRWD2UN3KWOWCPW5H4UI"
        );
        assert_eq!(
            token(Kind::Edrpou, "12345678", &k).unwrap(),
            "E-OMAXVYGUECV3OZOFDTDOPHVA4M"
        );
        assert_eq!(
            token(Kind::Phone, "+38 (050) 123-45-67", &k).unwrap(),
            "T-D67OAZJLLQXC7U6ZY5ZRS5CPB4"
        );
        assert_eq!(
            token(Kind::Text, "Тест", &k).unwrap(),
            "X-HGKNFLRACKSLPYPX4E4KJRDWDA"
        );
    }

    #[test]
    fn spelling_variants_give_one_token() {
        let k = key();
        let a = token(Kind::Name, "  Іваненко   Петро-Олексійович ", &k).unwrap();
        let b = token(Kind::Name, "іваненко петро \u{2013} олексійович", &k).unwrap();
        assert_eq!(a, b);
    }

    #[test]
    fn kinds_do_not_mix() {
        let k = key();
        let a = token(Kind::Name, "12345678", &k).unwrap();
        let b = token(Kind::Text, "12345678", &k).unwrap();
        assert_ne!(a, b);
    }

    #[test]
    fn different_keys_give_different_tokens() {
        let k1 = key();
        let mut k2 = key();
        k2[0] ^= 1;
        assert_ne!(
            token(Kind::Ipn, "1234567890", &k1).unwrap(),
            token(Kind::Ipn, "1234567890", &k2).unwrap()
        );
    }

    #[test]
    fn short_key_is_rejected() {
        assert_eq!(
            token(Kind::Ipn, "1234567890", &[1u8; 16]),
            Err(TokenError::KeyTooShort)
        );
    }

    #[test]
    fn token_does_not_contain_the_value() {
        let t = token(Kind::Ipn, "1234567890", &key()).unwrap();
        assert!(!t.contains("1234567890"));
        assert_eq!(t.len(), 2 + 26);
    }

    #[test]
    fn invalid_value_is_an_error() {
        assert!(matches!(
            token(Kind::Ipn, "12", &key()),
            Err(TokenError::Value(_))
        ));
    }

    #[test]
    fn new_key_is_random_and_long_enough() {
        let a = new_key().unwrap();
        let b = new_key().unwrap();
        assert_eq!(a.len(), MIN_KEY_BYTES);
        assert_ne!(a, b);
    }
}
