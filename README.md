# ZaNytky («За нитку»)

**🇺🇦 Українська** · [🇬🇧 English](docs/readme/README.en.md) · [🇫🇷 Français](docs/readme/README.fr.md) · [🇩🇪 Deutsch](docs/readme/README.de.md) · [🇪🇸 Español](docs/readme/README.es.md) · [🇰🇷 한국어](docs/readme/README.ko.md) · [🇯🇵 日本語](docs/readme/README.ja.md)

## Завантажити

**Остання версія: [ZaNytky v0.0.8](https://github.com/RomanZavadaM/ZaNytky/releases/tag/v0.0.23) (пререліз)**

| Платформа | Файл |
|---|---|
| Windows - інсталятор (рекомендовано) | [ZaNytky_0.0.23_Setup_Windows_x64.exe](https://github.com/RomanZavadaM/ZaNytky/releases/download/v0.0.23/ZaNytky_0.0.23_Setup_Windows_x64.exe) |
| Windows - MSI | [ZaNytky_0.0.23_Windows_x64.msi](https://github.com/RomanZavadaM/ZaNytky/releases/download/v0.0.23/ZaNytky_0.0.23_Windows_x64.msi) |
| Windows - портативний архів (лише для ознайомлення) | [ZaNytky_0.0.23_Windows_x64_Portable.zip](https://github.com/RomanZavadaM/ZaNytky/releases/download/v0.0.23/ZaNytky_0.0.23_Windows_x64_Portable.zip) |
| Windows - командний рядок | [ZaNytky_0.0.23_CLI_Windows_x64.zip](https://github.com/RomanZavadaM/ZaNytky/releases/download/v0.0.23/ZaNytky_0.0.23_CLI_Windows_x64.zip) |
| macOS (Apple Silicon) | [ZaNytky_0.0.23_macOS_arm64_Portable.zip](https://github.com/RomanZavadaM/ZaNytky/releases/download/v0.0.23/ZaNytky_0.0.23_macOS_arm64_Portable.zip) |
| macOS (Intel) | [ZaNytky_0.0.23_macOS_x86_64_Portable.zip](https://github.com/RomanZavadaM/ZaNytky/releases/download/v0.0.23/ZaNytky_0.0.23_macOS_x86_64_Portable.zip) |

- Контрольні суми - у файлі [SHA256SUMS.txt](https://github.com/RomanZavadaM/ZaNytky/releases/download/v0.0.23/SHA256SUMS.txt) та на [сторінці випуску](https://github.com/RomanZavadaM/ZaNytky/releases/tag/v0.0.23). Усі випуски: [Releases](https://github.com/RomanZavadaM/ZaNytky/releases).
- Збірки не підписані: Windows може показати попередження про невідомого видавця; macOS-збірки не нотаризовані й не перевірялись на реальному Mac.
- Портативний архів призначений лише для ознайомлення, без жодних гарантій; для постійної роботи використовуйте інсталятор.

Застосунок для впорядкування зв'язків між суб'єктами з відкритих джерел: джерела з повним текстом і файлами, твердження з цитатами, граф зв'язків, пошук шляху між суб'єктами. Персональні дані замінюються псевдонімами за ключем.

Статус: **ранній етап розробки** (пререліз 0.0.8). ZaNytky - **proprietary software**: публічність репозиторію не надає open-source ліцензії (LICENSE.md, COPYRIGHT.md, THIRD_PARTY_NOTICES.md). Дані користувача в репозиторії не зберігаються.

## Що вміє

- **Джерела:** повний текст і файли (скани, фото, відео) зберігаються в локальній базі; пошук за назвою, текстом і адресою.
- **Імпорт каналу Telegram** (експорт Telegram Desktop, JSON): кожен допис стає джерелом з текстом і вкладеннями.
- **Твердження з цитатою** з джерела: вид, сторони з ролями (наявні або нові суб'єкти), оцінка достовірності; завжди «неперевірене».
- **Суб'єкти:** картка з тверджень, джерел, ланцюжка власності й спільних ознак (адреса, телефон, номер авто, рахунок); злиття дублікатів; граф зв'язків; пошук найкоротшого зв'язку між двома суб'єктами.
- **Псевдонімізація:** однозначні коди за системним ключем; оригінали лише запечатані (age) на ключ розслідувача; знеособлення тексту й пакет для ШІ; перевірка відповіді ШІ й імпорт.
- **Звіт** у Markdown, Word і PDF з парольною фразою; зашифрована резервна копія папки даних. 7 мов інтерфейсу.

## Швидкий початок

1. Встановіть застосунок (або розпакуйте портативний архів і запустіть `ZaNytky.exe`).
2. «Налаштування»: оберіть порожню папку даних поза папкою з кодом, натисніть «Підготувати папку», створіть ключ розслідувача й збережіть його окремо.
3. «Джерела»: імпортуйте експорт каналу Telegram (кнопка угорі) або додайте текст і файли до джерела вручну.
4. Відкрийте джерело й створіть твердження: цитата, сторони, ролі. Переглядайте «Суб'єкти» та «Граф».
5. Для розбору ШІ: «Розбір документа» (знеособлення й пакет) і «Результат ШІ» (перевірка й імпорт). «Звіт» - підсумок.

## Командний рядок

Те саме доступно командами; `zanytky help` показує всі.

```bash
zanytky init --home ./home
zanytky identity-new ./investigator.txt
zanytky import-telegram "ChatExport_2026-11-01" --home ./home --reliability F
zanytky import result.json --package pkg-1 --home ./home
zanytky report --home ./home --identity ./investigator.txt --out report.docx --format docx --passphrase-file ./pass.txt
zanytky backup --home ./home --out copy.znbak --passphrase-file ./pass.txt
```

## Принципи

- Кожне твердження має джерело й ланцюжок до першоджерела; чутка не стає фактом, а відлуння однієї чутки - це одне першоджерело.
- Жодних оцінкових ярликів про людей: показуємо походження й оцінку надійності, а не вердикти.
- Лише відкриті джерела. Тексти й файли зберігаються локально, без знеособлення; ШІ отримує тільки знеособлений текст.

## Обмеження поточної версії (0.0.23)

- Пререліз: інсталятори не підписані.
- Імпорту PDF і розпізнавання тексту зі сканів та відео ще немає; цитату для твердження вставляють вручну.
- Звіт Word і перегляд PDF не перевірялись у Microsoft Word і PDF-переглядачах; розпізнавання невідомих імен просте.
- Тримайте папку даних поза папкою з кодом і не вписуйте особисті дані в ідентифікатор пакета: він передається ШІ без знеособлення.

## Безпека

Системний ключ (`system.key` у папці даних), приватний ключ розслідувача і парольну фразу зберігайте окремо й ніколи не додавайте в репозиторій. Застосунок нічого не надсилає в мережу.

## Документація

- [Керівництво користувача](docs/user-guide/USER_GUIDE.uk.md)
- [Нотатки випусків](docs/releases)
- [LICENSE](LICENSE.md)
