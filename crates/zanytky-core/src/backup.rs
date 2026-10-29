//! Encrypted backup of the home folder (system key, investigator list, databases).
//!
//! The backup is one file: a simple archive of the folder's files, encrypted
//! with a passphrase (age, scrypt) through [`crate::wrap`]. The passphrase is
//! never stored. Restoring writes only into an empty or missing folder, so it
//! can never overwrite existing data.
//!
//! Close every program that uses the folder before making a backup: the
//! databases are copied as files.

use std::fmt;
use std::fs;
use std::path::{Component, Path, PathBuf};

use crate::wrap::{unwrap, wrap};

const MAGIC: &[u8] = b"ZNBK1\n";
const REQUIRED: &str = "system.key";

#[derive(Debug)]
pub enum BackupError {
    Io(String),
    Crypto(String),
    Format(String),
    /// The restore folder already holds files.
    NotEmpty(PathBuf),
}

impl fmt::Display for BackupError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BackupError::Io(m) => write!(f, "помилка файлу: {m}"),
            BackupError::Crypto(m) => {
                write!(f, "невірна парольна фраза чи пошкоджений файл: {m}")
            }
            BackupError::Format(m) => write!(f, "некоректна резервна копія: {m}"),
            BackupError::NotEmpty(p) => {
                write!(f, "папка не порожня, відновлення скасовано: {}", p.display())
            }
        }
    }
}

impl std::error::Error for BackupError {}

fn io<E: fmt::Display>(error: E) -> BackupError {
    BackupError::Io(error.to_string())
}

fn skip(name: &str) -> bool {
    name.ends_with("-journal") || name.ends_with("-wal") || name.ends_with("-shm")
}

fn collect(root: &Path, dir: &Path, out: &mut Vec<(String, PathBuf)>) -> Result<(), BackupError> {
    for entry in fs::read_dir(dir).map_err(io)? {
        let entry = entry.map_err(io)?;
        let path = entry.path();
        let file_type = entry.file_type().map_err(io)?;
        if file_type.is_dir() {
            collect(root, &path, out)?;
        } else if file_type.is_file() {
            let name = entry.file_name().to_string_lossy().into_owned();
            if skip(&name) {
                continue;
            }
            let relative = path.strip_prefix(root).map_err(io)?;
            let parts: Vec<String> = relative
                .components()
                .map(|c| c.as_os_str().to_string_lossy().into_owned())
                .collect();
            out.push((parts.join("/"), path));
        }
    }
    Ok(())
}

/// Pack every file of `home` and encrypt the archive with `passphrase`.
pub fn backup_home(home: &Path, passphrase: &str) -> Result<Vec<u8>, BackupError> {
    let mut files = Vec::new();
    collect(home, home, &mut files)?;
    files.sort();
    if !files.iter().any(|(name, _)| name == REQUIRED) {
        return Err(BackupError::Format(format!("у папці немає {REQUIRED}")));
    }
    let mut archive = Vec::from(MAGIC);
    for (name, path) in &files {
        let data = fs::read(path).map_err(io)?;
        let name_bytes = name.as_bytes();
        archive.extend_from_slice(&(name_bytes.len() as u32).to_le_bytes());
        archive.extend_from_slice(name_bytes);
        archive.extend_from_slice(&(data.len() as u64).to_le_bytes());
        archive.extend_from_slice(&data);
    }
    wrap(&archive, passphrase).map_err(|e| BackupError::Crypto(e.to_string()))
}

fn take<'a>(data: &'a [u8], pos: &mut usize, len: usize) -> Result<&'a [u8], BackupError> {
    let end = pos
        .checked_add(len)
        .filter(|end| *end <= data.len())
        .ok_or_else(|| BackupError::Format("архів обірвано".into()))?;
    let slice = &data[*pos..end];
    *pos = end;
    Ok(slice)
}

fn read_u32(data: &[u8], pos: &mut usize) -> Result<u32, BackupError> {
    let bytes: [u8; 4] = take(data, pos, 4)?.try_into().unwrap();
    Ok(u32::from_le_bytes(bytes))
}

fn read_u64(data: &[u8], pos: &mut usize) -> Result<u64, BackupError> {
    let bytes: [u8; 8] = take(data, pos, 8)?.try_into().unwrap();
    Ok(u64::from_le_bytes(bytes))
}

fn safe_relative(name: &str) -> Result<PathBuf, BackupError> {
    if name.is_empty() {
        return Err(BackupError::Format("порожнє ім'я файлу".into()));
    }
    let mut out = PathBuf::new();
    for part in name.split('/') {
        let component = Path::new(part).components().collect::<Vec<_>>();
        match component.as_slice() {
            [Component::Normal(p)] => out.push(p),
            _ => return Err(BackupError::Format(format!("небезпечний шлях у копії: {name}"))),
        }
    }
    Ok(out)
}

