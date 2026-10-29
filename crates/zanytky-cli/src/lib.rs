//! Command line for ZaNytky.
//!
//! The home folder holds `system.key` (secret, never printed), `recipients.txt`
//! (public keys of investigators), `registry.db` and `model.db`. The
//! investigator's private identity and passphrase stay in files the
//! investigator keeps wherever they choose.

use std::collections::{HashMap, HashSet};
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

use zanytky_core::model::{Model, Reliability, SourceKind};
use zanytky_core::normalize::Kind;
use zanytky_core::pseudonym::{new_key, MIN_KEY_BYTES};
use zanytky_core::registry::{Registry, Resolution};
use zanytky_core::seal::{generate_identity, parse_identity, parse_recipient, Identity};
use zanytky_core::{
    backup_home, build_package_with_links, build_report, import_result, parse_result, pdf,
    restore_home, review_markdown, to_docx, to_markdown, wrap::wrap, SourceMeta,
};

type Out<'a> = &'a mut dyn Write;

const HELP: &str = "\
zanytky - команди

  init --home ПАПКА                       створити системний ключ і порожній список ключів розслідувачів
  identity-new ФАЙЛ                       створити пару ключів розслідувача (приватний - у файл, публічний - на екран)
  register ВИД ЗНАЧЕННЯ                   зареєструвати значення, вивести псевдонім
  find ВИД ЗНАЧЕННЯ                       чи є таке значення в реєстрі
  resolve ЗГАДКА (--same ТОКЕН | --new ІМ'Я [--ipn Х] [--phone Х] [--email Х] | --not-person)
                                          рішення розслідувача про згадку
  prepare ВХІД.md --out ПАПКА --id ПАКЕТ --title НАЗВА [--kind К] [--reliability Н] [--url У]
          [--date Д] [--from ВІД] [--context ФАЙЛ] [--identity ФАЙЛ]
                                          знеособити текст і скласти пакет для ШІ
  check result.json --package ПАКЕТ       перевірити відповідь ШІ, записати вичитку
  import result.json --package ПАКЕТ      записати перевірений результат у базу
  report --identity ФАЙЛ --out ФАЙЛ [--format md|docx|pdf] (--passphrase-file ФАЙЛ | --plain)
          [--title НАЗВА]                 звіт розслідувача (з розкриттям осіб)
  backup --out ФАЙЛ --passphrase-file ФАЙЛ
                                          зашифрована резервна копія всієї папки даних (закрийте інші програми)
  restore ФАЙЛ --to ПАПКА --passphrase-file ФАЙЛ
                                          відновити копію в порожню (або нову) папку; наявні файли не чіпає

Скрізь потрібно --home ПАПКА (або змінна ZANYTKY_HOME).
Види значень: name, ipn, edrpou, phone, email, text.
";

const SWITCHES: [&str; 2] = ["plain", "not-person"];

struct Args {
    positional: Vec<String>,
    flags: HashMap<String, String>,
    switches: HashSet<String>,
}

impl Args {
    fn parse(raw: &[String]) -> Args {
        let mut args = Args {
            positional: Vec::new(),
            flags: HashMap::new(),
            switches: HashSet::new(),
        };
        let mut i = 0;
        while i < raw.len() {
            if let Some(name) = raw[i].strip_prefix("--") {
                if SWITCHES.contains(&name) {
                    args.switches.insert(name.to_string());
                } else if i + 1 < raw.len() {
                    args.flags.insert(name.to_string(), raw[i + 1].clone());
                    i += 1;
                }
            } else {
                args.positional.push(raw[i].clone());
            }
            i += 1;
        }
        args
    }

    fn flag(&self, name: &str) -> Option<&str> {
        self.flags.get(name).map(String::as_str)
    }

    fn need(&self, name: &str) -> Result<&str, String> {
        self.flag(name)
            .ok_or_else(|| format!("потрібен параметр --{name}"))
    }

    fn pos(&self, index: usize, what: &str) -> Result<&str, String> {
        self.positional
            .get(index)
            .map(String::as_str)
            .ok_or_else(|| format!("потрібен аргумент: {what}"))
    }
}

fn io_error(path: &Path, error: std::io::Error) -> String {
    format!("{}: {error}", path.display())
}

fn read_text(path: &Path) -> Result<String, String> {
    fs::read_to_string(path).map_err(|e| io_error(path, e))
}

fn write_file(path: &Path, bytes: &[u8]) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        if !parent.as_os_str().is_empty() {
            fs::create_dir_all(parent).map_err(|e| io_error(parent, e))?;
        }
    }
    fs::write(path, bytes).map_err(|e| io_error(path, e))
}

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

