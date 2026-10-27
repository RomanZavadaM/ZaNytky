//! Password wrapper for exported files (ADR-0005, option A).
//!
//! Any export is encrypted with the investigator's passphrase (age, scrypt).
//! The passphrase is never stored; what happens to the opened data afterwards
//! is the investigator's responsibility.

use std::io::{Read, Write};

use age::secrecy::SecretString;

use crate::seal::SealError;

fn crypto<E: std::fmt::Display>(error: E) -> SealError {
    SealError::Crypto(error.to_string())
}

fn secret(passphrase: &str) -> Result<SecretString, SealError> {
    if passphrase.is_empty() {
        return Err(SealError::Crypto("empty passphrase".into()));
    }
    Ok(SecretString::from(passphrase.to_owned()))
}

/// Encrypt `data` with the passphrase.
pub fn wrap(data: &[u8], passphrase: &str) -> Result<Vec<u8>, SealError> {
    let encryptor = age::Encryptor::with_user_passphrase(secret(passphrase)?);
    let mut out = Vec::new();
    let mut writer = encryptor.wrap_output(&mut out).map_err(crypto)?;
    writer.write_all(data).map_err(crypto)?;
    writer.finish().map_err(crypto)?;
    Ok(out)
}

/// Decrypt data produced by [`wrap`].
pub fn unwrap(data: &[u8], passphrase: &str) -> Result<Vec<u8>, SealError> {
    let decryptor = age::Decryptor::new(data).map_err(crypto)?;
    let identity = age::scrypt::Identity::new(secret(passphrase)?);
    let mut reader = decryptor
        .decrypt(std::iter::once(&identity as &dyn age::Identity))
        .map_err(crypto)?;
    let mut out = Vec::new();
    reader.read_to_end(&mut out).map_err(crypto)?;
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trip() {
        let data = "Звіт: Іваненко Петро".as_bytes();
        let wrapped = wrap(data, "довга парольна фраза").unwrap();
        assert_eq!(unwrap(&wrapped, "довга парольна фраза").unwrap(), data);
    }

    #[test]
    fn wrong_passphrase_fails() {
        let wrapped = wrap(b"secret", "one").unwrap();
        assert!(unwrap(&wrapped, "two").is_err());
    }

    #[test]
    fn wrapped_bytes_hide_the_content() {
        let wrapped = wrap(b"1234567890", "pass").unwrap();
        assert!(!wrapped.windows(10).any(|window| window == b"1234567890"));
    }

    #[test]
    fn empty_passphrase_is_rejected() {
        assert!(wrap(b"x", "").is_err());
        assert!(unwrap(b"x", "").is_err());
    }
}
