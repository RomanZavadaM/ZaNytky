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

## Security

- Keep the system key, the investigator's private key and the passphrase separately. Losing the private key means the data can no longer be revealed.
- The application sends nothing over the network. The text you give to the AI leaves your computer, so submit only material from open sources.

## FAQ

- Windows warns about an unknown publisher: the build is unsigned; choose "More info" and "Run anyway".
- Empty window or a start error: Microsoft Edge WebView2 is required (present in Windows 10/11).

## Trial scenario (fictional data)

1. "Setup" tab: choose an empty folder, press "Prepare the folder", create an investigator key.
2. "Prepare document" tab: paste the text from `docs/examples/sample-document.md`, title "Trial document", package id `pkg-1`, press "De-identify and build the package".
3. In the list of unrecognised mentions choose "New person" for "Петро Іваненко", enter the full name, tax number 1234567890 and phone +38 050 123 45 67, press "Apply". Build the package again: now the name, tax number and phone are replaced by one pseudonym.
4. Copy the instruction and the de-identified text into an AI chat and ask it to answer with JSON only. Save the answer.
5. "AI result" tab: paste the answer, press "Check", read the review and press "Import into the database".
6. "Database" tab: browse the statements; try "Confirm" (for a statement sourced from a rumor the application refuses).
7. "Report" tab: create a Word report with a passphrase; open it with age and check that it contains the real name.