fn unhex(text: &str) -> Result<Vec<u8>, String> {
    let text = text.trim();
    if !text.is_ascii() || !text.len().is_multiple_of(2) {
        return Err("системний ключ пошкоджено".to_string());
    }
    (0..text.len())
        .step_by(2)
        .map(|i| {
            u8::from_str_radix(&text[i..i + 2], 16)
                .map_err(|_| "системний ключ пошкоджено".to_string())
        })
        .collect()
}

fn home_dir(args: &Args) -> Result<PathBuf, String> {
    match args.flag("home") {
        Some(path) => Ok(PathBuf::from(path)),
        None => std::env::var("ZANYTKY_HOME")
            .map(PathBuf::from)
            .map_err(|_| "потрібен --home ПАПКА або змінна ZANYTKY_HOME".to_string()),
    }
}

fn load_key(home: &Path) -> Result<Vec<u8>, String> {
    let key = unhex(&read_text(&home.join("system.key"))?)?;
    if key.len() < MIN_KEY_BYTES {
        return Err("системний ключ закороткий".to_string());
    }
    Ok(key)
}

fn load_recipients(home: &Path) -> Result<Vec<zanytky_core::seal::Recipient>, String> {
    let text = read_text(&home.join("recipients.txt"))?;
    let mut out = Vec::new();
    for line in text.lines().map(str::trim) {
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        out.push(parse_recipient(line).map_err(|e| e.to_string())?);
    }
    if out.is_empty() {
        return Err("у recipients.txt немає публічних ключів розслідувачів".to_string());
    }
    Ok(out)
}

fn open_registry(home: &Path) -> Result<Registry, String> {
    Registry::open(
        &home.join("registry.db"),
        &load_key(home)?,
        load_recipients(home)?,
    )
    .map_err(|e| e.to_string())
}

fn open_model(home: &Path) -> Result<Model, String> {
    Model::open(&home.join("model.db"), "cli").map_err(|e| e.to_string())
}

fn read_identity(path: &str) -> Result<Identity, String> {
    let text = read_text(Path::new(path))?;
    let line = text
        .lines()
        .map(str::trim)
        .find(|l| l.starts_with("AGE-SECRET-KEY-"))
        .ok_or_else(|| "у файлі немає приватного ключа розслідувача".to_string())?;
    parse_identity(line).map_err(|e| e.to_string())
}

fn kind_of(text: &str) -> Result<Kind, String> {
    Kind::parse(text).ok_or_else(|| format!("невідомий вид значення: {text}"))
}

fn cmd_init(args: &Args, out: Out) -> Result<(), String> {
    let home = home_dir(args)?;
    fs::create_dir_all(&home).map_err(|e| io_error(&home, e))?;
    let key_path = home.join("system.key");
    if key_path.exists() {
        return Err("системний ключ уже існує, він не перезаписується".to_string());
    }
    let key = new_key().map_err(|e| e.to_string())?;
    write_file(&key_path, hex(&key).as_bytes())?;
    let recipients = home.join("recipients.txt");
    if !recipients.exists() {
        write_file(
            &recipients,
            "# Публічні ключі розслідувачів (age1...), по одному в рядку.\n".as_bytes(),
        )?;
    }
    writeln!(out, "Створено {}", home.display()).map_err(|e| e.to_string())?;
    writeln!(
        out,
        "Збережіть system.key окремо від даних і ніколи не додавайте його в репозиторій. \
         Додайте публічний ключ розслідувача в recipients.txt."
    )
    .map_err(|e| e.to_string())
}

fn cmd_identity_new(args: &Args, out: Out) -> Result<(), String> {
    let path = PathBuf::from(args.pos(0, "файл для приватного ключа")?);
    if path.exists() {
        return Err("файл уже існує, його не перезаписую".to_string());
    }
    let (secret, public) = generate_identity();
    write_file(
        &path,
        format!("# приватний ключ розслідувача - зберігайте у безпечному місці\n{secret}\n")
            .as_bytes(),
    )?;
    writeln!(out, "Публічний ключ (додайте в recipients.txt): {public}").map_err(|e| e.to_string())
}

fn cmd_register(args: &Args, out: Out) -> Result<(), String> {
    let registry = open_registry(&home_dir(args)?)?;
    let token = registry
        .register(kind_of(args.pos(0, "вид")?)?, args.pos(1, "значення")?)
        .map_err(|e| e.to_string())?;
    writeln!(out, "{token}").map_err(|e| e.to_string())
}

fn cmd_find(args: &Args, out: Out) -> Result<(), String> {
    let registry = open_registry(&home_dir(args)?)?;
    let found = registry
        .find(kind_of(args.pos(0, "вид")?)?, args.pos(1, "значення")?)
        .map_err(|e| e.to_string())?;
    let line = found.unwrap_or_else(|| "немає".to_string());
    writeln!(out, "{line}").map_err(|e| e.to_string())
}

