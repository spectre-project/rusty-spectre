use super::psst::State;
use thiserror::Error;
use wasm_bindgen::prelude::*;

#[derive(Error, Debug)]
pub enum Error {
    #[error("{0}")]
    Custom(String),

    #[error("Unexpected state: {0}")]
    State(String),

    #[error("Constructor argument must be a valid payload, another PSST instance, Transaction or undefined")]
    Ctor(String),

    #[error("Invalid payload")]
    InvalidPayload,

    #[error("Transaction not finalized")]
    TxNotFinalized(#[from] crate::psst::TxNotFinalized),

    #[error(transparent)]
    Wasm(#[from] workflow_wasm::error::Error),

    #[error("Create state is not allowed for PSST initialized from transaction or a payload")]
    CreateNotAllowed,

    #[error("PSST must be initialized with a payload or CREATE role")]
    NotInitialized,

    #[error(transparent)]
    ConsensusClient(#[from] spectre_consensus_client::error::Error),

    #[error(transparent)]
    Psst(#[from] crate::error::Error),
}

impl Error {
    pub fn custom<T: std::fmt::Display>(msg: T) -> Self {
        Error::Custom(msg.to_string())
    }

    pub fn state(state: impl AsRef<State>) -> Self {
        Error::State(state.as_ref().display().to_string())
    }
}

impl From<&str> for Error {
    fn from(msg: &str) -> Self {
        Error::Custom(msg.to_string())
    }
}

impl From<String> for Error {
    fn from(msg: String) -> Self {
        Error::Custom(msg)
    }
}

impl From<Error> for JsValue {
    fn from(err: Error) -> Self {
        JsValue::from_str(&err.to_string())
    }
}
