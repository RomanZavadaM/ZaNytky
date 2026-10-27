# ZaNytky («За нитку»)

[🇺🇦 Українська](../../README.md) · [🇬🇧 English](README.en.md) · [🇫🇷 Français](README.fr.md) · **🇩🇪 Deutsch** · [🇪🇸 Español](README.es.md) · [🇰🇷 한국어](README.ko.md) · [🇯🇵 日本語](README.ja.md)

Anwendung zur Ordnung von Beziehungen zwischen Subjekten aus offenen Quellen mit schlüsselbasierter Pseudonymisierung personenbezogener Daten.

Status: **frühe Entwicklung** (Vorabversion). Umsetzung: Rust (Kern) und React (Fenster). Ukrainisch ist die maßgebliche Sprache der Oberfläche und Dokumentation.

ZaNytky ist **proprietäre Software**: ein öffentliches Repository gewährt keine Open-Source-Lizenz (siehe LICENSE.md, COPYRIGHT.md, THIRD_PARTY_NOTICES.md). Im Repository werden keine Nutzerdaten gespeichert.

> Diese Übersetzung ist maschinell erstellt und nicht von Muttersprachlern geprüft; maßgeblich ist der ukrainische Text.

## Funktionen

- Eindeutige Pseudonyme von Werten (Namen, Steuernummern, Telefone, E-Mails) mit einem Systemschlüssel: derselbe Wert ergibt immer denselben Code.
- Originale werden nur versiegelt für den öffentlichen Schlüssel des Ermittlers gespeichert (age); nur der Besitzer des privaten Schlüssels kann sie öffnen.
- Anonymisierung des Textes vor der KI-Analyse und Paket für die KI (Text, Anweisung, Metadaten); Entscheidungen des Ermittlers zu nicht erkannten Erwähnungen.
- Prüfung der KI-Antwort, menschliche Durchsicht, Import in die Datenbank (SQLite) als „ungeprüft“.
- Modell: Subjekte, Quellen mit Kette bis zur Ursprungsquelle, Aussagen mit Zuverlässigkeits- und Glaubwürdigkeitsnote, Eigentümerbaum.
- Ermittlerbericht als Markdown, Word und PDF, durch eine Passphrase geschützt.

## Download

Fertige Dateien für Windows (portables Zip ohne Installation, .msi/.exe-Installer) finden Sie auf der Seite der [Releases](https://github.com/RomanZavadaM/ZaNytky/releases/latest). Die Builds sind nicht signiert: Windows kann vor einem unbekannten Herausgeber warnen.

## Schnellstart (Anwendung)

1. Das portable Archiv entpacken und `ZaNytky.exe` starten (benötigt Microsoft Edge WebView2, in Windows 10/11 vorhanden).
2. Reiter „Einrichtung“: leeren Datenordner wählen und „Ordner vorbereiten“ drücken; Ermittlerschlüssel erzeugen und die Datei sicher aufbewahren.
3. Reiter „Dokument vorbereiten“: Text einfügen, „Anonymisieren und Paket für die KI erstellen“ drücken, Text und Anweisung kopieren und der KI geben.
4. Reiter „KI-Ergebnis“: Antwort (result.json) einfügen, prüfen, die Durchsicht lesen und importieren.
5. Reiter „Datenbank“ und „Bericht“: Aussagen ansehen, bestätigen oder widerlegen, Bericht erstellen.

## Kommandozeile

Dasselbe gibt es als Befehle; `zanytky help` listet alle auf.

```bash
zanytky init --home ./home
zanytky identity-new ./investigator.txt
zanytky register name "Іваненко Петро" --home ./home
zanytky prepare doc.md --home ./home --out ./pkg --id pkg-1 --title "Title" --identity ./investigator.txt
zanytky check  result.json --package pkg-1
zanytky import result.json --package pkg-1 --home ./home
zanytky report --home ./home --identity ./investigator.txt --out report.docx --format docx --passphrase-file ./pass.txt
```

## Sicherheit

Bewahren Sie den Systemschlüssel (`system.key` im Datenordner), den privaten Schlüssel des Ermittlers und die Passphrase getrennt auf und fügen Sie sie nie einem Repository hinzu. Die Anwendung sendet nichts über das Netzwerk.

## Dokumentation

- [Benutzerhandbuch](../user-guide/USER_GUIDE.de.md)
- [LICENSE](../../LICENSE.md)