fn cmd_resolve(args: &Args, out: Out) -> Result<(), String> {
    let registry = open_registry(&home_dir(args)?)?;
    let mention = args.pos(0, "згадка")?;
    let resolution = if args.switches.contains("not-person") {
        Resolution::NotAPerson
    } else if let Some(person) = args.flag("same") {
        Resolution::SamePerson(person.to_string())
    } else if let Some(name) = args.flag("new") {
        let mut codes = Vec::new();
        for (flag, kind) in [
            ("ipn", Kind::Ipn),
            ("phone", Kind::Phone),
            ("email", Kind::Email),
        ] {
            if let Some(value) = args.flag(flag) {
                codes.push((kind, value.to_string()));
            }
        }
        Resolution::NewPerson {
            name: name.to_string(),
            codes,
        }
    } else {
        return Err("потрібно --same, --new або --not-person".to_string());
    };
    let person = registry
        .resolve_mention(mention, &resolution)
        .map_err(|e| e.to_string())?;
    let line = person.unwrap_or_else(|| "не особа, без змін".to_string());
    writeln!(out, "{line}").map_err(|e| e.to_string())
}

fn cmd_prepare(args: &Args, out: Out) -> Result<(), String> {
    let home = home_dir(args)?;
    let registry = open_registry(&home)?;
    let key = load_key(&home)?;
    let text = read_text(Path::new(args.pos(0, "вхідний файл")?))?;
    let context = match args.flag("context") {
        Some(path) => Some(read_text(Path::new(path))?),
        None => None,
    };
    let (dictionary, warning) = match args.flag("identity") {
        Some(path) => (
            registry
                .dictionary(&read_identity(path)?)
                .map_err(|e| e.to_string())?,
            false,
        ),
        None => (Vec::new(), true),
    };
    let links = registry.links().map_err(|e| e.to_string())?;
    let meta = SourceMeta {
        kind: SourceKind::parse(args.flag("kind").unwrap_or("media"))
            .ok_or("невідомий вид джерела")?,
        title: args.need("title")?.to_string(),
        url: args.flag("url").map(str::to_string),
        published_at: args.flag("date").map(str::to_string),
        reliability: Reliability::parse(args.flag("reliability").unwrap_or("C"))
            .ok_or("невідома надійність")?,
        received_from: args.flag("from").map(str::to_string),
    };
    let id = args.need("id")?;
    let package = build_package_with_links(
        id,
        &meta,
        &text,
        context.as_deref(),
        &key,
        &dictionary,
        &links,
    )
    .map_err(|e| e.to_string())?;

    let dir = PathBuf::from(args.need("out")?);
    write_file(
        &dir.join("deidentified.md"),
        package.deidentified.as_bytes(),
    )?;
    write_file(&dir.join("prompt.md"), package.prompt.as_bytes())?;
    write_file(&dir.join("package.json"), package.meta_json.as_bytes())?;
    let mut mentions = String::from("# Згадки, які система не впізнала\n\n");
    if package.candidates.is_empty() {
        mentions.push_str("Немає.\n");
    }
    for candidate in &package.candidates {
        mentions.push_str(&format!("- {} ({})\n", candidate.text, candidate.count));
    }
    write_file(&dir.join("mentions.md"), mentions.as_bytes())?;

    writeln!(
        out,
        "Пакет {id}: замін {}, невпізнаних згадок {}. Файли в {}",
        package.replacements.iter().map(|r| r.count).sum::<u32>(),
        package.candidates.len(),
        dir.display()
    )
    .map_err(|e| e.to_string())?;
    if warning {
        writeln!(
            out,
            "Увага: не вказано --identity, тож імена зі словника не замінено (лише коди, телефони, пошти)."
        )
        .map_err(|e| e.to_string())?;
    }
    Ok(())
}

fn cmd_check(args: &Args, out: Out) -> Result<(), String> {
    let path = PathBuf::from(args.pos(0, "result.json")?);
    let file =
        parse_result(&read_text(&path)?, args.need("package")?).map_err(|e| e.to_string())?;
    let review = path.with_extension("review.md");
    write_file(&review, review_markdown(&file).as_bytes())?;
    writeln!(
        out,
        "Результат коректний: джерел {}, суб'єктів {}, тверджень {}. Вичитка: {}",
        file.sources.len(),
        file.entities.len(),
        file.statements.len(),
        review.display()
    )
    .map_err(|e| e.to_string())
}

