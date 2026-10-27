# ZaNytky («За нитку»)

[🇺🇦 Українська](../../README.md) · [🇬🇧 English](README.en.md) · **🇫🇷 Français** · [🇩🇪 Deutsch](README.de.md) · [🇪🇸 Español](README.es.md) · [🇰🇷 한국어](README.ko.md) · [🇯🇵 日本語](README.ja.md)

Application pour organiser les liens entre des sujets issus de sources ouvertes, avec pseudonymisation des données personnelles par clé.

Statut : **début de développement** (préversion). Implémentation : Rust (cœur) et React (fenêtre). L'ukrainien est la langue de référence de l'interface et de la documentation.

ZaNytky est un **logiciel propriétaire** : un dépôt public n'accorde pas de licence open source (voir LICENSE.md, COPYRIGHT.md, THIRD_PARTY_NOTICES.md). Aucune donnée d'utilisateur n'est stockée dans le dépôt.

> Cette traduction est automatique et n'a pas été relue par des locuteurs natifs ; le texte ukrainien fait foi.

## Fonctionnalités

- Pseudonymes univoques des valeurs (noms, numéros fiscaux, téléphones, e-mails) selon une clé système : la même valeur donne toujours le même code.
- Les originaux ne sont conservés que scellés pour la clé publique de l'enquêteur (age) ; seul le détenteur de la clé privée peut les ouvrir.
- Anonymisation du texte avant l'analyse par l'IA et paquet pour l'IA (texte, instruction, métadonnées) ; décisions de l'enquêteur sur les mentions non reconnues.
- Vérification de la réponse de l'IA, relecture humaine, import dans la base (SQLite) comme « non vérifié ».
- Modèle : sujets, sources avec chaîne jusqu'à la source d'origine, déclarations avec notes de fiabilité et de crédibilité, arbre de propriété.
- Rapport de l'enquêteur en Markdown, Word et PDF, protégé par une phrase secrète.

## Téléchargement

Les fichiers prêts pour Windows (zip portable sans installation, installateurs .msi/.exe) sont sur la page des [versions](https://github.com/RomanZavadaM/ZaNytky/releases/latest). Les builds ne sont pas signées : Windows peut avertir d'un éditeur inconnu.

## Démarrage rapide (application)

1. Décompressez l'archive portable et lancez `ZaNytky.exe` (nécessite Microsoft Edge WebView2, présent dans Windows 10/11).
2. Onglet « Configuration » : choisissez un dossier de données vide et cliquez sur « Préparer le dossier » ; créez une clé d'enquêteur et gardez son fichier en lieu sûr.
3. Onglet « Préparer le document » : collez le texte, cliquez sur « Anonymiser et préparer le paquet pour l'IA », copiez le texte et l'instruction et donnez-les à l'IA.
4. Onglet « Résultat de l'IA » : collez la réponse (result.json), vérifiez-la, lisez la relecture et importez.
5. Onglets « Base » et « Rapport » : parcourez les déclarations, confirmez ou réfutez-les, créez un rapport.

## Ligne de commande

Les mêmes actions existent en commandes ; `zanytky help` les liste toutes.

```bash
zanytky init --home ./home
zanytky identity-new ./investigator.txt
zanytky register name "Іваненко Петро" --home ./home
zanytky prepare doc.md --home ./home --out ./pkg --id pkg-1 --title "Title" --identity ./investigator.txt
zanytky check  result.json --package pkg-1
zanytky import result.json --package pkg-1 --home ./home
zanytky report --home ./home --identity ./investigator.txt --out report.docx --format docx --passphrase-file ./pass.txt
```

## Sécurité

Conservez séparément la clé système (`system.key` dans le dossier de données), la clé privée de l'enquêteur et la phrase secrète, et ne les ajoutez jamais à un dépôt. L'application n'envoie rien sur le réseau.

## Documentation

- [Guide d'utilisation](../user-guide/USER_GUIDE.fr.md)
- [LICENSE](../../LICENSE.md)
