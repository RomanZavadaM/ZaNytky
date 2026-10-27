//! ZaNytky core: canonical form of values and keyed one-to-one pseudonyms.

pub mod normalize;
pub mod pseudonym;

pub use normalize::{normalize, Kind, NormalizeError, NORM_VERSION};
pub use pseudonym::{new_key, token, TokenError, MIN_KEY_BYTES};
