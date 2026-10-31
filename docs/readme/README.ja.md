# ZaNytky («За нитку»)

[🇺🇦 Українська](../../README.md) · [🇬🇧 English](README.en.md) · [🇫🇷 Français](README.fr.md) · [🇩🇪 Deutsch](README.de.md) · [🇪🇸 Español](README.es.md) · [🇰🇷 한국어](README.ko.md) · **🇯🇵 日本語**

公開情報源の対象間の関係を整理し、個人情報を鍵で仮名化するアプリケーションです。

状態: **初期開発**（プレリリース）。実装: Rust（コア）と React（ウィンドウ）。インターフェースとドキュメントの基準言語はウクライナ語です。

ZaNytky は**プロプライエタリソフトウェア**です。リポジトリが公開されていてもオープンソースライセンスが付与されるわけではありません（LICENSE.md、COPYRIGHT.md、THIRD_PARTY_NOTICES.md を参照）。ユーザーデータはリポジトリに保存されません。

> この翻訳は自動生成であり、ネイティブによる校閲を受けていません。基準はウクライナ語の文面です。

## 機能

- システム鍵により値（氏名、納税番号、電話、メール）を一対一で仮名化します。同じ値は常に同じコードになります。
- 原本は調査員の公開鍵で封印した状態でのみ保存され（age）、秘密鍵の所有者だけが開けます。
- AI 分析前のテキスト匿名化と AI 用パッケージ（テキスト、指示、メタデータ）、認識されなかった言及に対する調査員の判断。
- AI の回答の検査、人による確認、データベース（SQLite）への「未検証」としてのインポート。
- モデル: 対象、一次情報源までの連鎖を持つ情報源、信頼度と信憑性の評価を持つ記述、所有構造のツリー。
- パスフレーズで保護された Markdown、Word、PDF の調査レポート。

## ダウンロード

**ダウンロード: [ZaNytky v0.0.4](https://github.com/RomanZavadaM/ZaNytky/releases/tag/v0.0.6)** (最新のプレリリース)

Windows 用の完成ファイル（インストール不要のポータブル zip、.msi/.exe インストーラー）は[リリース](https://github.com/RomanZavadaM/ZaNytky/releases)ページにあります。ビルドは署名されていないため、Windows が不明な発行元の警告を表示することがあります。

> **ポータブル版は試用のみを目的としています。** ご利用は自己責任で、いかなる保証もありません。日常的な作業にはインストール版（.exe/.msi インストーラー）をご利用ください。

## クイックスタート（アプリケーション）

1. ポータブルアーカイブを展開して `ZaNytky.exe` を実行します（何もインストールする必要はありません。`webview2` フォルダーを `ZaNytky.exe` の隣に置いたままにしてください）。
2. 「設定」タブ: 空のデータフォルダーを選び「フォルダーを準備」を押します。調査員の鍵を作成し、そのファイルを安全な場所に保管します。
3. 「文書の準備」タブ: テキストを貼り付け、「匿名化して AI 用パッケージを作成」を押し、テキストと指示をコピーして AI に渡します。
4. 「AI の結果」タブ: 回答（result.json）を貼り付けて検査し、確認内容を読んでからインポートします。
5. 「データベース」と「レポート」タブ: 記述を閲覧し、確認または反証し、レポートを作成します。

## コマンドライン

同じ操作をコマンドでも実行できます。`zanytky help` で一覧が表示されます。

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

## 原則

- すべての記述には出典と一次情報源までの連鎖があります。噂は事実にならず、同じ噂の反響は一つの一次情報源として扱います。
- 人物に対する評価的なレッテルは貼りません。プログラムは判定ではなく、出所と信頼度の等級を示します。
- 公開情報源のみを使用し、個人データは仮名に置き換えられ、AI には匿名化されたテキストだけが渡されます。

## 現在のバージョン（0.0.6）の制限

- プレリリース版であり、インストーラーには署名がありません。
- 「文書の解析」は現在テキストと Markdown のみ対応しています（.docx/.pdf の取り込みはまだありません）。
- Word レポートは Microsoft Word で、PDF はビューアーで確認していません。未知の名前の検出は単純（先頭が大文字の語）で、誤った候補が出ることがあります。
- データフォルダーはコードのフォルダーの外に置き、パッケージ識別子には個人データ（例: メールアドレス）を入れないでください。識別子は匿名化されずに AI に渡されます。

## セキュリティ

システム鍵（データフォルダー内の `system.key`）、調査員の秘密鍵、パスフレーズは別々に保管し、リポジトリには決して追加しないでください。アプリケーションはネットワークに何も送信しません。

## ドキュメント

- [ユーザーガイド](../user-guide/USER_GUIDE.ja.md)
- [LICENSE](../../LICENSE.md)
