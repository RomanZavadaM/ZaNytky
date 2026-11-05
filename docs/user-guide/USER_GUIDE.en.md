# ZaNytky - user guide

[🇺🇦 Українська](USER_GUIDE.uk.md) · **🇬🇧 English** · [🇫🇷 Français](USER_GUIDE.fr.md) · [🇩🇪 Deutsch](USER_GUIDE.de.md) · [🇪🇸 Español](USER_GUIDE.es.md) · [🇰🇷 한국어](USER_GUIDE.ko.md) · [🇯🇵 日本語](USER_GUIDE.ja.md)

This guide describes the application window. The Ukrainian version is the reference text.

> This translation is machine-made and has not been reviewed by native speakers; the Ukrainian text is authoritative.

## 1. Setup

- Choose a **data folder** (an empty one is best) and press "Prepare the folder". It receives the system key `system.key` and the file `recipients.txt` with investigators' public keys.
- Create an **investigator key**: the application writes the private key to a file you choose and adds the public key to `recipients.txt`. The private key is needed to reveal names and for reports; without it the data cannot be revealed. Keep the file in a safe place and make a copy.

## 2. Prepare a document

- Enter a title, the source kind (media, registry, authority statement, testimony, rumor, own), optionally an address, and a package id (unique for each document).
- Paste the text or open a file. If needed add the **investigator's context**, for example "Ivanov I. is the head of Example LLC".
- Press "De-identify and build the package for the AI". The application replaces known names, tax numbers, phones and e-mails with pseudonyms. Copy the de-identified text and the instruction and give them to the AI.
- In "Mentions the system did not recognise" choose for each mention: **same person** (give the pseudonym), **new person** (full name, optionally tax number, phone, e-mail) or **not a person**. Then build the package again.

## 3. AI result

- Paste the AI's answer (the result.json file) and the package id. Press "Check": the application verifies the structure and rules and shows a review with quotes. Only after the review press "Import into the database".
- Everything is imported as "unverified". A package can be imported only once.

## 4. Database

- A list of statements with a grade (for example B3: source reliability B, credibility 3) and a status. Persons are shown by pseudonym only.
- "Confirm" is possible only if there is a source that is not a rumor or testimony; otherwise the application refuses. "Refute" marks the statement refuted (it is not deleted).

## 5. Report

