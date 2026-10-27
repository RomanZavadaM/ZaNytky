# Third-Party Notices

ZaNytky includes or can be distributed with third-party software. Those components remain the property of their copyright holders and are governed by their own licenses, terms and rights. The ZaNytky proprietary license does **not** replace, restrict or relicense third-party components.

## Direct dependencies

None yet (the project contains documents only). Every new direct dependency is added here, with its license and purpose, in the same pull request that introduces it, after the license check (`deny.toml` for Rust crates).

Planned (not yet added; each is checked before use):
- age (Rust crate) - MIT OR Apache-2.0 - sealing originals (ADR-0001);
- a SHA-256/HMAC implementation (RustCrypto `hmac`, `sha2`) - MIT OR Apache-2.0;
- rusqlite - MIT, bundling SQLite (public domain) - ADR-0002.

## License review

Only dependencies usable in a proprietary distribution are allowed: MIT, Apache-2.0, BSD, ISC, Zlib, CC0, Unlicense, Unicode and similar permissive licenses (see `deny.toml`). Copyleft licenses (GPL, AGPL, LGPL, MPL, EPL, SSPL, BUSL) are not used without a written owner decision. Libraries for export formats (Word, PDF) are checked especially carefully: several well-known PDF libraries are AGPL or commercial.

**ZaNytky original materials:** Copyright (c) 2026 Roman Zavada (Роман Завада). All rights reserved. See [LICENSE.md](LICENSE.md).
