# ZaNytky - guide d'utilisation

[🇺🇦 Українська](USER_GUIDE.uk.md) · [🇬🇧 English](USER_GUIDE.en.md) · **🇫🇷 Français** · [🇩🇪 Deutsch](USER_GUIDE.de.md) · [🇪🇸 Español](USER_GUIDE.es.md) · [🇰🇷 한국어](USER_GUIDE.ko.md) · [🇯🇵 日本語](USER_GUIDE.ja.md)

Ce guide décrit la fenêtre de l'application. La version ukrainienne fait foi.

> Cette traduction est automatique et n'a pas été relue par des locuteurs natifs ; le texte ukrainien fait foi.

## 1. Configuration

- Choisissez un **dossier de données** (de préférence vide) et cliquez sur « Préparer le dossier ». Il reçoit la clé système `system.key` et le fichier `recipients.txt` contenant les clés publiques des enquêteurs.
- Créez une **clé d'enquêteur** : l'application écrit la clé privée dans un fichier de votre choix et ajoute la clé publique à `recipients.txt`. La clé privée sert à révéler les noms et pour les rapports ; sans elle, les données ne peuvent pas être révélées. Gardez le fichier en lieu sûr et faites-en une copie.

## 2. Préparer un document

- Indiquez un titre, le type de source (média, registre, déclaration d'une autorité, témoignage, rumeur, propre), éventuellement une adresse, et un identifiant de paquet (unique pour chaque document).
- Collez le texte ou ouvrez un fichier. Si besoin, ajoutez le **contexte de l'enquêteur**, par exemple « Ivanov I. dirige la société Exemple ».
- Cliquez sur « Anonymiser et préparer le paquet pour l'IA ». L'application remplace les noms, numéros fiscaux, téléphones et e-mails connus par des pseudonymes. Copiez le texte anonymisé et l'instruction et donnez-les à l'IA.
- Dans « Mentions non reconnues par le système », choisissez pour chaque mention : **même personne** (indiquez le pseudonyme), **nouvelle personne** (nom complet, éventuellement numéro fiscal, téléphone, e-mail) ou **pas une personne**. Refaites ensuite le paquet.

## 3. Résultat de l'IA

- Collez la réponse de l'IA (fichier result.json) et l'identifiant du paquet. Cliquez sur « Vérifier » : l'application contrôle la structure et les règles et affiche une relecture avec les citations. Ce n'est qu'après la relecture que vous cliquez sur « Importer dans la base ».
- Tout est importé comme « non vérifié ». Un paquet ne peut être importé qu'une fois.

## 4. Base

- Liste des déclarations avec une note (par exemple B3 : fiabilité de la source B, crédibilité 3) et un statut. Les personnes ne sont montrées que par pseudonyme.
- « Confirmer » n'est possible que s'il existe une source qui n'est ni une rumeur ni un témoignage ; sinon l'application refuse. « Réfuter » marque la déclaration comme réfutée (elle n'est pas supprimée).

## 5. Rapport

- Choisissez un format (Word, PDF, Markdown) et saisissez une phrase secrète. Le rapport révèle les personnes avec votre clé privée ; il contient donc des données personnelles en clair. Sans phrase secrète, un rapport n'est créé qu'après confirmation explicite « Sans phrase secrète ».
- Un rapport protégé a l'extension `.age` ; ouvrez-le avec [age](https://age-encryption.org) et votre phrase secrète.

## 6. Sources et canaux Telegram

- **Importer un canal.** Dans Telegram Desktop : canal → « Exporter l'historique » → format JSON. Dans l'onglet « Sources », « Choisir le dossier et importer » et choisissez le dossier contenant `result.json`. Chaque message devient une source avec son texte intégral ; photos, vidéos et fichiers deviennent des pièces jointes. L'import peut être répété : pas de doublons. La fiabilité des sources est F (inconnue) par défaut.
- **Sources.** Recherche par titre, texte et adresse (débuts de mots, toute casse, tous les mots doivent figurer). Dans une source ouverte : le texte (coller, enregistrer, copier), « Ajouter un fichier », « Afficher dans le dossier », « Afficher l'image ».
- **Une déclaration à partir d'une source.** Type, valeur, crédibilité 1-6, **citation obligatoire**, 2 à 4 parties avec rôles (sujets existants ou nouveaux). Une nouvelle personne se saisit par son nom complet : le nom va au registre scellé, seul un pseudonyme reste dans la base. Une déclaration est toujours « non vérifiée ».
- **Découverte automatique de sujets.** « Trouver les sujets dans le texte » liste personnes, organisations (SARL « ... ») et codes de registre, marqués « dans la base » ou « nouveau », avec « Ajouter à la base ». « Nouveaux sujets dans tous les textes » donne le haut des inconnus. Ce sont des indices : confirmez à la main.
- **Canaux.** Chronologie par mois avec pics d'activité, numéros de messages absents de l'exportation (peut-être supprimés), graphe du canal : d'où il transfère, vers quoi il renvoie, qui il mentionne. Un clic sur un nœud le développe. Pour remplir l'analyse d'un canal importé auparavant, réimportez son dossier.
- **Recoupements** (au moins deux canaux) : lignes avec une barre par canal - source de transferts, canal, site, mention, long texte identique ; filtre par type ; graphe sur demande. Les plateformes génériques (YouTube, Facebook...) sont exclues.
- **Aperçu.** Un bouton écrit un rapport sur toute la base : canaux, pics, recoupements, nouveaux sujets et **liens probables** (paires de sujets nommés dans le même paragraphe dans au moins 3 messages, avec une citation). Enregistrement en `.md` ; en ligne de commande : `zanytky overview --home DOSSIER --out FICHIER`.
- **Important.** Recoupements, pics, découverte et aperçu sont des indices pour des hypothèses, pas des faits. Les textes et fichiers sont conservés sans anonymisation, uniquement dans la base locale ; gardez le dossier de données hors des dossiers de code et faites des sauvegardes.

## 7. Preuves, graphe et ligne de commande

- **Ajouter une preuve.** Dans la fiche d'un sujet, à côté d'une déclaration : type de document (registre, déclaration officielle, média, témoignage, propre), fiabilité A-F, titre, adresse, date, texte ou extrait, note. « Ajouter et confirmer » n'existe que pour « registre » et « déclaration officielle » ; « Ajouter et réfuter » pour tout type. Une même adresse n'est pas dupliquée.
- **Graphe.** Onglet « Graphe » : la couleur du trait est le type de lien (propriété, poste, lien rapporté...), trait épais plein = confirmé par un document, pointillés = pas encore vérifié ; les nœuds sont étiquetés, les noms des personnes s'affichent si la clé de l'enquêteur est indiquée. « Enregistrer le graphe en HTML... » écrit un fichier autonome (sans internet, avec recherche) : tout le graphe ou seulement autour du sujet n° (1 à 3 pas), éventuellement les confirmés seuls.
- **Ligne de commande pour l'analyse.** `search REQUÊTE`, `source NUMÉRO --full`, `discover --source N`, `channels`, `overlaps`, `pairs`, `entity N`, `statements --grep`, `add-source`, `add-entity`, `make-statements --file FICHIER.json` (la citation de chaque déclaration doit figurer mot pour mot dans le texte de la source, sinon rien n'est écrit), `evidence`, `graph --out FICHIER.html [--entity N --depth K] [--confirmed]`. Chaque commande de lecture accepte `--json`.
- **Corriger les erreurs.** `detach --statement N --source M [--drop-source]` détache une preuve ajoutée par erreur ; `delete-statement N` et `delete-source N` suppriment une mauvaise déclaration ou une source inutilisée ; `merge --keep A --drop B` fusionne des doublons (pas des personnes), `unmerge --drop B` l'annule ; `evidence --expect MOTS` empêche d'écrire vers un mauvais numéro. La suppression est irréversible (journalisée) : faites d'abord `backup`.

## Sécurité

- Conservez séparément la clé système, la clé privée de l'enquêteur et la phrase secrète. Perdre la clé privée signifie que les données ne pourront plus être révélées.
- L'application n'envoie rien sur le réseau. Le texte que vous donnez à l'IA quitte votre ordinateur : ne soumettez que des documents issus de sources ouvertes.

## FAQ

- Windows signale un éditeur inconnu : la build n'est pas signée ; choisissez « Informations complémentaires » puis « Exécuter quand même ».
- Fenêtre vide ou erreur au démarrage : vérifiez que le dossier `webview2` est à côté de `ZaNytky.exe` (l'archive portable n'exige aucune installation ).

## Scénario d'essai (données fictives)

1. Onglet « Configuration » : choisissez un dossier vide, cliquez sur « Préparer le dossier », créez une clé d'enquêteur.
2. Onglet « Préparer le document » : collez le texte de `docs/examples/sample-document.md`, titre « Document d'essai », identifiant `pkg-1`, cliquez sur « Anonymiser et préparer le paquet ».
3. Dans la liste des mentions non reconnues, choisissez « Nouvelle personne » pour « Петро Іваненко », saisissez le nom complet, le numéro fiscal 1234567890 et le téléphone +38 050 123 45 67, cliquez sur « Appliquer ». Refaites le paquet : le nom, le numéro fiscal et le téléphone sont maintenant remplacés par un seul pseudonyme.
4. Copiez l'instruction et le texte anonymisé dans un chat d'IA et demandez de répondre uniquement en JSON. Enregistrez la réponse.
5. Onglet « Résultat de l'IA » : collez la réponse, cliquez sur « Vérifier », lisez la relecture puis « Importer dans la base ».
6. Onglet « Base » : parcourez les déclarations ; essayez « Confirmer » (pour une déclaration issue d'une rumeur, l'application refuse).
7. Onglet « Rapport » : créez un rapport Word avec une phrase secrète ; ouvrez-le avec age et vérifiez qu'il contient le vrai nom.
