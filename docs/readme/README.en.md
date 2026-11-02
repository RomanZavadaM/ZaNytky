# ZaNytky («За нитку»)

[🇺🇦 Українська](../../README.md) · **🇬🇧 English** · [🇫🇷 Français](README.fr.md) · [🇩🇪 Deutsch](README.de.md) · [🇪🇸 Español](README.es.md) · [🇰🇷 한국어](README.ko.md) · [🇯🇵 日本語](README.ja.md)

## Download

**Latest version: [ZaNytky v0.0.8](https://github.com/RomanZavadaM/ZaNytky/releases/tag/v0.0.9) (pre-release)**

| Platform | File |
|---|---|
| Windows - installer (recommended) | [ZaNytky_0.0.9_Setup_Windows_x64.exe](https://github.com/RomanZavadaM/ZaNytky/releases/download/v0.0.9/ZaNytky_0.0.9_Setup_Windows_x64.exe) |
| Windows - MSI | [ZaNytky_0.0.9_Windows_x64.msi](https://github.com/RomanZavadaM/ZaNytky/releases/download/v0.0.9/ZaNytky_0.0.9_Windows_x64.msi) |
| Windows - portable archive (evaluation only) | [ZaNytky_0.0.9_Windows_x64_Portable.zip](https://github.com/RomanZavadaM/ZaNytky/releases/download/v0.0.9/ZaNytky_0.0.9_Windows_x64_Portable.zip) |
| Windows - command line | [ZaNytky_0.0.9_CLI_Windows_x64.zip](https://github.com/RomanZavadaM/ZaNytky/releases/download/v0.0.9/ZaNytky_0.0.9_CLI_Windows_x64.zip) |
| macOS (Apple Silicon) | [ZaNytky_0.0.9_macOS_arm64_Portable.zip](https://github.com/RomanZavadaM/ZaNytky/releases/download/v0.0.9/ZaNytky_0.0.9_macOS_arm64_Portable.zip) |
| macOS (Intel) | [ZaNytky_0.0.9_macOS_x86_64_Portable.zip](https://github.com/RomanZavadaM/ZaNytky/releases/download/v0.0.9/ZaNytky_0.0.9_macOS_x86_64_Portable.zip) |

- Checksums are in [SHA256SUMS.txt](https://github.com/RomanZavadaM/ZaNytky/releases/download/v0.0.9/SHA256SUMS.txt) and on the [release page](https://github.com/RomanZavadaM/ZaNytky/releases/tag/v0.0.9). All releases: [Releases](https://github.com/RomanZavadaM/ZaNytky/releases).
- Builds are not signed: Windows may warn about an unknown publisher; macOS builds are not notarised and have not been tested on a real Mac.
- The portable archive is for evaluation only, without any warranty; use the installer for regular work.

An application for organising relationships between subjects from open sources: sources with full text and files, statements with quotes, a relationship graph, shortest-path search between subjects. Personal data is replaced with keyed pseudonyms.

Status: **early development** (pre-release 0.0.8). ZaNytky is **proprietary software**: a public repository does not grant an open-source licence (LICENSE.md, COPYRIGHT.md, THIRD_PARTY_NOTICES.md). No user data is stored in the repository.

> This translation is machine-made and has not been reviewed by native speakers; the Ukrainian text is authoritative.

## What it does

- **Sources:** full text and files (scans, photos, video) are kept in the local database; search by title, text and address.
- **Telegram channel import** (Telegram Desktop export, JSON): every post becomes a source with text and attachments.
- **Statements with a quote** from the source: kind, parties with roles (existing or new subjects), credibility grade; always "unverified".
- **Subjects:** a card with statements, sources, ownership chain and shared signs (address, phone, plate, account); merging duplicates; relationship graph; shortest path between two subjects.
- **Pseudonymisation:** one-to-one codes under a system key; originals only sealed (age) to the investigator's key; de-identification of text and a package for AI; checking and importing the AI's answer.
- **Report** in Markdown, Word and PDF with a passphrase; encrypted backup of the data folder. 7 interface languages.

## Quick start

1. Install the application (or unpack the portable archive and run `ZaNytky.exe`).
2. "Setup": choose an empty data folder outside the code folder, press "Prepare folder", create the investigator's key and keep it separately.
3. "Sources": import a Telegram channel export (button at the top) or add text and files to a source by hand.
4. Open a source and make a statement: quote, parties, roles. Browse "Entities" and "Graph".
5. For AI analysis: "Document" (de-identification and package) and "AI result" (check and import). "Report" gives the summary.

## Command line

The same is available as commands; `zanytky help` lists them all.

```bash
zanytky init --home ./home
zanytky identity-new ./investigator.txt
zanytky import-telegram "ChatExport_2026-11-01" --home ./home --reliability F
zanytky import result.json --package pkg-1 --home ./home
zanytky report --home ./home --identity ./investigator.txt --out report.docx --format docx --passphrase-file ./pass.txt
zanytky backup --home ./home --out copy.znbak --passphrase-file ./pass.txt
```

## Principles

- Every statement has a source and a chain to the root source; a rumour does not become a fact, and echoes of one rumour are one root source.
- No judgemental labels about people: we show origin and reliability grades, not verdicts.
- Open sources only. Texts and files are kept locally, without de-identification; the AI receives only de-identified text.

## Limitations of the current version (0.0.9)

- Pre-release: installers are not signed.
- No PDF import and no text recognition from scans or video yet; the quote for a statement is pasted by hand.
- The Word report and PDF view have not been checked in Microsoft Word or PDF viewers; recognition of unknown names is simple.
- Keep the data folder outside the code folder and do not put personal data in the package identifier: it goes to the AI without de-identification.

## Security

Keep the system key (`system.key` in the data folder), the investigator's private key and the passphrase separately and never add them to the repository. The application sends nothing to the network.

## Documentation

- [User guide](../../docs/user-guide/USER_GUIDE.en.md)
- [Release notes](../../docs/releases)
- [LICENSE](../../LICENSE.md)
