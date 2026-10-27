//! Data model (ADR-0002, ADR-0006): entities, sources, statements, grades.
//!
//! A statement is the main unit: it has several parties with roles and at
//! least one source. Sources form chains ("received from") that end in a root
//! source; independence is counted by roots, not by number of echoes.
//! Persons are stored only as pseudonyms; nothing is ever deleted.

use std::collections::HashSet;
use std::fmt;
use std::path::Path;

use rusqlite::{params, Connection, OptionalExtension};

const SCHEMA: &str = "
CREATE TABLE IF NOT EXISTS schema_version (version INTEGER NOT NULL);
CREATE TABLE IF NOT EXISTS entity (
    id INTEGER PRIMARY KEY,
    kind TEXT NOT NULL,
    token TEXT UNIQUE,
    label TEXT,
    public_code TEXT,
    created_at TEXT NOT NULL
);
CREATE TABLE IF NOT EXISTS source (
    id INTEGER PRIMARY KEY,
    kind TEXT NOT NULL,
    title TEXT NOT NULL,
    url TEXT,
    published_at TEXT,
    reliability TEXT NOT NULL,
    parent_id INTEGER,
    root_id INTEGER NOT NULL,
    recorded_at TEXT NOT NULL
);
CREATE TABLE IF NOT EXISTS statement (
    id INTEGER PRIMARY KEY,
    kind TEXT NOT NULL,
    value TEXT,
    valid_from TEXT,
    valid_to TEXT,
    as_of TEXT,
    status TEXT NOT NULL,
    credibility INTEGER NOT NULL CHECK (credibility BETWEEN 1 AND 6),
    note TEXT,
    recorded_at TEXT NOT NULL
);
CREATE TABLE IF NOT EXISTS statement_party (
    statement_id INTEGER NOT NULL,
    entity_id INTEGER NOT NULL,
    role TEXT NOT NULL,
    PRIMARY KEY (statement_id, entity_id, role)
);
CREATE TABLE IF NOT EXISTS statement_source (
    statement_id INTEGER NOT NULL,
    source_id INTEGER NOT NULL,
    PRIMARY KEY (statement_id, source_id)
);
CREATE TABLE IF NOT EXISTS change_log (
    id INTEGER PRIMARY KEY,
    at TEXT NOT NULL,
    actor TEXT NOT NULL,
    action TEXT NOT NULL,
    ref TEXT NOT NULL
);
CREATE INDEX IF NOT EXISTS idx_entity_token ON entity(token);
CREATE INDEX IF NOT EXISTS idx_party_entity ON statement_party(entity_id, role);
CREATE INDEX IF NOT EXISTS idx_party_statement ON statement_party(statement_id);
CREATE INDEX IF NOT EXISTS idx_statement_kind ON statement(kind);
CREATE INDEX IF NOT EXISTS idx_statement_valid ON statement(valid_from, valid_to);
CREATE INDEX IF NOT EXISTS idx_source_root ON source(root_id);
CREATE INDEX IF NOT EXISTS idx_statement_source ON statement_source(source_id);
";

const SCHEMA_VERSION: i64 = 1;
const NOW: &str = "strftime('%Y-%m-%dT%H:%M:%SZ', 'now')";

macro_rules! text_enum {
    ($(#[$meta:meta])* $name:ident { $($variant:ident => $text:literal),+ $(,)? }) => {
        $(#[$meta])*
        #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
        pub enum $name {
            $($variant),+
        }

        impl $name {
            pub fn as_str(self) -> &'static str {
                match self {
                    $(Self::$variant => $text),+
                }
            }

            pub fn parse(text: &str) -> Option<Self> {
                match text {
                    $($text => Some(Self::$variant),)+
                    _ => None,
                }
            }
        }
    };
}

text_enum!(
    /// What an entity is.
    EntityKind {
        Person => "person",
        Org => "org",
        Group => "group",
        Asset => "asset",
        Procurement => "procurement",
        Market => "market",
        Event => "event",
        Document => "document",
    }
);

text_enum!(
    /// Class of a source.
    SourceKind {
        Registry => "registry",
        Authority => "authority",
        Media => "media",
        Testimony => "testimony",
        Rumor => "rumor",
        Own => "own",
    }
);

text_enum!(
    /// What a statement says.
    StatementType {
        Owns => "owns",
        Beneficiary => "beneficiary",
        Director => "director",
        Founder => "founder",
        Won => "won",
        Customer => "customer",
        Supplies => "supplies",
        Uses => "uses",
        Coordinated => "coordinated",
        MemberOfGroup => "member_of_group",
        OperatesInMarket => "operates_in_market",
        ReportedLink => "reported_link",
    }
);

text_enum!(
    /// Verification status of a statement.
    Status {
        Unverified => "unverified",
        Confirmed => "confirmed",
        Refuted => "refuted",
    }
);

text_enum!(
    /// Reliability of a source, from best (A) to unknown (F).
    Reliability {
        A => "A",
        B => "B",
        C => "C",
        D => "D",
        E => "E",
        F => "F",
    }
);

/// Grade of a statement: best source reliability + credibility (1 best .. 6).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Grade {
    pub reliability: Reliability,
    pub credibility: u8,
}

