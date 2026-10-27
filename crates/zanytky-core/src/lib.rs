//! ZaNytky core: canonical form of values and keyed one-to-one pseudonyms.

pub mod export;
pub mod model;
pub mod normalize;
pub mod pseudonym;
pub mod registry;
pub mod seal;
pub mod wrap;

pub use export::{markdown_table, markdown_table_protected};
pub use model::{Grade, Model, ModelError};
pub use normalize::{normalize, Kind, NormalizeError, NORM_VERSION};
pub use pseudonym::{new_key, token, TokenError, MIN_KEY_BYTES};
pub use registry::{Registry, RegistryError};
pub use seal::{parse_identity, parse_recipient, seal, unseal, SealError, Sealed};
