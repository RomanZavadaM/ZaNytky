//! ZaNytky core: canonical form of values and keyed one-to-one pseudonyms.

pub mod normalize;
pub mod pseudonym;
pub mod seal;

pub use normalize::{normalize, Kind, NormalizeError, NORM_VERSION};
pub use pseudonym::{new_key, token, TokenError, MIN_KEY_BYTES};
pub use seal::{parse_identity, parse_recipient, seal, unseal, Sealed, SealError};
