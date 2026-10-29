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
