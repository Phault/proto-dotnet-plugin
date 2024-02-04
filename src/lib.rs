#[cfg(feature = "wasm")]
mod global_json;
#[cfg(feature = "wasm")]
mod proto;
#[cfg(feature = "wasm")]
mod release_index;

#[cfg(feature = "wasm")]
pub use proto::*;
