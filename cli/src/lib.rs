extern crate self as spectre_cli;

mod cli;
pub mod error;
pub mod extensions;
mod helpers;
mod imports;
mod matchers;
pub mod modules;
mod notifier;
pub mod result;
pub mod utils;
mod wizards;

pub use cli::{spectre_cli, Options, SpectreCli, TerminalOptions, TerminalTarget};
pub use workflow_terminal::Terminal;
