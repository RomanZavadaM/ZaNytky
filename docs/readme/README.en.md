# ZaNytky («За нитку»)

[🇺🇦 Українська](../../README.md) · **🇬🇧 English** · [🇫🇷 Français](README.fr.md) · [🇩🇪 Deutsch](README.de.md) · [🇪🇸 Español](README.es.md) · [🇰🇷 한국어](README.ko.md) · [🇯🇵 日本語](README.ja.md)

An application for organising relationships between subjects from open sources, with keyed pseudonymisation of personal data.

Status: **early development** (pre-release). Implementation: Rust (core) and React (window). Ukrainian is the canonical language of the interface and documentation.

ZaNytky is **proprietary software**: a public repository does not grant an open-source license (see LICENSE.md, COPYRIGHT.md, THIRD_PARTY_NOTICES.md). No user data is stored in the repository.

> This translation is machine-made and has not been reviewed by native speakers; the Ukrainian text is authoritative.

## What it does

- One-to-one pseudonyms of values (names, tax numbers, phones, e-mails) under a system key: the same value always gives the same code.
- Originals are stored only sealed to the investigator's public key (age); only the owner of the private key can open them.
- De-identification of the text before AI analysis and a package for the AI (text, instruction, metadata); the investigator's decisions about unrecognised mentions.
- Checking the AI's answer, human review, import into the database (SQLite) as "unverified".
- A model of subjects, sources with a chain to the root source, statements with reliability and credibility grades, an ownership tree.
- Investigator's report in Markdown, Word and PDF, protected by a passphrase.

## Download

**Download: [ZaNytky v0.0.3](https://github.com/RomanZavadaM/ZaNytky/releases/tag/v0.0.3)** (latest pre-release)

Ready files for Windows (portable zip without installation, .msi/.exe installers) are on the [releases](https://github.com/RomanZavadaM/ZaNytky/releases) page. Builds are unsigned: Windows may warn about an unknown publisher.

## Quick start (application)

1. Unzip the portable archive and run `ZaNytky.exe` (nothing needs to be installed; keep the `webview2` folder next to `ZaNytky.exe`).
2. "Setup" tab: choose an empty data folder and press "Prepare the folder"; create an investigator key and keep its file in a safe place.
3. "Prepare document" tab: paste the text, press "De-identify and build the package for the AI", copy the text and the instruction and give them to the AI.
4. "AI result" tab: paste the answer (result.json), check it, read the review and import.
5. "Database" and "Report" tabs: browse statements, confirm or refute them, create a report.

## Command line

The same is available as commands; `zanytky help` lists them all.

```bash
zanytky init --home ./home
zanytky identity-new ./investigator.txt
zanytky register name "Іваненко Петро" --home ./home
zanytky prepare doc.md --home ./home --out ./pkg --id pkg-1 --title "Title" --identity ./investigator.txt
zanytky check  result.json --package pkg-1
zanytky import result.json --package pkg-1 --home ./home
zanytky report --home ./home --identity ./investigator.txt --out report.docx --format docx --passphrase-file ./pass.txt
```

## Principles

- Every claim has a source and a chain to the primary source; a rumour does not become a fact, and echoes of one rumour count as one primary source.
- No evaluative labels about people: the program shows origin and a reliability grade, not verdicts.
- Open sources only; personal data is replaced by pseudonyms; the AI receives only de-identified text.

## Limitations of the current version (0.0.3)

- This is a pre-release: the installers are not signed.
- "Document analysis" currently accepts only text and Markdown (no .docx/.pdf import yet).
- The Word report has not been checked in Microsoft Word and the PDF has not been checked in a viewer; unknown-name detection is simple (capitalised words) and gives false candidates.
- Keep the data folder outside the code folder and do not put personal data (for example an e-mail address) in the package identifier: it is passed to the AI without de-identification.

## Security

Keep the system key (`system.key` in the data folder), the investigator's private key and the passphrase separately and never add them to a repository. The application sends nothing over the network.

## Documentation

- [User guide](../user-guide/USER_GUIDE.en.md)
- [LICENSE](../../LICENSE.md)
