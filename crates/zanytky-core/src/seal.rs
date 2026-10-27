//! Sealed originals: age encryption to the investigator's public key(s).
//!
//! The system holds only public keys, so it can seal but never open. Opening
//! needs the investigator's private identity, which the investigator keeps.
//! No own cryptography: the audited `age` crate does the work.

use std::fmt;
use std::io::{Read, Write};
use std::str::FromStr;

use crate::normalize::{Kind, NORM_VERSION};

const MAGIC: &str = "ZSEAL1";

pub use age::x25519::{Identity, Recipient};

/// What a sealed message holds once opened.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Sealed {
    pub kind: Kind,
    pub norm: String,
    pub value: String,
}

#[derive(Debug, PartialEq, Eq)]
pub enum SealError {
    NoRecipients,
    /// The age library refused the operation (wrong key, damaged data, ...).
    Crypto(String),
    /// The opened data is not a ZaNytky sealed value.
    BadPayload,
}

impl fmt::Display for SealError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SealError::NoRecipients => write!(f, "at least one recipient is required"),
            SealError::Crypto(message) => write!(f, "cannot seal or open: {message}"),
            SealError::BadPayload => write!(f, "not a sealed value"),
        }
    }
}

impl std::error::Error for SealError {}

fn crypto<E: fmt::Display>(error: E) -> SealError {
    SealError::Crypto(error.to_string())
}

/// Parse an investigator's public key (`age1...`).
pub fn parse_recipient(text: &str) -> Result<Recipient, SealError> {
    Recipient::from_str(text.trim()).map_err(|error| SealError::Crypto(error.to_string()))
}

/// Parse an investigator's private identity (`AGE-SECRET-KEY-1...`).
pub fn parse_identity(text: &str) -> Result<Identity, SealError> {
    Identity::from_str(text.trim()).map_err(|error| SealError::Crypto(error.to_string()))
}

/// Seal the original `value` of `kind` for the given recipients.
pub fn seal(kind: Kind, value: &str, recipients: &[Recipient]) -> Result<Vec<u8>, SealError> {
    if recipients.is_empty() {
        return Err(SealError::NoRecipients);
    }
    let payload = format!("{MAGIC}\n{}\n{NORM_VERSION}\n{value}", kind.as_str());
    let targets: Vec<&dyn age::Recipient> = recipients
        .iter()
        .map(|recipient| recipient as &dyn age::Recipient)
        .collect();
    let encryptor = age::Encryptor::with_recipients(targets.into_iter()).map_err(crypto)?;
    let mut out = Vec::new();
    let mut writer = encryptor.wrap_output(&mut out).map_err(crypto)?;
    writer.write_all(payload.as_bytes()).map_err(crypto)?;
    writer.finish().map_err(crypto)?;
    Ok(out)
}

/// Open a sealed value with the investigator's private identity.
pub fn unseal(data: &[u8], identity: &Identity) -> Result<Sealed, SealError> {
    let decryptor = age::Decryptor::new(data).map_err(crypto)?;
    let mut reader = decryptor
        .decrypt(std::iter::once(identity as &dyn age::Identity))
        .map_err(crypto)?;
    let mut bytes = Vec::new();
    reader.read_to_end(&mut bytes).map_err(crypto)?;
    let text = String::from_utf8(bytes).map_err(|_| SealError::BadPayload)?;
    let mut parts = text.splitn(4, '\n');
    if parts.next() != Some(MAGIC) {
        return Err(SealError::BadPayload);
    }
    let kind = parts
        .next()
        .and_then(Kind::parse)
        .ok_or(SealError::BadPayload)?;
    let norm = parts.next().ok_or(SealError::BadPayload)?.to_string();
    let value = parts.next().ok_or(SealError::BadPayload)?.to_string();
    Ok(Sealed { kind, norm, value })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pair() -> (Identity, Recipient) {
        let identity = Identity::generate();
        let recipient = identity.to_public();
        (identity, recipient)
    }

    #[test]
    fn round_trip() {
        let (identity, recipient) = pair();
        let data = seal(Kind::Name, "Іваненко Петро", &[recipient]).unwrap();
        let opened = unseal(&data, &identity).unwrap();
        assert_eq!(opened.kind, Kind::Name);
        assert_eq!(opened.norm, NORM_VERSION);
        assert_eq!(opened.value, "Іваненко Петро");
    }

    #[test]
    fn value_with_newlines_survives() {
        let (identity, recipient) = pair();
        let data = seal(Kind::Text, "перший рядок\nдругий рядок\n", &[recipient]).unwrap();
        assert_eq!(
            unseal(&data, &identity).unwrap().value,
            "перший рядок\nдругий рядок\n"
        );
    }

    #[test]
    fn any_listed_investigator_can_open() {
        let (id_a, rec_a) = pair();
        let (id_b, rec_b) = pair();
        let data = seal(Kind::Ipn, "1234567890", &[rec_a, rec_b]).unwrap();
        assert_eq!(unseal(&data, &id_a).unwrap().value, "1234567890");
        assert_eq!(unseal(&data, &id_b).unwrap().value, "1234567890");
    }

    #[test]
    fn a_stranger_cannot_open() {
        let (_, recipient) = pair();
        let (stranger, _) = pair();
        let data = seal(Kind::Ipn, "1234567890", &[recipient]).unwrap();
        assert!(matches!(
            unseal(&data, &stranger),
            Err(SealError::Crypto(_))
        ));
    }

    #[test]
    fn sealed_bytes_do_not_contain_the_value() {
        let (_, recipient) = pair();
        let value = "1234567890";
        let data = seal(Kind::Ipn, value, &[recipient]).unwrap();
        assert!(!data
            .windows(value.len())
            .any(|window| window == value.as_bytes()));
    }

    #[test]
    fn damaged_data_is_rejected() {
        let (identity, recipient) = pair();
        let mut data = seal(Kind::Name, "Тест", &[recipient]).unwrap();
        let last = data.len() - 1;
        data[last] ^= 0xff;
        assert!(unseal(&data, &identity).is_err());
    }

    #[test]
    fn sealing_needs_a_recipient() {
        assert_eq!(seal(Kind::Name, "Тест", &[]), Err(SealError::NoRecipients));
    }

    #[test]
    fn keys_parse_from_text() {
        let (identity, recipient) = pair();
        use age::secrecy::ExposeSecret;
        let secret = identity.to_string();
        let parsed = parse_identity(secret.expose_secret()).unwrap();
        let rec = parse_recipient(&recipient.to_string()).unwrap();
        let data = seal(Kind::Text, "x", &[rec]).unwrap();
        assert_eq!(unseal(&data, &parsed).unwrap().value, "x");
        assert!(parse_recipient("not a key").is_err());
    }
}
