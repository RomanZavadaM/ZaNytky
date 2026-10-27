use std::fs;
use std::path::Path;

use zanytky_cli::run;

fn cli(args: &[&str]) -> Result<String, String> {
    let args: Vec<String> = args.iter().map(|s| s.to_string()).collect();
    let mut out = Vec::new();
    run(&args, &mut out).map(|()| String::from_utf8(out).unwrap())
}

fn home_arg(dir: &Path) -> String {
    dir.join("home").display().to_string()
}

/// A home folder with the system key and one investigator.
fn setup(dir: &Path) -> (String, String) {
    let home = home_arg(dir);
    cli(&["init", "--home", home.as_str()]).unwrap();
    let identity = dir.join("investigator.txt").display().to_string();
    let output = cli(&["identity-new", identity.as_str()]).unwrap();
    let public = output.rsplit(": ").next().unwrap().trim().to_string();
    let recipients = Path::new(home.as_str()).join("recipients.txt");
    let mut text = fs::read_to_string(&recipients).unwrap();
    text.push_str(&format!("{public}\n"));
    fs::write(&recipients, text).unwrap();
    (home, identity)
}

#[test]
fn init_makes_a_key_and_refuses_to_overwrite_it() {
    let dir = tempfile::tempdir().unwrap();
    let home = home_arg(dir.path());
    cli(&["init", "--home", home.as_str()]).unwrap();
    let key = fs::read_to_string(Path::new(home.as_str()).join("system.key")).unwrap();
    assert_eq!(key.trim().len(), 64);
    assert!(cli(&["init", "--home", home.as_str()]).is_err());
}

#[test]
fn registering_needs_an_investigator_key() {
    let dir = tempfile::tempdir().unwrap();
    let home = home_arg(dir.path());
    cli(&["init", "--home", home.as_str()]).unwrap();
    let result = cli(&[
        "register",
        "name",
        "Іваненко Петро",
        "--home",
        home.as_str(),
    ]);
    assert!(result.unwrap_err().contains("recipients.txt"));
}

#[test]
fn register_and_find_give_the_same_token() {
    let dir = tempfile::tempdir().unwrap();
    let (home, _) = setup(dir.path());
    let token = cli(&[
        "register",
        "name",
        "Іваненко Петро",
        "--home",
        home.as_str(),
    ])
    .unwrap();
    let found = cli(&[
        "find",
        "name",
        "  іваненко   ПЕТРО ",
        "--home",
        home.as_str(),
    ])
    .unwrap();
    assert_eq!(token.trim(), found.trim());
    assert_eq!(
        cli(&["find", "name", "Невідомий", "--home", home.as_str()])
            .unwrap()
            .trim(),
        "немає"
    );
}

