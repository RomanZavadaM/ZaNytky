# Third-Party Notices

ZaNytky includes or can be distributed with third-party software. Those components remain the property of their copyright holders and are governed by their own licenses, terms and rights. The ZaNytky proprietary license does **not** replace, restrict or relicense third-party components.

## Direct dependencies

Core library `crates/zanytky-core`:

- hmac, sha2 (RustCrypto) - MIT OR Apache-2.0 - keyed pseudonyms (HMAC-SHA256)
- unicode-normalization - MIT OR Apache-2.0 - canonical form of text values
- getrandom - MIT OR Apache-2.0 - random system key
- age - MIT OR Apache-2.0 - sealing originals to the investigator's public key; no default features (no CLI, plugin or SSH code)

Every new direct dependency is added here, with its license and purpose, in the same pull request that introduces it, after the license check (`deny.toml` for Rust crates).

- rusqlite - MIT - SQLite registry of pseudonyms; bundles SQLite (public domain)
- serde, serde_json - MIT OR Apache-2.0 - parse package metadata and the AI result (`result.json`)
- docx-rs - MIT - writing Word (.docx) reports; no default features (no image support)
- genpdf - MIT OR Apache-2.0 - PDF reports; no default features (no image support)
- Noto Sans (Regular) - SIL Open Font License 1.1, Copyright 2022 The Noto Project Authors - font embedded in PDF reports. The application embeds a Latin and Cyrillic subset (`crates/zanytky-core/assets/NotoSans-Regular-subset.ttf`, made with fonttools by the `Font subset` workflow; SHA-256 74507180ebaf8d60918fb5aad62ec8e0705e938ff7a2a62e34cc1c985409a33e); the full font is kept for reference (`NotoSans-Regular.ttf`), license text in `assets/OFL.txt`; source: github.com/notofonts/notofonts.github.io, SHA-256 478c558ea716033cd60c03438f628dfa75694dcf6b5f6d505a2f05fd2b4f3823)
- tauri, tauri-build, tauri-plugin-dialog - MIT OR Apache-2.0 - desktop application shell (`app/src-tauri`)
- tempfile - MIT OR Apache-2.0 - development dependency, tests only

## npm packages (desktop application `app/`)

- React, React DOM - MIT
- `@tauri-apps/api`, `@tauri-apps/plugin-dialog`, `@tauri-apps/cli` - MIT OR Apache-2.0
- Vite, `@vitejs/plugin-react` - MIT (build tools)
- TypeScript - Apache-2.0 (build tool)

The application scaffold (Tauri configuration, build setup) is taken from MD Notes, a project of the same copyright owner.

## License review

Only dependencies usable in a proprietary distribution are allowed: MIT, Apache-2.0, BSD, ISC, Zlib, CC0, Unlicense, Unicode and similar permissive licenses (see `deny.toml`). Copyleft licenses (GPL, AGPL, LGPL, MPL, EPL, SSPL, BUSL) are not used without a written owner decision. Libraries for export formats (Word, PDF) are checked especially carefully: several well-known PDF libraries are AGPL or commercial.

**ZaNytky original materials:** Copyright (c) 2026 Roman Zavada (Роман Завада). All rights reserved. See [LICENSE.md](LICENSE.md).

## Microsoft Edge WebView2 Runtime (портативна збірка)
Портативний архів містить копію Microsoft Edge WebView2 Runtime (папка `webview2`), що розповсюджується за ліцензією Microsoft (Microsoft Software License Terms for WebView2 Runtime). Вона не входить до переліку дозвільних ліцензій відкритого коду і використана як системний компонент, не як залежність коду; рішення власника 29.10.2026 (варіант 1).
