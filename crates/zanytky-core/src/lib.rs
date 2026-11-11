//! ZaNytky core: canonical form of values and keyed one-to-one pseudonyms.

/// A value as indented JSON (for a program that reads the output of the command line).
pub fn json_pretty<T: serde::Serialize>(value: &T) -> Result<String, String> {
    serde_json::to_string_pretty(value).map_err(|e| e.to_string())
}

pub mod b64;
pub mod backup;
pub mod card;
pub mod channel;
pub mod deidentify;
pub mod demo;
pub mod discover;
pub mod docx_text;
pub mod dossier;
pub mod export;
pub mod import;
pub mod make;
pub mod model;
pub mod normalize;
pub mod overlap;
pub mod overview;
pub mod package;
pub mod path;
pub mod pdf;
pub mod pseudonym;
pub mod registry;
pub mod report;
pub mod result;
pub mod seal;
pub mod sources;
pub mod telegram;
pub mod wrap;

pub use b64::base64_encode;
pub use backup::{backup_home, restore_home, BackupError};
pub use card::{
    entity_card, entity_graph, entity_list, graph_html, EntityCard, EntityGraph, EntityListRow,
    GraphScope, SharedSign,
};
pub use channel::{
    channel_gaps, channel_graph, channel_list, channel_timeline, ChannelGaps, ChannelRow, MonthRow,
};
pub use deidentify::{
    deidentify, deidentify_with_links, Candidate, Deidentified, DictEntry, Replacement,
};
pub use demo::{create_demo, DemoInfo};
pub use discover::{
    add_found, discover_everywhere, discover_in_text, subject_pairs, Found, SubjectPair,
};
pub use docx_text::{docx_to_text, DocxError};
pub use dossier::{
    analyze_article, article_markdown, dossier_markdown, word_uk, ArticleAnalysis, ArticleDoc,
    ArticleLink, ArticleSubject,
};
pub use export::{markdown_table, markdown_table_protected};
pub use import::{import_result, review_with_links, ImportError, ImportSummary, LinkMatch};
pub use make::{
    add_evidence, check_statement_request, create_statement, parse_statement_requests,
    EvidenceRequest, PartyRequest, StatementRequest,
};
pub use model::{Grade, Model, ModelError};
pub use normalize::{normalize, Kind, NormalizeError, NORM_VERSION};
pub use overlap::{channel_overlaps, overlaps_graph, Shared, SharedChannel};
pub use overview::build_overview;
pub use package::{build_package, build_package_with_links, Package, SourceMeta};
pub use path::{find_paths, EntityPath, PathResult};
pub use pdf::{pdf_protected, to_pdf};
pub use pseudonym::{new_key, token, TokenError, MIN_KEY_BYTES};
pub use registry::{Registry, RegistryError, Resolution};
pub use report::{
    build_report, docx_protected, markdown_protected, to_docx, to_markdown, Block, Report,
    ReportError,
};
pub use result::{parse_result, review_markdown, ResultError, ResultFile};
pub use seal::{parse_identity, parse_recipient, seal, unseal, SealError, Sealed};
pub use sources::{source_list, source_view, SourceListPage, SourceListRow};
pub use telegram::{import_telegram_export, TelegramError, TelegramSummary};
