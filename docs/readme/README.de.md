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

**Herunterladen: [ZaNytky v0.0.4](https://github.com/RomanZavadaM/ZaNytky/releases/tag/v0.0.4)** (aktuelle Vorabversion)

Fertige Dateien für Windows (portables Zip ohne Installation, .msi/.exe-Installer) finden Sie auf der Seite der [Releases](https://github.com/RomanZavadaM/ZaNytky/releases). Die Builds sind nicht signiert: Windows kann vor einem unbekannten Herausgeber warnen.

## Schnellstart (Anwendung)

1. Das portable Archiv entpacken und `ZaNytky.exe` starten (nichts muss installiert werden; den Ordner `webview2` nicht von `ZaNytky.exe` trennen).
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
zanytky backup --home ./home --out copy.znbak --passphrase-file ./pass.txt
zanytky restore copy.znbak --to ./restored --passphrase-file ./pass.txt
```

## Grundsätze

- Jede Aussage hat eine Quelle und eine Kette bis zur Primärquelle; ein Gerücht wird nicht zur Tatsache, und Echos desselben Gerüchts zählen als eine Primärquelle.
- Keine wertenden Etiketten über Personen: Das Programm zeigt Herkunft und eine Zuverlässigkeitsstufe, keine Urteile.
- Nur offene Quellen; personenbezogene Daten werden durch Pseudonyme ersetzt; die KI erhält nur anonymisierten Text.

## Einschränkungen der aktuellen Version (0.0.4)

- Dies ist eine Vorabversion: Die Installationsprogramme sind nicht signiert.
- „Dokumentanalyse“ akzeptiert derzeit nur Text und Markdown (noch kein .docx/.pdf-Import).
- Der Word-Bericht wurde nicht in Microsoft Word und das PDF nicht in einem Viewer geprüft; die Erkennung unbekannter Namen ist einfach (großgeschriebene Wörter) und liefert falsche Kandidaten.
- Halten Sie den Datenordner außerhalb des Code-Ordners und tragen Sie keine personenbezogenen Daten (z. B. eine E-Mail-Adresse) in die Paketkennung ein: Sie wird ohne Anonymisierung an die KI weitergegeben.

## Sicherheit

Bewahren Sie den Systemschlüssel (`system.key` im Datenordner), den privaten Schlüssel des Ermittlers und die Passphrase getrennt auf und fügen Sie sie nie einem Repository hinzu. Die Anwendung sendet nichts über das Netzwerk.

## Dokumentation

- [Benutzerhandbuch](../user-guide/USER_GUIDE.de.md)
- [LICENSE](../../LICENSE.md)