#[test]
fn the_whole_flow_from_text_to_report() {
    let dir = tempfile::tempdir().unwrap();
    let (home, identity) = setup(dir.path());
    let token = cli(&[
        "register",
        "name",
        "Іваненко Петро",
        "--home",
        home.as_str(),
    ])
    .unwrap()
    .trim()
    .to_string();

    // 1. prepare the package for the AI
    let input = dir.path().join("doc.md");
    fs::write(
        &input,
        "Іваненко Петро володіє 40% ТОВ Приклад (ІПН 1234567890).",
    )
    .unwrap();
    let pkg = dir.path().join("pkg");
    let message = cli(&[
        "prepare",
        input.to_str().unwrap(),
        "--home",
        home.as_str(),
        "--out",
        pkg.to_str().unwrap(),
        "--id",
        "pkg-1",
        "--title",
        "Стаття про Іваненко Петро",
        "--identity",
        identity.as_str(),
    ])
    .unwrap();
    assert!(message.contains("Пакет pkg-1"));
    let text = fs::read_to_string(pkg.join("deidentified.md")).unwrap();
    assert!(text.contains(&token));
    assert!(!text.contains("Іваненко"));
    assert!(!text.contains("1234567890"));
    assert!(pkg.join("prompt.md").exists());
    assert!(pkg.join("mentions.md").exists());
    let meta = fs::read_to_string(pkg.join("package.json")).unwrap();
    assert!(!meta.contains("Іваненка"));

    // 2. the AI's answer is checked and imported
    let result = dir.path().join("result.json");
    fs::write(
        &result,
        format!(
            r#"{{"package": "pkg-1",
 "sources": [{{"ref": "s1", "kind": "registry", "title": "Реєстр", "url": null,
   "published_at": null, "reliability": "A", "received_from": null}}],
 "entities": [
   {{"ref": "e1", "kind": "org", "label": "ТОВ Приклад", "token": null, "public_code": "12345678"}},
   {{"ref": "e2", "kind": "person", "label": null, "token": "{token}", "public_code": null}}],
 "statements": [{{"kind": "owns", "value": "40%", "valid_from": null, "valid_to": null,
   "as_of": null, "parties": [{{"entity": "e2", "role": "owner"}}, {{"entity": "e1", "role": "owned"}}],
   "sources": ["s1"], "credibility": 2, "quote": "володіє 40%", "note": null}}]}}"#
        ),
    )
    .unwrap();
    let checked = cli(&["check", result.to_str().unwrap(), "--package", "pkg-1"]).unwrap();
    assert!(checked.contains("тверджень 1"));
    assert!(dir.path().join("result.review.md").exists());
    let imported = cli(&[
        "import",
        result.to_str().unwrap(),
        "--package",
        "pkg-1",
        "--home",
        home.as_str(),
    ])
    .unwrap();
    assert!(imported.contains("тверджень 1"));
    assert!(cli(&[
        "import",
        result.to_str().unwrap(),
        "--package",
        "pkg-1",
        "--home",
        home.as_str()
    ])
    .is_err());

    // 3. the report reveals the person, and needs a passphrase or an explicit --plain
    let report = dir.path().join("report.md");
    let args = |extra: &[&str]| {
        let mut v = vec![
            "report",
            "--home",
            home.as_str(),
            "--identity",
            identity.as_str(),
            "--out",
            report.to_str().unwrap(),
        ];
        v.extend_from_slice(extra);
        cli(&v)
    };
    assert!(args(&[]).is_err());
    args(&["--plain"]).unwrap();
    let markdown = fs::read_to_string(&report).unwrap();
    assert!(markdown.contains("Іваненко Петро (owner)"));

    let pass = dir.path().join("pass.txt");
    fs::write(&pass, "довга парольна фраза\n").unwrap();
    args(&[
        "--passphrase-file",
        pass.to_str().unwrap(),
        "--format",
        "docx",
    ])
    .unwrap();
    let protected = dir.path().join("report.md.age");
    assert!(protected.exists());
    assert!(!fs::read(&protected).unwrap().starts_with(b"PK"));
}

#[test]
fn a_mention_can_be_resolved_and_is_then_recognised() {
    let dir = tempfile::tempdir().unwrap();
    let (home, identity) = setup(dir.path());
    let token = cli(&[
        "resolve",
        "Іванов І.",
        "--new",
        "Іваненко Петро Олексійович",
        "--ipn",
        "1234567890",
        "--home",
        home.as_str(),
    ])
    .unwrap()
    .trim()
    .to_string();
    let input = dir.path().join("doc.md");
    fs::write(&input, "Іванов І. має ІПН 1234567890.").unwrap();
    let pkg = dir.path().join("pkg");
    cli(&[
        "prepare",
        input.to_str().unwrap(),
        "--home",
        home.as_str(),
        "--out",
        pkg.to_str().unwrap(),
        "--id",
        "pkg-2",
        "--title",
        "Документ",
        "--identity",
        identity.as_str(),
    ])
    .unwrap();
    let text = fs::read_to_string(pkg.join("deidentified.md")).unwrap();
    assert_eq!(text.trim(), format!("{token} має ІПН {token}."));
}

#[test]
fn unknown_commands_and_missing_arguments_are_errors() {
    assert!(cli(&["bogus"]).is_err());
    assert!(cli(&["help"]).unwrap().contains("zanytky - команди"));
    let dir = tempfile::tempdir().unwrap();
    let home = home_arg(dir.path());
    assert!(cli(&["register", "name", "x", "--home", home.as_str()]).is_err());
}