impl fmt::Display for Grade {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}{}", self.reliability.as_str(), self.credibility)
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum ModelError {
    Db(String),
    /// The input breaks a rule of the model.
    Invalid(&'static str),
    NotFound,
    /// A statement cannot be confirmed without a document-level source.
    ConfirmNeedsDocument,
}

impl fmt::Display for ModelError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ModelError::Db(message) => write!(f, "database error: {message}"),
            ModelError::Invalid(message) => write!(f, "invalid: {message}"),
            ModelError::NotFound => write!(f, "not found"),
            ModelError::ConfirmNeedsDocument => {
                write!(
                    f,
                    "cannot confirm: no source that is not a rumor or testimony"
                )
            }
        }
    }
}

impl std::error::Error for ModelError {}

impl From<rusqlite::Error> for ModelError {
    fn from(error: rusqlite::Error) -> Self {
        ModelError::Db(error.to_string())
    }
}

pub struct NewEntity<'a> {
    pub kind: EntityKind,
    pub token: Option<&'a str>,
    pub label: Option<&'a str>,
    pub public_code: Option<&'a str>,
}

pub struct NewSource<'a> {
    pub kind: SourceKind,
    pub title: &'a str,
    pub url: Option<&'a str>,
    pub published_at: Option<&'a str>,
    pub reliability: Reliability,
    /// The source this one was received from.
    pub parent_id: Option<i64>,
}

pub struct NewStatement<'a> {
    pub kind: StatementType,
    pub value: Option<&'a str>,
    pub valid_from: Option<&'a str>,
    pub valid_to: Option<&'a str>,
    pub as_of: Option<&'a str>,
    pub credibility: u8,
    pub note: Option<&'a str>,
    /// (entity id, role)
    pub parties: &'a [(i64, &'a str)],
    pub sources: &'a [i64],
}

/// One step up the ownership chain.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OwnershipEdge {
    pub statement_id: i64,
    pub owner: i64,
    pub owned: i64,
    pub value: Option<String>,
    pub depth: u32,
}

pub struct Model {
    conn: Connection,
    actor: String,
}

impl Model {
    fn with_connection(conn: Connection, actor: &str) -> Result<Self, ModelError> {
        conn.execute_batch(SCHEMA)?;
        let versions: i64 =
            conn.query_row("SELECT COUNT(*) FROM schema_version", [], |r| r.get(0))?;
        if versions == 0 {
            conn.execute(
                "INSERT INTO schema_version (version) VALUES (?1)",
                params![SCHEMA_VERSION],
            )?;
        }
        Ok(Self {
            conn,
            actor: actor.to_string(),
        })
    }

    pub fn open(path: &Path, actor: &str) -> Result<Self, ModelError> {
        Self::with_connection(Connection::open(path)?, actor)
    }

    pub fn open_in_memory(actor: &str) -> Result<Self, ModelError> {
        Self::with_connection(Connection::open_in_memory()?, actor)
    }

    fn log(&self, action: &str, reference: &str) -> Result<(), ModelError> {
        self.conn.execute(
            &format!("INSERT INTO change_log (at, actor, action, ref) VALUES ({NOW}, ?1, ?2, ?3)"),
            params![self.actor, action, reference],
        )?;
        Ok(())
    }

    fn exists(&self, table: &str, id: i64) -> Result<bool, ModelError> {
        let found: Option<i64> = self
            .conn
            .query_row(
                &format!("SELECT 1 FROM {table} WHERE id = ?1"),
                params![id],
                |r| r.get(0),
            )
            .optional()?;
        Ok(found.is_some())
    }

