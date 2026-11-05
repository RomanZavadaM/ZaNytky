# ZaNytky («За нитку»)

[🇺🇦 Українська](../../README.md) · [🇬🇧 English](README.en.md) · [🇫🇷 Français](README.fr.md) · [🇩🇪 Deutsch](README.de.md) · [🇪🇸 Español](README.es.md) · [🇰🇷 한국어](README.ko.md) · **🇯🇵 日本語**

## ダウンロード

**最新版：[ZaNytky v0.0.23](https://github.com/RomanZavadaM/ZaNytky/releases/tag/v0.0.24)（プレリリース）**

| プラットフォーム | ファイル |
|---|---|
| Windows - インストーラー（推奨） | [ZaNytky_0.0.24_Setup_Windows_x64.exe](https://github.com/RomanZavadaM/ZaNytky/releases/download/v0.0.24/ZaNytky_0.0.24_Setup_Windows_x64.exe) |
| Windows - MSI | [ZaNytky_0.0.24_Windows_x64.msi](https://github.com/RomanZavadaM/ZaNytky/releases/download/v0.0.24/ZaNytky_0.0.24_Windows_x64.msi) |
| Windows - ポータブル版（お試し用） | [ZaNytky_0.0.24_Windows_x64_Portable.zip](https://github.com/RomanZavadaM/ZaNytky/releases/download/v0.0.24/ZaNytky_0.0.24_Windows_x64_Portable.zip) |
| Windows - コマンドライン | [ZaNytky_0.0.24_CLI_Windows_x64.zip](https://github.com/RomanZavadaM/ZaNytky/releases/download/v0.0.24/ZaNytky_0.0.24_CLI_Windows_x64.zip) |
| macOS（Apple Silicon） | [ZaNytky_0.0.24_macOS_arm64_Portable.zip](https://github.com/RomanZavadaM/ZaNytky/releases/download/v0.0.24/ZaNytky_0.0.24_macOS_arm64_Portable.zip) |
| macOS（Intel） | [ZaNytky_0.0.24_macOS_x86_64_Portable.zip](https://github.com/RomanZavadaM/ZaNytky/releases/download/v0.0.24/ZaNytky_0.0.24_macOS_x86_64_Portable.zip) |

- チェックサム：[SHA256SUMS.txt](https://github.com/RomanZavadaM/ZaNytky/releases/download/v0.0.24/SHA256SUMS.txt) と[リリースページ](https://github.com/RomanZavadaM/ZaNytky/releases/tag/v0.0.24)。すべてのリリース：[Releases](https://github.com/RomanZavadaM/ZaNytky/releases)。
- ビルドは署名されていません。Windows が不明な発行元の警告を出すことがあります。macOS ビルドは公証されておらず、実機の Mac では未検証です。
- ポータブル版はお試し用で、いかなる保証もありません。継続利用にはインストーラーを使ってください。

公開情報源から対象間の関係を整理するアプリケーション：全文とファイルを持つ情報源、引用付きの主張、関係グラフ、対象間の最短経路検索。個人データはキーに基づく仮名に置き換えられます。

ステータス：**初期開発段階**（プレリリース 0.0.23）。ZaNytky は**プロプライエタリソフトウェア**です。リポジトリが公開されていてもオープンソースライセンスは付与されません（LICENSE.md、COPYRIGHT.md、THIRD_PARTY_NOTICES.md）。リポジトリにユーザーデータは保存されません。

> この翻訳は機械翻訳で、ネイティブによる確認を受けていません。正文はウクライナ語です。

## できること

- **情報源：**全文とファイル（スキャン、写真、動画）をローカルデータベースに保存し、タイトル・本文・アドレスで検索します。
- **Telegram チャンネルの取り込み**（Telegram Desktop のエクスポート、JSON）：各投稿が本文と添付を持つ情報源になります。
- **情報源の引用付きの主張：**種類、役割を持つ当事者（既存または新規の対象）、信頼性の評価。常に「未検証」です。
- **対象：**主張、情報源、所有の連鎖、共通の特徴（住所、電話、車両番号、口座）を持つカード。重複の統合、関係グラフ、2 つの対象間の最短経路。
- **仮名化：**システムキーによる一対一のコード。原本は調査者のキーでのみ封印（age）。テキストの匿名化と AI 用パッケージ、AI 回答の検査と取り込み。
- **レポート：**Markdown、Word、PDF（パスフレーズ保護）。データフォルダーの暗号化バックアップ。7 言語。
- **証拠と検証：**主張に文書を添付します（「証拠を追加」：登記、機関の声明、メディア、証言、自前）。「確認済み」は登記または機関の文書でのみ設定され、反証はどの種類でも可能です。
- **データベースの分析：**情報源の高速検索、概要、同じ段落に出る主体のペア、チャンネル間の重なり、主体の自動検出。グラフ（全体または一部）は単独で開ける HTML ファイル 1 つとして保存できます。
- **自動分析用のコマンドライン**（search, source, discover, channels, overlaps, pairs, entity, statements, add-entity, add-source, make-statements, evidence, detach, delete-statement, delete-source, merge, unmerge, graph）：`--json` で読み取り。`make-statements` は各主張の引用が情報源の本文にある場合にのみ主張の一括を書き込みます。誤りは手作業ではなくコマンドで直します。

## クイックスタート

1. アプリケーションをインストールします（またはポータブル版を展開して `ZaNytky.exe` を実行）。
2. 「設定」：コードフォルダーの外にある空のデータフォルダーを選び、「フォルダーを準備」を押し、調査者キーを作成して別に保管します。
3. 「情報源」：Telegram チャンネルのエクスポートを取り込む（上部のボタン）か、情報源に本文とファイルを手で追加します。
4. 情報源を開いて主張を作成します：引用、当事者、役割。「対象」と「グラフ」を確認します。
5. AI 分析：「文書」（匿名化とパッケージ）と「AI の結果」（検査と取り込み）。「レポート」で要約を作ります。

## コマンドライン

同じ機能をコマンドでも使えます。`zanytky help` で一覧を表示します。

```bash
zanytky init --home ./home
zanytky identity-new ./investigator.txt
zanytky import-telegram "ChatExport_2026-11-01" --home ./home --reliability F
zanytky import result.json --package pkg-1 --home ./home
zanytky report --home ./home --identity ./investigator.txt --out report.docx --format docx --passphrase-file ./pass.txt
zanytky backup --home ./home --out copy.znbak --passphrase-file ./pass.txt
```

コマンドでの分析と誤りの修正：

```bash
zanytky search "Приклад" --home ./home
zanytky source 123 --full --home ./home
zanytky discover --source 123 --home ./home
zanytky make-statements --file statements.json --home ./home
zanytky evidence --statement 5 --kind registry --title "Витяг" --url https://example.org/x --status confirmed --home ./home
zanytky graph --out graph.html --entity 32 --depth 2 --home ./home
```

`evidence` の `--expect` は証拠が誤った主張に付くのを防ぎます。`detach`、`delete-statement`、`delete-source`、`merge` で誤りを直します（削除は元に戻せないため、大量変更の前に `backup`）。

## 原則

- すべての主張には情報源と一次情報源までの連鎖があります。噂は事実になりません。1 つの噂の反響は 1 つの一次情報源です。
- 人物に対する評価的なレッテルは貼りません。出所と信頼性の評価を示し、判決は示しません。
- 公開情報源のみ。本文とファイルは匿名化せずローカルに保管し、AI には匿名化したテキストだけを渡します。

## 現行バージョン（0.0.24）の制限

- プレリリース：インストーラーは署名されていません。
- PDF の取り込みとスキャン・動画からの文字認識はまだありません。主張の引用は手で貼り付けます。
- Word レポートと PDF 表示は Microsoft Word や PDF ビューアーで未確認です。未知の名前の認識は単純です。
- データフォルダーはコードフォルダーの外に置き、パッケージ ID に個人データを書かないでください。ID は匿名化されずに AI へ渡されます。

## セキュリティ

システムキー（データフォルダーの `system.key`）、調査者の秘密鍵、パスフレーズは別々に保管し、決してリポジトリに追加しないでください。アプリケーションはネットワークに何も送信しません。

## ドキュメント

- [ユーザーガイド](../../docs/user-guide/USER_GUIDE.ja.md)
- [リリースノート](../../docs/releases)
- [LICENSE](../../LICENSE.md)
