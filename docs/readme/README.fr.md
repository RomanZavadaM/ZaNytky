# ZaNytky («За нитку»)

[🇺🇦 Українська](../../README.md) · [🇬🇧 English](README.en.md) · **🇫🇷 Français** · [🇩🇪 Deutsch](README.de.md) · [🇪🇸 Español](README.es.md) · [🇰🇷 한국어](README.ko.md) · [🇯🇵 日本語](README.ja.md)

## Téléchargement

**Dernière version : [ZaNytky v0.0.23](https://github.com/RomanZavadaM/ZaNytky/releases/tag/v0.0.25) (préversion)**

| Plateforme | Fichier |
|---|---|
| Windows - installateur (recommandé) | [ZaNytky_0.0.25_Setup_Windows_x64.exe](https://github.com/RomanZavadaM/ZaNytky/releases/download/v0.0.25/ZaNytky_0.0.25_Setup_Windows_x64.exe) |
| Windows - MSI | [ZaNytky_0.0.25_Windows_x64.msi](https://github.com/RomanZavadaM/ZaNytky/releases/download/v0.0.25/ZaNytky_0.0.25_Windows_x64.msi) |
| Windows - archive portable (évaluation seulement) | [ZaNytky_0.0.25_Windows_x64_Portable.zip](https://github.com/RomanZavadaM/ZaNytky/releases/download/v0.0.25/ZaNytky_0.0.25_Windows_x64_Portable.zip) |
| Windows - ligne de commande | [ZaNytky_0.0.25_CLI_Windows_x64.zip](https://github.com/RomanZavadaM/ZaNytky/releases/download/v0.0.25/ZaNytky_0.0.25_CLI_Windows_x64.zip) |
| macOS (Apple Silicon) | [ZaNytky_0.0.25_macOS_arm64_Portable.zip](https://github.com/RomanZavadaM/ZaNytky/releases/download/v0.0.25/ZaNytky_0.0.25_macOS_arm64_Portable.zip) |
| macOS (Intel) | [ZaNytky_0.0.25_macOS_x86_64_Portable.zip](https://github.com/RomanZavadaM/ZaNytky/releases/download/v0.0.25/ZaNytky_0.0.25_macOS_x86_64_Portable.zip) |

- Sommes de contrôle : [SHA256SUMS.txt](https://github.com/RomanZavadaM/ZaNytky/releases/download/v0.0.25/SHA256SUMS.txt) et [page de la version](https://github.com/RomanZavadaM/ZaNytky/releases/tag/v0.0.25). Toutes les versions : [Releases](https://github.com/RomanZavadaM/ZaNytky/releases).
- Les builds ne sont pas signés : Windows peut avertir d'un éditeur inconnu ; les builds macOS ne sont pas notarisés et n'ont pas été testés sur un vrai Mac.
- L'archive portable sert uniquement à l'évaluation, sans aucune garantie ; pour un usage régulier, utilisez l'installateur.

Application pour organiser les liens entre sujets à partir de sources ouvertes : sources avec texte intégral et fichiers, déclarations avec citations, graphe des liens, recherche du chemin le plus court entre sujets. Les données personnelles sont remplacées par des pseudonymes à clé.

Statut : **développement précoce** (préversion 0.0.23). ZaNytky est un **logiciel propriétaire** : un dépôt public n'accorde pas de licence open source (LICENSE.md, COPYRIGHT.md, THIRD_PARTY_NOTICES.md). Aucune donnée utilisateur n'est stockée dans le dépôt.

> Cette traduction est automatique et n'a pas été relue par des locuteurs natifs ; le texte ukrainien fait foi.

## Fonctions

- **Sources :** texte intégral et fichiers (scans, photos, vidéo) dans la base locale ; recherche par titre, texte et adresse.
- **Import d'un canal Telegram** (export Telegram Desktop, JSON) : chaque message devient une source avec texte et pièces jointes.
- **Déclarations avec citation** de la source : type, parties avec rôles (sujets existants ou nouveaux), niveau de crédibilité ; toujours « non vérifiée ».
- **Sujets :** fiche avec déclarations, sources, chaîne de propriété et signes communs (adresse, téléphone, plaque, compte) ; fusion des doublons ; graphe ; chemin le plus court entre deux sujets.
- **Pseudonymisation :** codes uniques à clé système ; originaux uniquement scellés (age) pour la clé de l'enquêteur ; anonymisation du texte et paquet pour l'IA ; contrôle et import de la réponse.
- **Rapport** Markdown, Word et PDF avec phrase secrète ; sauvegarde chiffrée du dossier de données. 7 langues.
- **Preuves et vérification :** un document est joint à une déclaration (« Ajouter une preuve » : registre, déclaration officielle, média, témoignage, propre) ; « confirmé » n'est posé que par un document de registre ou de déclaration officielle, tout type peut réfuter.
- **Analyse de la base :** recherche rapide dans les sources, l'aperçu, paires de sujets dans un même paragraphe, recoupements entre canaux, découverte automatique de sujets ; le graphe (entier ou en partie) s'enregistre en un fichier HTML autonome.
- **Ligne de commande pour l'analyse automatisée** (search, source, discover, channels, overlaps, pairs, entity, statements, add-entity, add-source, make-statements, evidence, detach, delete-statement, delete-source, merge, unmerge, graph) : lecture avec `--json` ; `make-statements` n'écrit un lot de déclarations que si la citation de chacune figure dans le texte de sa source ; les erreurs se corrigent par commandes, pas à la main.

## Démarrage rapide

1. Installez l'application (ou décompressez l'archive portable et lancez `ZaNytky.exe`).
2. « Paramètres » : choisissez un dossier de données vide hors du dossier du code, « Préparer le dossier », créez la clé de l'enquêteur et gardez-la à part.
3. « Sources » : importez l'export d'un canal Telegram (bouton en haut) ou ajoutez texte et fichiers à la main.
4. Ouvrez une source et créez une déclaration : citation, parties, rôles. Consultez « Sujets » et « Graphe ».
5. Pour l'analyse par IA : « Document » (anonymisation et paquet) et « Résultat IA » (contrôle et import). « Rapport » donne la synthèse.

## Ligne de commande

Les mêmes fonctions existent en commandes ; `zanytky help` les liste.

```bash
zanytky init --home ./home
zanytky identity-new ./investigator.txt
zanytky import-telegram "ChatExport_2026-11-01" --home ./home --reliability F
zanytky import result.json --package pkg-1 --home ./home
zanytky report --home ./home --identity ./investigator.txt --out report.docx --format docx --passphrase-file ./pass.txt
zanytky backup --home ./home --out copy.znbak --passphrase-file ./pass.txt
```

Analyse et correction des erreurs par commande :

```bash
zanytky search "Приклад" --home ./home
zanytky source 123 --full --home ./home
zanytky discover --source 123 --home ./home
zanytky make-statements --file statements.json --home ./home
zanytky evidence --statement 5 --kind registry --title "Витяг" --url https://example.org/x --status confirmed --home ./home
zanytky graph --out graph.html --entity 32 --depth 2 --home ./home
```

`--expect` dans `evidence` empêche d'attacher une preuve à une mauvaise déclaration ; `detach`, `delete-statement`, `delete-source` et `merge` corrigent les erreurs (la suppression est irréversible, faites `backup` avant les changements en masse).

## Principes

- Chaque déclaration a une source et une chaîne jusqu'à la source première ; une rumeur ne devient pas un fait, et les échos d'une rumeur ne forment qu'une source première.
- Aucune étiquette de jugement sur les personnes : nous montrons l'origine et la fiabilité, pas des verdicts.
- Sources ouvertes uniquement. Textes et fichiers sont conservés localement, sans anonymisation ; l'IA ne reçoit que du texte anonymisé.

## Limites de la version actuelle (0.0.25)

- Préversion : les installateurs ne sont pas signés.
- Pas d'import PDF ni de reconnaissance de texte des scans ou vidéos ; la citation d'une déclaration se colle à la main.
- Le rapport Word et l'affichage PDF n'ont pas été vérifiés dans Microsoft Word ni des lecteurs PDF ; la reconnaissance des noms inconnus est simple.
- Gardez le dossier de données hors du dossier du code et ne mettez pas de données personnelles dans l'identifiant du paquet : il est transmis à l'IA sans anonymisation.

## Sécurité

Conservez séparément la clé système (`system.key` dans le dossier de données), la clé privée de l'enquêteur et la phrase secrète, et ne les ajoutez jamais au dépôt. L'application n'envoie rien sur le réseau.

## Documentation

- [Guide de l'utilisateur](../../docs/user-guide/USER_GUIDE.fr.md)
- [Notes de version](../../docs/releases)
- [LICENSE](../../LICENSE.md)