    pub fn add_entity(&self, new: &NewEntity) -> Result<i64, ModelError> {
        if new.kind == EntityKind::Person {
            if new.label.is_some() {
                return Err(ModelError::Invalid("a person has no label"));
            }
            if new.token.is_none() {
                return Err(ModelError::Invalid("a person needs a pseudonym token"));
            }
        } else if new.label.is_none() {
            return Err(ModelError::Invalid("a non-person entity needs a label"));
        }
        self.conn.execute(
            &format!(
                "INSERT INTO entity (kind, token, label, public_code, created_at) \
                 VALUES (?1, ?2, ?3, ?4, {NOW})"
            ),
            params![new.kind.as_str(), new.token, new.label, new.public_code],
        )?;
        let id = self.conn.last_insert_rowid();
        self.log("add_entity", &format!("entity:{id}"))?;
        Ok(id)
    }

    fn root_of(&self, source_id: i64) -> Result<i64, ModelError> {
        self.conn
            .query_row(
                "SELECT root_id FROM source WHERE id = ?1",
                params![source_id],
                |r| r.get(0),
            )
            .optional()?
            .ok_or(ModelError::NotFound)
    }

    /// Add a source; its root is the root of the parent, or itself.
    pub fn add_source(&self, new: &NewSource) -> Result<i64, ModelError> {
        if new.title.trim().is_empty() {
            return Err(ModelError::Invalid("a source needs a title"));
        }
        let root = match new.parent_id {
            Some(parent) => Some(self.root_of(parent)?),
            None => None,
        };
        self.conn.execute(
            &format!(
                "INSERT INTO source (kind, title, url, published_at, reliability, \
                 parent_id, root_id, recorded_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, {NOW})"
            ),
            params![
                new.kind.as_str(),
                new.title,
                new.url,
                new.published_at,
                new.reliability.as_str(),
                new.parent_id,
                root.unwrap_or(0)
            ],
        )?;
        let id = self.conn.last_insert_rowid();
        if root.is_none() {
            self.conn
                .execute("UPDATE source SET root_id = id WHERE id = ?1", params![id])?;
        }
        self.log("add_source", &format!("source:{id}"))?;
        Ok(id)
    }

    /// Add an unverified statement with its parties and sources.
    pub fn add_statement(&self, new: &NewStatement) -> Result<i64, ModelError> {
        if !(1..=6).contains(&new.credibility) {
            return Err(ModelError::Invalid("credibility must be 1..6"));
        }
        if new.parties.is_empty() {
            return Err(ModelError::Invalid("a statement needs parties"));
        }
        if new.sources.is_empty() {
            return Err(ModelError::Invalid("a statement needs a source"));
        }
        for (entity, _) in new.parties {
            if !self.exists("entity", *entity)? {
                return Err(ModelError::NotFound);
            }
        }
        for source in new.sources {
            if !self.exists("source", *source)? {
                return Err(ModelError::NotFound);
            }
        }
        let tx = self.conn.unchecked_transaction()?;
        tx.execute(
            &format!(
                "INSERT INTO statement (kind, value, valid_from, valid_to, as_of, status, \
                 credibility, note, recorded_at) VALUES (?1, ?2, ?3, ?4, ?5, 'unverified', \
                 ?6, ?7, {NOW})"
            ),
            params![
                new.kind.as_str(),
                new.value,
                new.valid_from,
                new.valid_to,
                new.as_of,
                new.credibility,
                new.note
            ],
        )?;
        let id = tx.last_insert_rowid();
        for (entity, role) in new.parties {
            tx.execute(
                "INSERT OR IGNORE INTO statement_party (statement_id, entity_id, role) \
                 VALUES (?1, ?2, ?3)",
                params![id, entity, role],
            )?;
        }
        for source in new.sources {
            tx.execute(
                "INSERT OR IGNORE INTO statement_source (statement_id, source_id) \
                 VALUES (?1, ?2)",
                params![id, source],
            )?;
        }
        tx.commit()?;
        self.log("add_statement", &format!("statement:{id}"))?;
        Ok(id)
    }

