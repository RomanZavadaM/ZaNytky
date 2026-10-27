//! Pseudonym registry in SQLite: token -> sealed original.
//!
//! The database holds only tokens, the kind, the normalisation version and the
//! sealed (encrypted) original. Plain values never reach the file.

use std::fmt;
use std::path::Path;

use rusqlite::{params, Connection, OptionalExtension};

use crate::normalize::{Kind, NORM_VERSION};
use crate::pseudonym::{token, TokenError, MIN_KEY_BYTES};
use crate::seal::{seal, unseal, Identity, Recipient, SealError, Sealed};

const SCHEMA: &str = "
CREATE TABLE IF NOT EXISTS pseudonym (
    token  TEXT PRIMARY KEY,
    kind   TEXT NOT NULL,
    norm   TEXT NOT NULL,
    sealed BLOB NOT NULL
)";

#[derive(Debug, PartialEq, Eq)]
pub enum RegistryError {
    Token(TokenError),
    Seal(SealError),
    Db(String),
    /// No record with this token.
    NotFound,
}

impl fmt::Display for RegistryError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RegistryError::Token(error) => write!(f, "{error}"),
            RegistryError::Seal(error) => write!(f, "{error}"),
            RegistryError::Db(message) => write!(f, "database error: {message}"),
            RegistryError::NotFound => write!(f, "no such token"),
        }
    }
}

impl std::error::Error for RegistryError {}

impl From<TokenError> for RegistryError {
    fn from(error: TokenError) -> Self {
        RegistryError::Token(error)
    }
}

impl From<SealError> for RegistryError {
    fn from(error: SealError) -> Self {
        RegistryError::Seal(error)
    }
}

impl From<rusqlite::Error> for RegistryError {
    fn from(error: rusqlite::Error) -> Self {
        RegistryError::Db(error.to_string())
    }
}

/// Registry of pseudonyms. Holds the system key (for tokens) and the
/// investigators' public keys (for sealing); it can never open a sealed value.
pub struct Registry {
    conn: Connection,
    key: Vec<u8>,
    recipients: Vec<Recipient>,
}

impl Registry {
    fn with_connection(
        conn: Connection,
        key: &[u8],
        recipients: Vec<Recipient>,
    ) -> Result<Self, RegistryError> {
        if key.len() < MIN_KEY_BYTES {
            return Err(TokenError::KeyTooShort.into());
        }
        if recipients.is_empty() {
            return Err(SealError::NoRecipients.into());
        }
        conn.execute_batch(SCHEMA)?;
        Ok(Self {
            conn,
            key: key.to_vec(),
            recipients,
        })
    }

    /// Open (or create) the registry file.
    pub fn open(
        path: &Path,
        key: &[u8],
        recipients: Vec<Recipient>,
    ) -> Result<Self, RegistryError> {
        Self::with_connection(Connection::open(path)?, key, recipients)
    }

    /// A registry that lives only in memory (tests, previews).
    pub fn open_in_memory(key: &[u8], recipients: Vec<Recipient>) -> Result<Self, RegistryError> {
        Self::with_connection(Connection::open_in_memory()?, key, recipients)
    }

    fn exists(&self, tok: &str) -> Result<bool, RegistryError> {
        let found: Option<i64> = self
            .conn
            .query_row(
                "SELECT 1 FROM pseudonym WHERE token = ?1",
                params![tok],
                |row| row.get(0),
            )
            .optional()?;
        Ok(found.is_some())
    }

    /// Token of the value; the original is sealed on first sight.
    pub fn register(&self, kind: Kind, value: &str) -> Result<String, RegistryError> {
        let tok = token(kind, value, &self.key)?;
        if !self.exists(&tok)? {
            let sealed = seal(kind, value, &self.recipients)?;
            self.conn.execute(
                "INSERT INTO pseudonym (token, kind, norm, sealed) VALUES (?1, ?2, ?3, ?4)",
                params![tok, kind.as_str(), NORM_VERSION, sealed],
            )?;
        }
        Ok(tok)
    }

    /// The query by a known value: its token if the value is already registered.
    pub fn find(&self, kind: Kind, value: &str) -> Result<Option<String>, RegistryError> {
        let tok = token(kind, value, &self.key)?;
        Ok(self.exists(&tok)?.then_some(tok))
    }

