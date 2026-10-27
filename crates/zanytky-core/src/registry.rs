//! Pseudonym registry in SQLite: token -> sealed original.
//!
//! The database holds only tokens, the kind, the normalisation version and the
//! sealed (encrypted) original. Plain values never reach the file.

use std::collections::HashMap;
use std::fmt;
use std::path::Path;

use rusqlite::{params, Connection, OptionalExtension};

use crate::deidentify::DictEntry;
use crate::normalize::{Kind, NORM_VERSION};
use crate::pseudonym::{token, TokenError, MIN_KEY_BYTES};
use crate::seal::{seal, unseal, Identity, Recipient, SealError, Sealed};

const SCHEMA: &str = "
CREATE TABLE IF NOT EXISTS pseudonym (
    token  TEXT PRIMARY KEY,
    kind   TEXT NOT NULL,
    norm   TEXT NOT NULL,
    sealed BLOB NOT NULL
);
CREATE TABLE IF NOT EXISTS alias (
    alias_token  TEXT PRIMARY KEY,
    kind         TEXT NOT NULL,
    person_token TEXT NOT NULL,
    sealed       BLOB NOT NULL
)";

#[derive(Debug, PartialEq, Eq)]
pub enum RegistryError {
    Token(TokenError),
    Seal(SealError),
    Db(String),
    /// No record with this token.
    NotFound,
    /// The spelling or code is already linked to another person.
    Conflict,
}

impl fmt::Display for RegistryError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RegistryError::Token(error) => write!(f, "{error}"),
            RegistryError::Seal(error) => write!(f, "{error}"),
            RegistryError::Db(message) => write!(f, "database error: {message}"),
            RegistryError::NotFound => write!(f, "no such token"),
            RegistryError::Conflict => write!(f, "already linked to another person"),
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