    /// Change the status. A rumor or testimony alone can never confirm.
    pub fn set_status(&self, statement_id: i64, status: Status) -> Result<(), ModelError> {
        if !self.exists("statement", statement_id)? {
            return Err(ModelError::NotFound);
        }
        if status == Status::Confirmed {
            let documents: i64 = self.conn.query_row(
                "SELECT COUNT(*) FROM statement_source ss \
                 JOIN source s ON s.id = ss.source_id \
                 JOIN source r ON r.id = s.root_id \
                 WHERE ss.statement_id = ?1 AND r.kind NOT IN ('rumor', 'testimony')",
                params![statement_id],
                |r| r.get(0),
            )?;
            if documents == 0 {
                return Err(ModelError::ConfirmNeedsDocument);
            }
        }
        self.conn.execute(
            "UPDATE statement SET status = ?1 WHERE id = ?2",
            params![status.as_str(), statement_id],
        )?;
        self.log(
            &format!("set_status:{}", status.as_str()),
            &format!("statement:{statement_id}"),
        )?;
        Ok(())
    }

    pub fn status(&self, statement_id: i64) -> Result<Status, ModelError> {
        let text: String = self
            .conn
            .query_row(
                "SELECT status FROM statement WHERE id = ?1",
                params![statement_id],
                |r| r.get(0),
            )
            .optional()?
            .ok_or(ModelError::NotFound)?;
        Status::parse(&text).ok_or(ModelError::Invalid("unknown status in the database"))
    }

    /// Best source reliability plus the statement's credibility, e.g. `C3`.
    pub fn grade(&self, statement_id: i64) -> Result<Grade, ModelError> {
        let (reliability, credibility): (String, i64) = self
            .conn
            .query_row(
                "SELECT MIN(s.reliability), st.credibility FROM statement st \
                 JOIN statement_source ss ON ss.statement_id = st.id \
                 JOIN source s ON s.id = ss.source_id \
                 WHERE st.id = ?1 GROUP BY st.id",
                params![statement_id],
                |r| Ok((r.get(0)?, r.get(1)?)),
            )
            .optional()?
            .ok_or(ModelError::NotFound)?;
        Ok(Grade {
            reliability: Reliability::parse(&reliability)
                .ok_or(ModelError::Invalid("unknown reliability in the database"))?,
            credibility: u8::try_from(credibility)
                .map_err(|_| ModelError::Invalid("bad credibility in the database"))?,
        })
    }

    /// Number of distinct root sources behind a statement.
    pub fn independent_origins(&self, statement_id: i64) -> Result<u32, ModelError> {
        let count: i64 = self.conn.query_row(
            "SELECT COUNT(DISTINCT s.root_id) FROM statement_source ss \
             JOIN source s ON s.id = ss.source_id WHERE ss.statement_id = ?1",
            params![statement_id],
            |r| r.get(0),
        )?;
        Ok(u32::try_from(count).unwrap_or(0))
    }

    /// The chain "received from": the source, its parent, ..., the root.
    pub fn provenance(&self, source_id: i64) -> Result<Vec<i64>, ModelError> {
        let mut chain = vec![source_id];
        let mut current = source_id;
        loop {
            let parent: Option<i64> = self
                .conn
                .query_row(
                    "SELECT parent_id FROM source WHERE id = ?1",
                    params![current],
                    |r| r.get(0),
                )
                .optional()?
                .ok_or(ModelError::NotFound)?;
            match parent {
                Some(next) if !chain.contains(&next) => {
                    chain.push(next);
                    current = next;
                }
                _ => return Ok(chain),
            }
        }
    }

    /// Owners above `start`, level by level; refuted statements are skipped and
    /// cycles stop the walk.
    pub fn ownership_up(
        &self,
        start: i64,
        max_depth: u32,
    ) -> Result<Vec<OwnershipEdge>, ModelError> {
        let mut edges = Vec::new();
        let mut seen = HashSet::from([start]);
        let mut frontier = vec![start];
        for depth in 1..=max_depth {
            let mut next = Vec::new();
            for owned in frontier {
                let mut stmt = self.conn.prepare(
                    "SELECT st.id, st.value, owner.entity_id FROM statement st \
                     JOIN statement_party owner ON owner.statement_id = st.id AND owner.role = 'owner' \
                     JOIN statement_party owned ON owned.statement_id = st.id AND owned.role = 'owned' \
                     WHERE st.kind = 'owns' AND st.status != 'refuted' AND owned.entity_id = ?1 \
                     ORDER BY st.id",
                )?;
                let rows = stmt.query_map(params![owned], |r| {
                    Ok((
                        r.get::<_, i64>(0)?,
                        r.get::<_, Option<String>>(1)?,
                        r.get::<_, i64>(2)?,
                    ))
                })?;
                for row in rows {
                    let (statement_id, value, owner) = row?;
                    edges.push(OwnershipEdge {
                        statement_id,
                        owner,
                        owned,
                        value,
                        depth,
                    });
                    if seen.insert(owner) {
                        next.push(owner);
                    }
                }
            }
            if next.is_empty() {
                break;
            }
            frontier = next;
        }
        Ok(edges)
    }