/// Decrypt a backup and write its files into `dest` (must be empty or missing).
/// Returns the number of restored files.
pub fn restore_home(data: &[u8], passphrase: &str, dest: &Path) -> Result<usize, BackupError> {
    let archive = unwrap(data, passphrase).map_err(|e| BackupError::Crypto(e.to_string()))?;
    if !archive.starts_with(MAGIC) {
        return Err(BackupError::Format("це не резервна копія ZaNytky".into()));
    }
    // parse and validate everything before touching the disk
    let mut pos = MAGIC.len();
    let mut entries: Vec<(PathBuf, &[u8])> = Vec::new();
    while pos < archive.len() {
        let name_len = read_u32(&archive, &mut pos)? as usize;
        let name = take(&archive, &mut pos, name_len)?;
        let name = std::str::from_utf8(name)
            .map_err(|_| BackupError::Format("ім'я файлу не UTF-8".into()))?;
        let relative = safe_relative(name)?;
        let data_len = read_u64(&archive, &mut pos)?;
        let data_len = usize::try_from(data_len)
            .map_err(|_| BackupError::Format("завеликий файл".into()))?;
        entries.push((relative, take(&archive, &mut pos, data_len)?));
    }
    if !entries.iter().any(|(p, _)| p == Path::new(REQUIRED)) {
        return Err(BackupError::Format(format!("у копії немає {REQUIRED}")));
    }
    if dest.exists() {
        let mut listing = fs::read_dir(dest).map_err(io)?;
        if listing.next().is_some() {
            return Err(BackupError::NotEmpty(dest.to_path_buf()));
        }
    }
    for (relative, bytes) in &entries {
        let target = dest.join(relative);
        if let Some(parent) = target.parent() {
            fs::create_dir_all(parent).map_err(io)?;
        }
        fs::write(&target, bytes).map_err(io)?;
    }
    Ok(entries.len())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_home(dir: &Path) -> PathBuf {
        let home = dir.join("home");
        fs::create_dir_all(home.join("pkg")).unwrap();
        fs::write(home.join("system.key"), b"00112233445566778899aabbccddeeff").unwrap();
        fs::write(home.join("recipients.txt"), "age1publickey\n").unwrap();
        fs::write(home.join("model.db"), [1u8, 2, 3, 0, 255]).unwrap();
        fs::write(home.join("model.db-journal"), b"temporary").unwrap();
        fs::write(home.join("pkg").join("note.txt"), "вкладений файл").unwrap();
        home
    }

    #[test]
    fn round_trip_restores_every_file() {
        let dir = tempfile::tempdir().unwrap();
        let home = sample_home(dir.path());
        let backup = backup_home(&home, "довга парольна фраза").unwrap();
        let target = dir.path().join("restored");
        let count = restore_home(&backup, "довга парольна фраза", &target).unwrap();
        assert_eq!(count, 4);
        assert_eq!(fs::read(target.join("model.db")).unwrap(), vec![1u8, 2, 3, 0, 255]);
        assert_eq!(fs::read_to_string(target.join("pkg").join("note.txt")).unwrap(), "вкладений файл");
        assert!(!target.join("model.db-journal").exists());
    }

    #[test]
    fn backup_does_not_show_the_content() {
        let dir = tempfile::tempdir().unwrap();
        let home = sample_home(dir.path());
        let backup = backup_home(&home, "pass").unwrap();
        let needle = b"00112233445566778899aabbccddeeff";
        assert!(!backup.windows(needle.len()).any(|w| w == needle));
    }

    #[test]
    fn wrong_passphrase_is_refused() {
        let dir = tempfile::tempdir().unwrap();
        let home = sample_home(dir.path());
        let backup = backup_home(&home, "one").unwrap();
        let err = restore_home(&backup, "two", &dir.path().join("out")).unwrap_err();
        assert!(matches!(err, BackupError::Crypto(_)));
        assert!(!dir.path().join("out").exists());
    }

    #[test]
    fn restore_never_overwrites_a_folder_with_files() {
        let dir = tempfile::tempdir().unwrap();
        let home = sample_home(dir.path());
        let backup = backup_home(&home, "pass").unwrap();
        let err = restore_home(&backup, "pass", &home).unwrap_err();
        assert!(matches!(err, BackupError::NotEmpty(_)));
        assert_eq!(fs::read(home.join("model.db")).unwrap(), vec![1u8, 2, 3, 0, 255]);
    }

    #[test]
    fn a_folder_without_the_system_key_is_not_backed_up() {
        let dir = tempfile::tempdir().unwrap();
        let home = dir.path().join("empty-home");
        fs::create_dir_all(&home).unwrap();
        fs::write(home.join("model.db"), b"x").unwrap();
        assert!(matches!(backup_home(&home, "pass"), Err(BackupError::Format(_))));
    }

    #[test]
    fn paths_that_leave_the_folder_are_rejected() {
        for bad in ["../evil.txt", "/abs.txt", "a/../../b", "a//b", ""] {
            let mut archive = Vec::from(MAGIC);
            for (name, data) in [(REQUIRED, &b"k"[..]), (bad, &b"x"[..])] {
                archive.extend_from_slice(&(name.len() as u32).to_le_bytes());
                archive.extend_from_slice(name.as_bytes());
                archive.extend_from_slice(&(data.len() as u64).to_le_bytes());
                archive.extend_from_slice(data);
            }
            let wrapped = wrap(&archive, "pass").unwrap();
            let dir = tempfile::tempdir().unwrap();
            let target = dir.path().join("out");
            assert!(
                matches!(restore_home(&wrapped, "pass", &target), Err(BackupError::Format(_))),
                "{bad:?} must be rejected"
            );
            assert!(!target.exists(), "nothing may be written for {bad:?}");
        }
    }

    #[test]
    fn truncated_archive_is_rejected() {
        let mut archive = Vec::from(MAGIC);
        archive.extend_from_slice(&10u32.to_le_bytes());
        archive.extend_from_slice(b"short");
        let wrapped = wrap(&archive, "pass").unwrap();
        let dir = tempfile::tempdir().unwrap();
        assert!(matches!(
            restore_home(&wrapped, "pass", &dir.path().join("out")),
            Err(BackupError::Format(_))
        ));
    }

    #[test]
    fn a_foreign_encrypted_file_is_not_accepted() {
        let wrapped = wrap(b"not a backup", "pass").unwrap();
        let dir = tempfile::tempdir().unwrap();
        assert!(matches!(
            restore_home(&wrapped, "pass", &dir.path().join("out")),
            Err(BackupError::Format(_))
        ));
    }
}
