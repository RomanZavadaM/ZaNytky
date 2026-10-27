//! Investigator export (ADR-0005): revealed values as Markdown, wrapped with a passphrase.
//!
//! Exporting needs the investigator's private identity (to reveal) and a
//! passphrase (to protect the result). The output must be written outside the
//! repositories.

use crate::registry::{Registry, RegistryError};
use crate::seal::Identity;
use crate::wrap::wrap;

fn escape_cell(text: &str) -> String {
    text.replace('|', "\\|").replace(['\r', '\n'], " ")
}

/// Markdown table of every registered pseudonym with its revealed original.
pub fn markdown_table(registry: &Registry, identity: &Identity) -> Result<String, RegistryError> {
    let mut out = String::from(
        "# Розкриті значення\n\n\
         Документ містить відкриті персональні дані. Відповідальність за його зберігання \
         і використання несе розслідувач.\n\n\
         | Псевдонім | Вид | Значення |\n| --- | --- | --- |\n",
    );
    for tok in registry.tokens()? {
        let sealed = registry.reveal(&tok, identity)?;
        out.push_str(&format!(
            "| {} | {} | {} |\n",
            tok,
            sealed.kind.as_str(),
            escape_cell(&sealed.value)
        ));
    }
    Ok(out)
}

/// The Markdown table, encrypted with the investigator's passphrase.
pub fn markdown_table_protected(
    registry: &Registry,
    identity: &Identity,
    passphrase: &str,
) -> Result<Vec<u8>, RegistryError> {
    let markdown = markdown_table(registry, identity)?;
    Ok(wrap(markdown.as_bytes(), passphrase)?)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::normalize::Kind;
    use crate::wrap::unwrap;

    fn key() -> Vec<u8> {
        (0u8..32).collect()
    }

    fn setup() -> (Registry, Identity) {
        let identity = Identity::generate();
        let registry = Registry::open_in_memory(&key(), vec![identity.to_public()]).unwrap();
        (registry, identity)
    }

    #[test]
    fn table_lists_revealed_values() {
        let (registry, identity) = setup();
        let tok = registry.register(Kind::Name, "Іваненко Петро").unwrap();
        registry.register(Kind::Ipn, "1234567890").unwrap();
        let table = markdown_table(&registry, &identity).unwrap();
        assert!(table.contains(&tok));
        assert!(table.contains("| name | Іваненко Петро |"));
        assert!(table.contains("| ipn | 1234567890 |"));
    }

    #[test]
    fn pipes_and_newlines_do_not_break_the_table() {
        let (registry, identity) = setup();
        registry.register(Kind::Text, "а|б\nв").unwrap();
        let table = markdown_table(&registry, &identity).unwrap();
        assert!(table.contains("а\\|б в"));
    }

    #[test]
    fn protected_export_needs_the_passphrase() {
        let (registry, identity) = setup();
        registry.register(Kind::Ipn, "1234567890").unwrap();
        let protected = markdown_table_protected(&registry, &identity, "пароль").unwrap();
        assert!(!protected.windows(10).any(|w| w == b"1234567890"));
        let opened = unwrap(&protected, "пароль").unwrap();
        assert!(String::from_utf8(opened).unwrap().contains("1234567890"));
        assert!(unwrap(&protected, "інший").is_err());
    }

    #[test]
    fn export_needs_the_right_identity() {
        let (registry, _) = setup();
        registry.register(Kind::Ipn, "1234567890").unwrap();
        let stranger = Identity::generate();
        assert!(markdown_table(&registry, &stranger).is_err());
    }
}