    /// Open the original. Needs the investigator's private identity.
    pub fn reveal(&self, tok: &str, identity: &Identity) -> Result<Sealed, RegistryError> {
        let sealed: Vec<u8> = self
            .conn
            .query_row(
                "SELECT sealed FROM pseudonym WHERE token = ?1",
                params![tok],
                |row| row.get(0),
            )
            .optional()?
            .ok_or(RegistryError::NotFound)?;
        Ok(unseal(&sealed, identity)?)
    }

    /// Number of registered values.
    pub fn count(&self) -> Result<u64, RegistryError> {
        let count: i64 = self
            .conn
            .query_row("SELECT COUNT(*) FROM pseudonym", [], |row| row.get(0))?;
        Ok(u64::try_from(count).unwrap_or(0))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn key() -> Vec<u8> {
        (0u8..32).collect()
    }

    fn investigator() -> (Identity, Recipient) {
        let identity = Identity::generate();
        let recipient = identity.to_public();
        (identity, recipient)
    }

    #[test]
    fn register_is_idempotent_across_spellings() {
        let (_, recipient) = investigator();
        let registry = Registry::open_in_memory(&key(), vec![recipient]).unwrap();
        let a = registry
            .register(Kind::Name, "  Іваненко   Петро-Олексійович ")
            .unwrap();
        let b = registry
            .register(Kind::Name, "іваненко петро \u{2013} олексійович")
            .unwrap();
        assert_eq!(a, b);
        assert_eq!(registry.count().unwrap(), 1);
    }

    #[test]
    fn find_returns_only_registered_values() {
        let (_, recipient) = investigator();
        let registry = Registry::open_in_memory(&key(), vec![recipient]).unwrap();
        assert_eq!(registry.find(Kind::Ipn, "1234567890").unwrap(), None);
        let tok = registry.register(Kind::Ipn, "1234567890").unwrap();
        assert_eq!(registry.find(Kind::Ipn, "1234-567-890").unwrap(), Some(tok));
    }

    #[test]
    fn investigator_reveals_the_original() {
        let (identity, recipient) = investigator();
        let registry = Registry::open_in_memory(&key(), vec![recipient]).unwrap();
        let tok = registry.register(Kind::Name, "Іваненко Петро").unwrap();
        let opened = registry.reveal(&tok, &identity).unwrap();
        assert_eq!(opened.kind, Kind::Name);
        assert_eq!(opened.value, "Іваненко Петро");
    }

    #[test]
    fn a_stranger_cannot_reveal() {
        let (_, recipient) = investigator();
        let (stranger, _) = investigator();
        let registry = Registry::open_in_memory(&key(), vec![recipient]).unwrap();
        let tok = registry.register(Kind::Ipn, "1234567890").unwrap();
        assert!(matches!(
            registry.reveal(&tok, &stranger),
            Err(RegistryError::Seal(_))
        ));
    }

    #[test]
    fn unknown_token_is_not_found() {
        let (identity, recipient) = investigator();
        let registry = Registry::open_in_memory(&key(), vec![recipient]).unwrap();
        assert_eq!(
            registry.reveal("I-UNKNOWN", &identity),
            Err(RegistryError::NotFound)
        );
    }

    #[test]
    fn short_key_and_missing_recipient_are_rejected() {
        let (_, recipient) = investigator();
        assert!(matches!(
            Registry::open_in_memory(&[1u8; 8], vec![recipient]),
            Err(RegistryError::Token(TokenError::KeyTooShort))
        ));
        assert!(matches!(
            Registry::open_in_memory(&key(), vec![]),
            Err(RegistryError::Seal(SealError::NoRecipients))
        ));
    }

    #[test]
    fn the_database_file_has_no_plain_values() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("registry.db");
        let (identity, recipient) = investigator();
        let tok;
        {
            let registry = Registry::open(&path, &key(), vec![recipient.clone()]).unwrap();
            registry.register(Kind::Ipn, "1234567890").unwrap();
            tok = registry.register(Kind::Name, "Іваненко Петро").unwrap();
        }
        let bytes = std::fs::read(&path).unwrap();
        for needle in ["1234567890", "Іваненко", "Петро"] {
            assert!(
                !bytes
                    .windows(needle.len())
                    .any(|window| window == needle.as_bytes()),
                "plain value {needle} found in the database file"
            );
        }
        let reopened = Registry::open(&path, &key(), vec![recipient]).unwrap();
        assert_eq!(reopened.count().unwrap(), 2);
        assert_eq!(
            reopened.reveal(&tok, &identity).unwrap().value,
            "Іваненко Петро"
        );
    }
}