- Choose a format (Word, PDF, Markdown) and enter a passphrase. The report reveals persons with your private key, so it contains open personal data. Without a passphrase a report is created only after you explicitly confirm "No passphrase".
- A protected report has the extension `.age`; open it with [age](https://age-encryption.org) and your passphrase.

## 6. Sources and Telegram channels

- **Importing a channel.** In Telegram Desktop: channel → "Export chat history" → JSON format. In the "Sources" tab press "Choose folder and import" and pick the folder with `result.json`. Every post becomes a source with its full text; photos, videos and files become attachments. The import can be repeated: no duplicates. The reliability of the sources is F (unknown) by default.
- **Sources.** Search by title, text and address (word beginnings, any letter case, every word must be present). In an open source: the text (paste, save, copy), "Add file", "Show in folder", "Show picture".
- **A statement from a source.** In an open source: kind, value, credibility 1-6, a **required quote**, 2-4 parties with roles (existing or new subjects). A new person is entered by full name: the name goes to the sealed registry, only a pseudonym stays in the base. A statement is always "unverified".
- **Automatic discovery of subjects.** "Find subjects in the text" lists persons, organisations (LLC "...") and registry codes marked "in the base" or "new" with an "Add to the base" button. "New subjects in all texts" gives the top of the unknown ones across the base. These are hints: confirm by hand.
- **Channels.** A timeline by month with activity peaks, post numbers missing from the export (possibly deleted), a channel graph: where it forwards from, what it links to, whom it mentions. Click a node to expand it (posts, other channels of the base with the same connection). To fill the analysis for a channel imported earlier, import its folder again.
- **Overlaps** (at least two channels): rows with a bar per channel - a common source of forwards, channel, site, mention, identical long text; a filter by kind; the graph on request. Generic platforms (YouTube, Facebook, etc.) are left out.
- **Overview.** One button writes a report over the whole base: channels, peaks, overlaps, new subjects and **likely links** (pairs of subjects named in the same paragraph in at least 3 posts, with an example quote). Save it as `.md` with the button; the same on the command line: `zanytky overview --home FOLDER --out FILE`.
- **Important.** Overlaps, peaks, discovery and the Overview are hints for hypotheses, not facts. Texts and files of sources are stored without de-identification in the local database only; keep the data folder outside code folders and make backups.

## 7. Evidence, graph and the command line

- **Add evidence.** In a subject card next to a statement: kind of document (registry, official statement, media, testimony, own), reliability A-F, title, address, date, text or extract, note. "Add and confirm" is available only for the kinds "registry" and "official statement"; "Add and refute" for any kind. The same address is not duplicated.
- **Graph.** The "Graph" tab: line colour is the kind of link (ownership, position, reported link...), a thick solid line is confirmed by a document, dashed is not checked yet; nodes are labelled, names of persons are shown when the investigator's key is set. "Save graph as HTML..." writes one standalone file (no internet, with search): the whole graph or only around subject no. (1-3 steps), optionally only confirmed.
- **Command line for analysis.** `search QUERY`, `source NUMBER --full`, `discover --source N`, `channels`, `overlaps`, `pairs`, `entity N`, `statements --grep`, `add-source`, `add-entity`, `make-statements --file FILE.json` (the quote of each statement must be verbatim in its source text, otherwise nothing is written), `evidence`, `graph --out FILE.html [--entity N --depth K] [--confirmed]`. Every reading command has `--json`.
- **Fixing mistakes.** `detach --statement N --source M [--drop-source]` detaches a wrongly added evidence; `delete-statement N` and `delete-source N` delete a wrong statement or an unused source; `merge --keep A --drop B` merges duplicates (not persons), `unmerge --drop B` undoes it; `evidence --expect WORDS` stops an evidence from going to a wrong number. Deletion is irreversible (it is logged): run `backup` first.
- **Graph nodes.** Colour by type: person yellow, organisation blue, event purple, asset green, group pink; reliability is shown by the node outline (dashed is weaker).
- **Large imports and the locked base.** Importing a large channel keeps the base open for writing for a long time; meanwhile the app may say the base is locked. Wait for the import to end and refresh the window; do not kill the import (unfinished work is rolled back, but the journal stays until the next opening).

## Security

- Keep the system key, the investigator's private key and the passphrase separately. Losing the private key means the data can no longer be revealed.
- The application sends nothing over the network. The text you give to the AI leaves your computer, so submit only material from open sources.

## FAQ

- Windows warns about an unknown publisher: the build is unsigned; choose "More info" and "Run anyway".
- Empty window or a start error: make sure the `webview2` folder is next to `ZaNytky.exe` (the portable archive needs no installation).

## Trial scenario (fictional data)

1. "Setup" tab: choose an empty folder, press "Prepare the folder", create an investigator key.
2. "Prepare document" tab: paste the text from `docs/examples/sample-document.md`, title "Trial document", package id `pkg-1`, press "De-identify and build the package".
3. In the list of unrecognised mentions choose "New person" for "Петро Іваненко", enter the full name, tax number 1234567890 and phone +38 050 123 45 67, press "Apply". Build the package again: now the name, tax number and phone are replaced by one pseudonym.
4. Copy the instruction and the de-identified text into an AI chat and ask it to answer with JSON only. Save the answer.
5. "AI result" tab: paste the answer, press "Check", read the review and press "Import into the database".
6. "Database" tab: browse the statements; try "Confirm" (for a statement sourced from a rumor the application refuses).
7. "Report" tab: create a Word report with a passphrase; open it with age and check that it contains the real name.
