# ZaNytky («За нитку»)

Застосунок для впорядкування зв'язків між суб'єктами з відкритих джерел із псевдонімізацією персональних даних за ключем.

Статус: **ранній етап розробки**, є командний рядок `zanytky`. Мова реалізації - Rust. Українська - основна мова інтерфейсу й документації.

ZaNytky - **proprietary software**: публічність репозиторію не надає open-source ліцензії (див. [LICENSE.md](LICENSE.md), [COPYRIGHT.md](COPYRIGHT.md), [THIRD_PARTY_NOTICES.md](THIRD_PARTY_NOTICES.md)).

Дані користувача в репозиторії не зберігаються.

## Що вміє

- Однозначні псевдоніми значень (імена, ІПН, телефони, пошти) за системним ключем: те саме значення завжди дає той самий код.
- Оригінали зберігаються лише запечатаними на публічний ключ розслідувача ([age](https://age-encryption.org)); розкрити може лише власник приватного ключа.
- Знеособлення тексту перед розбором і пакет для ШІ (текст, інструкція, метадані).
- Перевірка відповіді ШІ, вичитка людиною, імпорт у базу (SQLite) як «неперевірене».
- Модель: суб'єкти, джерела з ланцюжком до першоджерела, твердження з оцінкою надійності й достовірності, дерево власності.
- Звіт розслідувача у Markdown, Word і PDF, захищений парольною фразою.

## Швидкий початок

Потрібен [Rust](https://rustup.rs) (або готовий `zanytky.exe` з розділу Actions → «Windows build»).

```bash
cargo build --release -p zanytky-cli
# далі: target/release/zanytky (на Windows - zanytky.exe)

zanytky init --home ./home                       # системний ключ і порожній список ключів
zanytky identity-new ./investigator.txt          # пара ключів розслідувача; публічний ключ -> home/recipients.txt
zanytky register name "Іваненко Петро" --home ./home
zanytky prepare doc.md --home ./home --out ./pkg --id pkg-1 --title "Назва" --identity ./investigator.txt
# пакет з ./pkg віддається ШІ; відповідь зберігається як result.json
zanytky check  result.json --package pkg-1
zanytky import result.json --package pkg-1 --home ./home
zanytky report --home ./home --identity ./investigator.txt --out report.docx --format docx --passphrase-file ./pass.txt
```

`zanytky help` показує всі команди. Системний ключ (`home/system.key`), приватний ключ розслідувача і парольну фразу зберігайте окремо й ніколи не додавайте в репозиторій.

## Перевірки

`cargo fmt --all --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace`; ліцензії залежностей перевіряє `cargo deny` за [deny.toml](deny.toml). Перелік ліцензій виводиться в кожному запуску CI.