    /// Number of entries in the change log.
    pub fn log_len(&self) -> Result<u64, ModelError> {
        let count: i64 = self
            .conn
            .query_row("SELECT COUNT(*) FROM change_log", [], |r| r.get(0))?;
        Ok(u64::try_from(count).unwrap_or(0))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn model() -> Model {
        Model::open_in_memory("tester").unwrap()
    }

    fn org(m: &Model, label: &str) -> i64 {
        m.add_entity(&NewEntity {
            kind: EntityKind::Org,
            token: None,
            label: Some(label),
            public_code: None,
        })
        .unwrap()
    }

    fn source(m: &Model, kind: SourceKind, reliability: Reliability, parent: Option<i64>) -> i64 {
        m.add_source(&NewSource {
            kind,
            title: "джерело",
            url: None,
            published_at: None,
            reliability,
            parent_id: parent,
        })
        .unwrap()
    }

    fn owns(m: &Model, owner: i64, owned: i64, share: &str, src: i64) -> i64 {
        m.add_statement(&NewStatement {
            kind: StatementType::Owns,
            value: Some(share),
            valid_from: None,
            valid_to: None,
            as_of: None,
            credibility: 2,
            note: None,
            parties: &[(owner, "owner"), (owned, "owned")],
            sources: &[src],
        })
        .unwrap()
    }

    #[test]
    fn a_person_is_a_token_without_a_label() {
        let m = model();
        let bad_label = m.add_entity(&NewEntity {
            kind: EntityKind::Person,
            token: Some("N-ABC"),
            label: Some("Іваненко"),
            public_code: None,
        });
        assert!(matches!(bad_label, Err(ModelError::Invalid(_))));
        let no_token = m.add_entity(&NewEntity {
            kind: EntityKind::Person,
            token: None,
            label: None,
            public_code: None,
        });
        assert!(matches!(no_token, Err(ModelError::Invalid(_))));
        let ok = m.add_entity(&NewEntity {
            kind: EntityKind::Person,
            token: Some("N-ABC"),
            label: None,
            public_code: None,
        });
        assert!(ok.is_ok());
    }

    #[test]
    fn an_organisation_needs_a_label() {
        let m = model();
        let result = m.add_entity(&NewEntity {
            kind: EntityKind::Org,
            token: None,
            label: None,
            public_code: Some("12345678"),
        });
        assert!(matches!(result, Err(ModelError::Invalid(_))));
    }

    #[test]
    fn echoes_of_one_root_count_once() {
        let m = model();
        let a = org(&m, "А");
        let b = org(&m, "Б");
        let root = source(&m, SourceKind::Rumor, Reliability::F, None);
        let echo1 = source(&m, SourceKind::Media, Reliability::C, Some(root));
        let echo2 = source(&m, SourceKind::Media, Reliability::C, Some(echo1));
        let other = source(&m, SourceKind::Registry, Reliability::A, None);
        let st = m
            .add_statement(&NewStatement {
                kind: StatementType::Owns,
                value: None,
                valid_from: None,
                valid_to: None,
                as_of: None,
                credibility: 3,
                note: None,
                parties: &[(a, "owner"), (b, "owned")],
                sources: &[echo1, echo2],
            })
            .unwrap();
        assert_eq!(m.independent_origins(st).unwrap(), 1);
        let st2 = m
            .add_statement(&NewStatement {
                kind: StatementType::Owns,
                value: None,
                valid_from: None,
                valid_to: None,
                as_of: None,
                credibility: 3,
                note: None,
                parties: &[(a, "owner"), (b, "owned")],
                sources: &[echo2, other],
            })
            .unwrap();
        assert_eq!(m.independent_origins(st2).unwrap(), 2);
        assert_eq!(m.provenance(echo2).unwrap(), vec![echo2, echo1, root]);
    }

    #[test]
    fn a_rumor_alone_cannot_confirm() {
        let m = model();
        let a = org(&m, "А");
        let b = org(&m, "Б");
        let rumor = source(&m, SourceKind::Rumor, Reliability::F, None);
        let echo = source(&m, SourceKind::Media, Reliability::C, Some(rumor));
        let st = owns(&m, a, b, "40%", echo);
        assert_eq!(
            m.set_status(st, Status::Confirmed),
            Err(ModelError::ConfirmNeedsDocument)
        );
        assert_eq!(m.status(st).unwrap(), Status::Unverified);
    }

    #[test]
    fn a_registry_root_can_confirm() {
        let m = model();
        let a = org(&m, "А");
        let b = org(&m, "Б");
        let registry = source(&m, SourceKind::Registry, Reliability::A, None);
        let media = source(&m, SourceKind::Media, Reliability::C, Some(registry));
        let st = owns(&m, a, b, "40%", media);
        m.set_status(st, Status::Confirmed).unwrap();
        assert_eq!(m.status(st).unwrap(), Status::Confirmed);
    }

    #[test]
    fn the_grade_uses_the_best_source() {
        let m = model();
        let a = org(&m, "А");
        let b = org(&m, "Б");
        let good = source(&m, SourceKind::Authority, Reliability::B, None);
        let weak = source(&m, SourceKind::Testimony, Reliability::D, None);
        let st = m
            .add_statement(&NewStatement {
                kind: StatementType::Owns,
                value: None,
                valid_from: None,
                valid_to: None,
                as_of: None,
                credibility: 3,
                note: None,
                parties: &[(a, "owner"), (b, "owned")],
                sources: &[weak, good],
            })
            .unwrap();
        assert_eq!(m.grade(st).unwrap().to_string(), "B3");
    }

    #[test]
    fn a_statement_needs_parties_sources_and_valid_credibility() {
        let m = model();
        let a = org(&m, "А");
        let s = source(&m, SourceKind::Registry, Reliability::A, None);
        let base = |parties: &'static [(i64, &'static str)], sources: &'static [i64], c: u8| {
            m.add_statement(&NewStatement {
                kind: StatementType::Uses,
                value: None,
                valid_from: None,
                valid_to: None,
                as_of: None,
                credibility: c,
                note: None,
                parties,
                sources,
            })
        };
        assert!(base(&[], &[1], 3).is_err());
        assert!(base(&[(1, "owner")], &[], 3).is_err());
        assert!(base(&[(1, "owner")], &[1], 0).is_err());
        assert!(base(&[(1, "owner")], &[1], 7).is_err());
        assert!(base(&[(99, "owner")], &[1], 3).is_err());
        assert_eq!((a, s), (1, 1));
        assert!(base(&[(1, "owner")], &[1], 3).is_ok());
    }

    #[test]
    fn ownership_goes_up_and_stops_on_cycles() {
        let m = model();
        let company = org(&m, "Компанія");
        let holding = org(&m, "Холдинг");
        let top = org(&m, "Верхня");
        let src = source(&m, SourceKind::Registry, Reliability::A, None);
        owns(&m, holding, company, "100%", src);
        owns(&m, top, holding, "60%", src);
        owns(&m, company, top, "1%", src);
        let edges = m.ownership_up(company, 10).unwrap();
        let owners: Vec<i64> = edges.iter().map(|e| e.owner).collect();
        assert_eq!(owners, vec![holding, top, company]);
        assert_eq!(edges[0].depth, 1);
        assert_eq!(edges[1].depth, 2);
        assert_eq!(edges[1].value.as_deref(), Some("60%"));
    }

    #[test]
    fn refuted_statements_are_skipped_but_kept() {
        let m = model();
        let company = org(&m, "Компанія");
        let holding = org(&m, "Холдинг");
        let src = source(&m, SourceKind::Registry, Reliability::A, None);
        let st = owns(&m, holding, company, "100%", src);
        m.set_status(st, Status::Refuted).unwrap();
        assert!(m.ownership_up(company, 5).unwrap().is_empty());
        assert_eq!(m.status(st).unwrap(), Status::Refuted);
    }

    #[test]
    fn every_change_is_logged() {
        let m = model();
        let a = org(&m, "А");
        let b = org(&m, "Б");
        let src = source(&m, SourceKind::Registry, Reliability::A, None);
        let st = owns(&m, a, b, "10%", src);
        m.set_status(st, Status::Confirmed).unwrap();
        assert_eq!(m.log_len().unwrap(), 5);
    }
}
