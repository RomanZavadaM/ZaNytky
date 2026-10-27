//! ZaNytky core: canonical form of values and keyed one-to-one pseudonyms.

pub mod deidentify;
pub mod export;
pub mod import;
pub mod model;
pub mod normalize;
pub mod package;
pub mod pdf;
pub mod pseudonym;
pub mod registry;
pub mod report;
pub mod result;
pub mod seal;
pub mod wrap;

pub use deidentify::{
    deidentify, deidentify_with_links, Candidate, Deidentified, DictEntry, Replacement,
};
pub use export::{markdown_table, markdown_table_protected};
pub use import::{import_result, ImportError, ImportSummary};
pub use model::{Grade, Model, ModelError};
pub use normalize::{normalize, Kind, NormalizeError, NORM_VERSION};
pub use package::{build_package, build_package_with_links, Package, SourceMeta};
pub use pdf::{pdf_protected, to_pdf};
pub use pseudonym::{new_key, token, TokenError, MIN_KEY_BYTES};
pub use registry::{Registry, RegistryError, Resolution};
pub use report::{
    build_report, docx_protected, markdown_protected, to_docx, to_markdown, Block, Report,
    ReportError,
};
pub use result::{parse_result, review_markdown, ResultError, ResultFile};
pub use seal::{parse_identity, parse_recipient, seal, unseal, SealError, Sealed};
