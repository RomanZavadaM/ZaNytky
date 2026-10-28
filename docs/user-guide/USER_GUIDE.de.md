# ZaNytky - Benutzerhandbuch

[🇺🇦 Українська](USER_GUIDE.uk.md) · [🇬🇧 English](USER_GUIDE.en.md) · [🇫🇷 Français](USER_GUIDE.fr.md) · **🇩🇪 Deutsch** · [🇪🇸 Español](USER_GUIDE.es.md) · [🇰🇷 한국어](USER_GUIDE.ko.md) · [🇯🇵 日本語](USER_GUIDE.ja.md)

Dieses Handbuch beschreibt das Anwendungsfenster. Maßgeblich ist die ukrainische Fassung.

> Diese Übersetzung ist maschinell erstellt und nicht von Muttersprachlern geprüft; maßgeblich ist der ukrainische Text.

## 1. Einrichtung

- Wählen Sie einen **Datenordner** (am besten leer) und drücken Sie „Ordner vorbereiten“. Darin entstehen der Systemschlüssel `system.key` und die Datei `recipients.txt` mit den öffentlichen Schlüsseln der Ermittler.
- Erzeugen Sie einen **Ermittlerschlüssel**: Die Anwendung schreibt den privaten Schlüssel in eine von Ihnen gewählte Datei und fügt den öffentlichen Schlüssel zu `recipients.txt` hinzu. Der private Schlüssel wird zum Offenlegen von Namen und für Berichte benötigt; ohne ihn lassen sich die Daten nicht offenlegen. Bewahren Sie die Datei sicher auf und fertigen Sie eine Kopie an.

## 2. Dokument vorbereiten

- Geben Sie Titel, Art der Quelle (Medien, Register, Behördenerklärung, Zeugenaussage, Gerücht, eigene), optional eine Adresse und eine Paket-ID (je Dokument eindeutig) an.
- Fügen Sie den Text ein oder öffnen Sie eine Datei. Bei Bedarf ergänzen Sie den **Kontext des Ermittlers**, z. B. „Iwanow I. leitet die Beispiel GmbH“.
- Drücken Sie „Anonymisieren und Paket für die KI erstellen“. Die Anwendung ersetzt bekannte Namen, Steuernummern, Telefone und E-Mails durch Pseudonyme. Kopieren Sie den anonymisierten Text und die Anweisung und geben Sie sie der KI.
- Wählen Sie unter „Vom System nicht erkannte Erwähnungen“ je Erwähnung: **dieselbe Person** (Pseudonym angeben), **neue Person** (vollständiger Name, optional Steuernummer, Telefon, E-Mail) oder **keine Person**. Erstellen Sie danach das Paket erneut.

## 3. KI-Ergebnis

- Fügen Sie die Antwort der KI (Datei result.json) und die Paket-ID ein. Drücken Sie „Prüfen“: Die Anwendung prüft Struktur und Regeln und zeigt eine Durchsicht mit Zitaten. Erst nach der Durchsicht drücken Sie „In die Datenbank importieren“.
- Alles wird als „ungeprüft“ importiert. Ein Paket kann nur einmal importiert werden.

## 4. Datenbank

- Liste der Aussagen mit Bewertung (z. B. B3: Zuverlässigkeit der Quelle B, Glaubwürdigkeit 3) und Status. Personen werden nur als Pseudonym angezeigt.
- „Bestätigen“ ist nur möglich, wenn es eine Quelle gibt, die weder Gerücht noch Zeugenaussage ist; sonst lehnt die Anwendung ab. „Widerlegen“ markiert die Aussage als widerlegt (sie wird nicht gelöscht).

## 5. Bericht

- Wählen Sie ein Format (Word, PDF, Markdown) und geben Sie eine Passphrase ein. Der Bericht legt Personen mit Ihrem privaten Schlüssel offen und enthält daher offene personenbezogene Daten. Ohne Passphrase wird ein Bericht nur nach ausdrücklicher Bestätigung „Ohne Passphrase“ erstellt.
- Ein geschützter Bericht hat die Endung `.age`; öffnen Sie ihn mit [age](https://age-encryption.org) und Ihrer Passphrase.

## Sicherheit

- Bewahren Sie Systemschlüssel, privaten Schlüssel des Ermittlers und Passphrase getrennt auf. Geht der private Schlüssel verloren, lassen sich die Daten nicht mehr offenlegen.
- Die Anwendung sendet nichts über das Netzwerk. Der Text, den Sie der KI geben, verlässt Ihren Computer; reichen Sie daher nur Material aus offenen Quellen ein.

## Häufige Fragen

- Windows warnt vor einem unbekannten Herausgeber: Der Build ist nicht signiert; wählen Sie „Weitere Informationen“ und „Trotzdem ausführen“.
- Leeres Fenster oder Startfehler: Microsoft Edge WebView2 wird benötigt (in Windows 10/11 vorhanden).

## Probedurchlauf (fiktive Daten)

1. Reiter „Einrichtung“: leeren Ordner wählen, „Ordner vorbereiten“ drücken, Ermittlerschlüssel erzeugen.
2. Reiter „Dokument vorbereiten“: Text aus `docs/examples/sample-document.md` einfügen, Titel „Probedokument“, Paket-ID `pkg-1`, „Anonymisieren und Paket erstellen“ drücken.
3. In der Liste der nicht erkannten Erwähnungen für „Петро Іваненко“ „Neue Person“ wählen, vollständigen Namen, Steuernummer 1234567890 und Telefon +38 050 123 45 67 eingeben, „Anwenden“ drücken. Paket erneut erstellen: Name, Steuernummer und Telefon sind nun durch ein Pseudonym ersetzt.
4. Anweisung und anonymisierten Text in einen KI-Chat kopieren und um eine reine JSON-Antwort bitten. Antwort speichern.
5. Reiter „KI-Ergebnis“: Antwort einfügen, „Prüfen“ drücken, Durchsicht lesen und „In die Datenbank importieren“ drücken.
6. Reiter „Datenbank“: Aussagen ansehen; „Bestätigen“ ausprobieren (bei einer Aussage aus einer Gerüchtequelle lehnt die Anwendung ab).
7. Reiter „Bericht“: Word-Bericht mit Passphrase erstellen; mit age öffnen und prüfen, dass er den echten Namen enthält.