fn cmd_import(args: &Args, out: Out) -> Result<(), String> {
    let home = home_dir(args)?;
    let path = PathBuf::from(args.pos(0, "result.json")?);
    let file =
        parse_result(&read_text(&path)?, args.need("package")?).map_err(|e| e.to_string())?;
    let summary = import_result(&open_model(&home)?, &file).map_err(|e| e.to_string())?;
    writeln!(
        out,
        "Імпортовано: джерел {}, нових суб'єктів {}, наявних {}, тверджень {} (усі неперевірені).",
        summary.sources, summary.entities_new, summary.entities_existing, summary.statements
    )
    .map_err(|e| e.to_string())
}

fn cmd_report(args: &Args, out: Out) -> Result<(), String> {
    let home = home_dir(args)?;
    let registry = open_registry(&home)?;
    let model = open_model(&home)?;
    let identity = read_identity(args.need("identity")?)?;
    let report = build_report(
        args.flag("title").unwrap_or("Звіт розслідувача"),
        &model,
        &registry,
        &identity,
    )
    .map_err(|e| e.to_string())?;
    let bytes = match args.flag("format").unwrap_or("md") {
        "md" => to_markdown(&report).into_bytes(),
        "docx" => to_docx(&report).map_err(|e| e.to_string())?,
        "pdf" => pdf::to_pdf(&report).map_err(|e| e.to_string())?,
        other => return Err(format!("невідомий формат: {other}")),
    };
    let mut target = PathBuf::from(args.need("out")?);
    let bytes = match args.flag("passphrase-file") {
        Some(path) => {
            let passphrase = read_text(Path::new(path))?;
            let passphrase = passphrase.trim_end_matches(['\r', '\n']);
            let mut name = target.as_os_str().to_owned();
            name.push(".age");
            target = PathBuf::from(name);
            wrap(&bytes, passphrase).map_err(|e| e.to_string())?
        }
        None if args.switches.contains("plain") => bytes,
        None => {
            return Err(
                "потрібен --passphrase-file (захист паролем) або явне --plain (без захисту)"
                    .to_string(),
            )
        }
    };
    write_file(&target, &bytes)?;
    writeln!(out, "Звіт записано: {}", target.display()).map_err(|e| e.to_string())
}

fn passphrase_from(args: &Args) -> Result<String, String> {
    let text = read_text(Path::new(args.need("passphrase-file")?))?;
    let passphrase = text.trim_end_matches(['', '
']);
    if passphrase.is_empty() {
        return Err("файл парольної фрази порожній".to_string());
    }
    Ok(passphrase.to_string())
}

fn cmd_backup(args: &Args, out: Out) -> Result<(), String> {
    let home = home_dir(args)?;
    let target = PathBuf::from(args.need("out")?);
    if target.exists() {
        return Err("файл копії вже існує, він не перезаписується".to_string());
    }
    let bytes = backup_home(&home, &passphrase_from(args)?).map_err(|e| e.to_string())?;
    write_file(&target, &bytes)?;
    writeln!(out, "Копію записано: {} ({} байт)", target.display(), bytes.len())
        .map_err(|e| e.to_string())
}

fn cmd_restore(args: &Args, out: Out) -> Result<(), String> {
    let source = PathBuf::from(args.pos(0, "файл резервної копії")?);
    let dest = PathBuf::from(args.need("to")?);
    let data = fs::read(&source).map_err(|e| io_error(&source, e))?;
    let count = restore_home(&data, &passphrase_from(args)?, &dest).map_err(|e| e.to_string())?;
    writeln!(out, "Відновлено файлів: {count} у {}", dest.display()).map_err(|e| e.to_string())
}

/// Run one command. `args` are the arguments after the program name.
pub fn run(args: &[String], out: &mut dyn Write) -> Result<(), String> {
    let Some((command, rest)) = args.split_first() else {
        return write!(out, "{HELP}").map_err(|e| e.to_string());
    };
    let parsed = Args::parse(rest);
    match command.as_str() {
        "init" => cmd_init(&parsed, out),
        "identity-new" => cmd_identity_new(&parsed, out),
        "register" => cmd_register(&parsed, out),
        "find" => cmd_find(&parsed, out),
        "resolve" => cmd_resolve(&parsed, out),
        "prepare" => cmd_prepare(&parsed, out),
        "check" => cmd_check(&parsed, out),
        "import" => cmd_import(&parsed, out),
        "report" => cmd_report(&parsed, out),
        "backup" => cmd_backup(&parsed, out),
        "restore" => cmd_restore(&parsed, out),
        "help" | "--help" | "-h" => write!(out, "{HELP}").map_err(|e| e.to_string()),
        other => Err(format!("невідома команда: {other}. Див. zanytky help")),
    }
}
