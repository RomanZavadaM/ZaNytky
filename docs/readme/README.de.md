# ZaNytky («За нитку»)

[🇺🇦 Українська](../../README.md) · [🇬🇧 English](README.en.md) · [🇫🇷 Français](README.fr.md) · **🇩🇪 Deutsch** · [🇪🇸 Español](README.es.md) · [🇰🇷 한국어](README.ko.md) · [🇯🇵 日本語](README.ja.md)

## Download

**Neueste Version: [ZaNytky v0.0.8](https://github.com/RomanZavadaM/ZaNytky/releases/tag/v0.0.23) (Vorabversion)**

| Plattform | Datei |
|---|---|
| Windows - Installer (empfohlen) | [ZaNytky_0.0.23_Setup_Windows_x64.exe](https://github.com/RomanZavadaM/ZaNytky/releases/download/v0.0.23/ZaNytky_0.0.23_Setup_Windows_x64.exe) |
| Windows - MSI | [ZaNytky_0.0.23_Windows_x64.msi](https://github.com/RomanZavadaM/ZaNytky/releases/download/v0.0.23/ZaNytky_0.0.23_Windows_x64.msi) |
| Windows - portables Archiv (nur zur Ansicht) | [ZaNytky_0.0.23_Windows_x64_Portable.zip](https://github.com/RomanZavadaM/ZaNytky/releases/download/v0.0.23/ZaNytky_0.0.23_Windows_x64_Portable.zip) |
| Windows - Befehlszeile | [ZaNytky_0.0.23_CLI_Windows_x64.zip](https://github.com/RomanZavadaM/ZaNytky/releases/download/v0.0.23/ZaNytky_0.0.23_CLI_Windows_x64.zip) |
| macOS (Apple Silicon) | [ZaNytky_0.0.23_macOS_arm64_Portable.zip](https://github.com/RomanZavadaM/ZaNytky/releases/download/v0.0.23/ZaNytky_0.0.23_macOS_arm64_Portable.zip) |
| macOS (Intel) | [ZaNytky_0.0.23_macOS_x86_64_Portable.zip](https://github.com/RomanZavadaM/ZaNytky/releases/download/v0.0.23/ZaNytky_0.0.23_macOS_x86_64_Portable.zip) |

- Prüfsummen: [SHA256SUMS.txt](https://github.com/RomanZavadaM/ZaNytky/releases/download/v0.0.23/SHA256SUMS.txt) und [Release-Seite](https://github.com/RomanZavadaM/ZaNytky/releases/tag/v0.0.23). Alle Versionen: [Releases](https://github.com/RomanZavadaM/ZaNytky/releases).
- Die Builds sind nicht signiert: Windows kann vor einem unbekannten Herausgeber warnen; macOS-Builds sind nicht notarisiert und nicht auf einem echten Mac getestet.
- Das portable Archiv dient nur zur Ansicht, ohne jede Gewähr; für den Dauerbetrieb den Installer verwenden.

Anwendung zur Ordnung von Beziehungen zwischen Subjekten aus offenen Quellen: Quellen mit Volltext und Dateien, Aussagen mit Zitaten, Beziehungsgraph, Suche des kürzesten Wegs zwischen Subjekten. Personenbezogene Daten werden durch schlüsselbasierte Pseudonyme ersetzt.

Status: **frühe Entwicklung** (Vorabversion 0.0.8). ZaNytky ist **proprietäre Software**: ein öffentliches Repository gewährt keine Open-Source-Lizenz (LICENSE.md, COPYRIGHT.md, THIRD_PARTY_NOTICES.md). Im Repository werden keine Nutzerdaten gespeichert.

> Diese Übersetzung ist maschinell erstellt und nicht von Muttersprachlern geprüft; maßgeblich ist der ukrainische Text.

## Funktionen

- **Quellen:** Volltext und Dateien (Scans, Fotos, Video) in der lokalen Datenbank; Suche nach Titel, Text und Adresse.
- **Telegram-Kanal-Import** (Export aus Telegram Desktop, JSON): jeder Beitrag wird zur Quelle mit Text und Anhängen.
- **Aussagen mit Zitat** aus der Quelle: Art, Beteiligte mit Rollen (vorhandene oder neue Subjekte), Glaubwürdigkeit; immer »ungeprüft«.
- **Subjekte:** Karte mit Aussagen, Quellen, Eigentumskette und gemeinsamen Merkmalen (Adresse, Telefon, Kennzeichen, Konto); Zusammenführen von Dubletten; Graph; kürzester Weg zwischen zwei Subjekten.
- **Pseudonymisierung:** eindeutige Codes mit Systemschlüssel; Originale nur versiegelt (age) für den Schlüssel des Ermittlers; Anonymisierung des Textes und KI-Paket; Prüfung und Import der KI-Antwort.
- **Bericht** als Markdown, Word und PDF mit Passphrase; verschlüsselte Sicherung des Datenordners. 7 Sprachen.

## Schnellstart

1. Anwendung installieren (oder das portable Archiv entpacken und `ZaNytky.exe` starten).
2. »Einstellungen«: leeren Datenordner außerhalb des Code-Ordners wählen, »Ordner vorbereiten«, Ermittlerschlüssel erzeugen und getrennt aufbewahren.
3. »Quellen«: Telegram-Kanal-Export importieren (Schaltfläche oben) oder Text und Dateien von Hand hinzufügen.
4. Quelle öffnen und Aussage erstellen: Zitat, Beteiligte, Rollen. »Subjekte« und »Graph« ansehen.
5. Für die KI-Analyse: »Dokument« (Anonymisierung und Paket) und »KI-Ergebnis« (Prüfung und Import). »Bericht« liefert die Zusammenfassung.

## Befehlszeile

Dasselbe gibt es als Befehle; `zanytky help` zeigt alle.

```bash
zanytky init --home ./home
zanytky identity-new ./investigator.txt
zanytky import-telegram "ChatExport_2026-11-01" --home ./home --reliability F
zanytky import result.json --package pkg-1 --home ./home
zanytky report --home ./home --identity ./investigator.txt --out report.docx --format docx --passphrase-file ./pass.txt
zanytky backup --home ./home --out copy.znbak --passphrase-file ./pass.txt
```

## Grundsätze

- Jede Aussage hat eine Quelle und eine Kette zur Ursprungsquelle; ein Gerücht wird nicht zur Tatsache, Echos eines Gerüchts sind eine Ursprungsquelle.
- Keine wertenden Etiketten über Personen: gezeigt werden Herkunft und Verlässlichkeit, keine Urteile.
- Nur offene Quellen. Texte und Dateien bleiben lokal und unanonymisiert; die KI erhält nur anonymisierten Text.

## Einschränkungen der aktuellen Version (0.0.23)

- Vorabversion: Installer sind nicht signiert.
- Noch kein PDF-Import und keine Texterkennung aus Scans oder Video; das Zitat für eine Aussage wird von Hand eingefügt.
- Word-Bericht und PDF-Ansicht wurden nicht in Microsoft Word und PDF-Betrachtern geprüft; die Erkennung unbekannter Namen ist einfach.
- Datenordner außerhalb des Code-Ordners halten und keine personenbezogenen Daten in die Paket-ID schreiben: sie geht ohne Anonymisierung an die KI.

## Sicherheit

Systemschlüssel (`system.key` im Datenordner), privaten Ermittlerschlüssel und Passphrase getrennt aufbewahren und nie ins Repository legen. Die Anwendung sendet nichts ins Netz.

## Dokumentation

- [Benutzerhandbuch](../../docs/user-guide/USER_GUIDE.de.md)
- [Versionshinweise](../../docs/releases)
- [LICENSE](../../LICENSE.md)
