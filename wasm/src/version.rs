use wasm_bindgen::prelude::*;

/// Returns the version of the Rusty Spectre framework.
/// @category General
#[wasm_bindgen]
pub fn version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}
