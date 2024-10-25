//!
//! PSST is a crate for working with Partially Signed Spectre Transactions (PSSTs).
//! This crate provides following primitives: `PSST`, `PSSTBuilder` and `Bundle`.
//! The `Bundle` struct is used for PSST exchange payload serialization and carries
//! multiple `PSST` instances allowing for exchange of Spectre sweep transactions.
//!

pub mod bundle;
pub mod error;
pub mod global;
pub mod input;
pub mod output;
pub mod psst;
pub mod role;
pub mod wasm;

mod convert;
mod utils;

pub mod prelude {
    pub use crate::bundle::Bundle;
    pub use crate::bundle::*;
    pub use crate::global::Global;
    pub use crate::input::Input;
    pub use crate::output::Output;
    pub use crate::psst::*;

    // not quite sure why it warns of unused imports,
    // perhaps due to the fact that enums have no variants?
    #[allow(unused_imports)]
    pub use crate::role::*;
}