/// What the investigator decided about a mention.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Resolution {
    /// Not a person (a company name, a place, ...).
    NotAPerson,
    /// The same person as an already registered one.
    SamePerson(String),
    /// A new person: full name and known codes (tax number, phone, e-mail).
    NewPerson {
        name: String,
        codes: Vec<(Kind, String)>,
    },
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

    /// Link a spelling or a code to a registered person. The text is sealed
    /// like any original; the returned token is the pseudonym of the spelling.
    pub fn link(
        &self,
        kind: Kind,
        value: &str,
        person_token: &str,
    ) -> Result<String, RegistryError> {
        if !self.exists(person_token)? {
            return Err(RegistryError::NotFound);
        }
        let alias = token(kind, value, &self.key)?;
        if alias == person_token {
            return Ok(alias);
        }
        let existing: Option<String> = self
            .conn
            .query_row(
                "SELECT person_token FROM alias WHERE alias_token = ?1",
                params![alias],
                |row| row.get(0),
            )
            .optional()?;
        match existing {
            Some(person) if person == person_token => Ok(alias),
            Some(_) => Err(RegistryError::Conflict),
            None => {
                let sealed = seal(kind, value, &self.recipients)?;
                self.conn.execute(
                    "INSERT INTO alias (alias_token, kind, person_token, sealed) \
                     VALUES (?1, ?2, ?3, ?4)",
                    params![alias, kind.as_str(), person_token, sealed],
                )?;
                Ok(alias)
            }
        }
    }

    /// The investigator's decision about a mention found in a document.
    pub fn resolve_mention(
        &self,
        mention: &str,
        resolution: &Resolution,
    ) -> Result<Option<String>, RegistryError> {
        match resolution {
            Resolution::NotAPerson => Ok(None),
            Resolution::SamePerson(person) => {
                self.link(Kind::Name, mention, person)?;
                Ok(Some(person.clone()))
            }
            Resolution::NewPerson { name, codes } => {
                let person = self.register(Kind::Name, name)?;
                self.link(Kind::Name, mention, &person)?;
                for (kind, value) in codes {
                    self.link(*kind, value, &person)?;
                }
                Ok(Some(person))
            }
        }
    }

    /// Code pseudonym -> person pseudonym, for de-identification.
    pub fn links(&self) -> Result<HashMap<String, String>, RegistryError> {
        let mut stmt = self
            .conn
            .prepare("SELECT alias_token, person_token FROM alias WHERE kind != 'name'")?;
        let rows = stmt.query_map([], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
        })?;
        Ok(rows.collect::<Result<HashMap<_, _>, _>>()?)
    }

    /// The investigator's dictionary of names and spellings. Opening it needs
    /// the private identity; it exists only for the time of the processing.
    pub fn dictionary(&self, identity: &Identity) -> Result<Vec<DictEntry>, RegistryError> {
        let mut out = Vec::new();
        let mut names = self
            .conn
            .prepare("SELECT token, sealed FROM pseudonym WHERE kind = 'name' ORDER BY token")?;
        let rows = names.query_map([], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, Vec<u8>>(1)?))
        })?;
        for row in rows {
            let (tok, sealed) = row?;
            let opened = unseal(&sealed, identity)?;
            out.push(DictEntry {
                text: opened.value,
                token: Some(tok),
            });
        }
        let mut aliases = self.conn.prepare(
            "SELECT person_token, sealed FROM alias WHERE kind = 'name' ORDER BY alias_token",
        )?;
        let rows = aliases.query_map([], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, Vec<u8>>(1)?))
        })?;
        for row in rows {
            let (person, sealed) = row?;
            let opened = unseal(&sealed, identity)?;
            out.push(DictEntry {
                text: opened.value,
                token: Some(person),
            });
        }
        Ok(out)
    }

    /// Every registered token, sorted.
    pub fn tokens(&self) -> Result<Vec<String>, RegistryError> {
        let mut stmt = self
            .conn
            .prepare("SELECT token FROM pseudonym ORDER BY token")?;
        let rows = stmt.query_map([], |row| row.get::<_, String>(0))?;
        Ok(rows.collect::<Result<Vec<_>, _>>()?)
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
    fn a_spelling_is_linked_to_a_person_and_enters_the_dictionary() {
        let (identity, recipient) = investigator();
        let registry = Registry::open_in_memory(&key(), vec![recipient]).unwrap();
        let person = registry.register(Kind::Name, "Іваненко Петро").unwrap();
        let alias = registry.link(Kind::Name, "Іванов І.", &person).unwrap();
        assert_ne!(alias, person);
        let dictionary = registry.dictionary(&identity).unwrap();
        assert!(dictionary.contains(&DictEntry::alias("Іванов І.", &person)));
        assert!(dictionary.contains(&DictEntry::alias("Іваненко Петро", &person)));
    }

    #[test]
    fn linking_is_idempotent_and_conflicts_are_refused() {
        let (_, recipient) = investigator();
        let registry = Registry::open_in_memory(&key(), vec![recipient]).unwrap();
        let a = registry.register(Kind::Name, "Іваненко Петро").unwrap();
        let b = registry.register(Kind::Name, "Коваль Олексій").unwrap();
        registry.link(Kind::Name, "Іванов І.", &a).unwrap();
        registry.link(Kind::Name, "Іванов І.", &a).unwrap();
        assert_eq!(
            registry.link(Kind::Name, "Іванов І.", &b),
            Err(RegistryError::Conflict)
        );
        assert_eq!(
            registry.link(Kind::Name, "Хтось", "N-UNKNOWN"),
            Err(RegistryError::NotFound)
        );
    }

    #[test]
    fn the_dictionary_needs_the_right_identity() {
        let (_, recipient) = investigator();
        let (stranger, _) = investigator();
        let registry = Registry::open_in_memory(&key(), vec![recipient]).unwrap();
        registry.register(Kind::Name, "Іваненко Петро").unwrap();
        assert!(registry.dictionary(&stranger).is_err());
    }

    #[test]
    fn a_new_person_with_codes_is_recognised_everywhere() {
        let (identity, recipient) = investigator();
        let registry = Registry::open_in_memory(&key(), vec![recipient]).unwrap();
        let person = registry
            .resolve_mention(
                "Іваненко П.",
                &Resolution::NewPerson {
                    name: "Іваненко Петро Олексійович".to_string(),
                    codes: vec![
                        (Kind::Ipn, "1234567890".to_string()),
                        (Kind::Phone, "0501234567".to_string()),
                    ],
                },
            )
            .unwrap()
            .unwrap();
        let dictionary = registry.dictionary(&identity).unwrap();
        let links = registry.links().unwrap();
        let text = "Іваненко П. (ІПН 1234567890, тел. +38 050 123 45 67) та Іваненко Петро Олексійович.";
        let result = crate::deidentify::deidentify_with_links(text, &key(), &dictionary, &links)
            .unwrap();
        assert_eq!(
            result.text,
            format!("{person} (ІПН {person}, тел. {person}) та {person}.")
        );
        assert!(result.replacements.iter().all(|r| r.token == person));
        assert_eq!(result.replacements.iter().map(|r| r.count).sum::<u32>(), 4);
    }

    #[test]
    fn a_mention_that_is_not_a_person_changes_nothing() {
        let (_, recipient) = investigator();
        let registry = Registry::open_in_memory(&key(), vec![recipient]).unwrap();
        assert_eq!(
            registry
                .resolve_mention("ТОВ Приклад", &Resolution::NotAPerson)
                .unwrap(),
            None
        );
        assert_eq!(registry.count().unwrap(), 0);
    }

    #[test]
    fn links_are_sealed_in_the_database_file() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("registry.db");
        let (_, recipient) = investigator();
        {
            let registry = Registry::open(&path, &key(), vec![recipient]).unwrap();
            let person = registry.register(Kind::Name, "Іваненко Петро").unwrap();
            registry.link(Kind::Name, "Іванов Прізвисько", &person).unwrap();
            registry.link(Kind::Ipn, "1234567890", &person).unwrap();
        }
        let bytes = std::fs::read(&path).unwrap();
        for needle in ["Прізвисько", "Іванов", "1234567890"] {
            assert!(
                !bytes
                    .windows(needle.len())
                    .any(|window| window == needle.as_bytes()),
                "{needle} found in the database file"
            );
        }
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
