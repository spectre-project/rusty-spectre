use crate::psst::PSST as Native;
use crate::role::*;
use spectre_consensus_core::tx::TransactionId;
use wasm_bindgen::prelude::*;
// use js_sys::Object;
use crate::psst::Inner;
use serde::{Deserialize, Serialize};
use spectre_consensus_client::{Transaction, TransactionInput, TransactionInputT, TransactionOutput, TransactionOutputT};
use std::sync::MutexGuard;
use std::sync::{Arc, Mutex};
use workflow_wasm::{
    convert::{Cast, CastFromJs, TryCastFromJs},
    // extensions::object::*,
    // error::Error as CastError,
};

use super::error::*;
use super::result::*;

#[derive(Clone, Serialize, Deserialize)]
#[serde(tag = "state", content = "payload")]
pub enum State {
    NoOp(Option<Inner>),
    Creator(Native<Creator>),
    Constructor(Native<Constructor>),
    Updater(Native<Updater>),
    Signer(Native<Signer>),
    Combiner(Native<Combiner>),
    Finalizer(Native<Finalizer>),
    Extractor(Native<Extractor>),
}

impl AsRef<State> for State {
    fn as_ref(&self) -> &State {
        self
    }
}

impl State {
    // this is not a Display trait intentionally
    pub fn display(&self) -> &'static str {
        match self {
            State::NoOp(_) => "Init",
            State::Creator(_) => "Creator",
            State::Constructor(_) => "Constructor",
            State::Updater(_) => "Updater",
            State::Signer(_) => "Signer",
            State::Combiner(_) => "Combiner",
            State::Finalizer(_) => "Finalizer",
            State::Extractor(_) => "Extractor",
        }
    }
}

impl From<State> for PSST {
    fn from(state: State) -> Self {
        PSST { state: Arc::new(Mutex::new(Some(state))) }
    }
}

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(typescript_type = "PSST | Transaction | string | undefined")]
    pub type CtorT;
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Payload {
    data: String,
}

impl<T> TryFrom<Payload> for Native<T> {
    type Error = Error;

    fn try_from(value: Payload) -> Result<Self> {
        let Payload { data } = value;
        if data.starts_with("PSST") {
            unimplemented!("PSST binary serialization")
        } else {
            Ok(serde_json::from_str(&data).map_err(|err| format!("Invalid JSON: {err}"))?)
        }
    }
}

#[wasm_bindgen(inspectable)]
#[derive(Clone, CastFromJs)]
pub struct PSST {
    state: Arc<Mutex<Option<State>>>,
}

impl TryCastFromJs for PSST {
    type Error = Error;
    fn try_cast_from<'a, R>(value: &'a R) -> std::result::Result<Cast<'a, Self>, Self::Error>
    where
        R: AsRef<JsValue> + 'a,
    {
        Self::resolve(value, || {
            if let Some(data) = value.as_ref().as_string() {
                let psst_inner: Inner = serde_json::from_str(&data).map_err(|_| Error::InvalidPayload)?;
                Ok(PSST::from(State::NoOp(Some(psst_inner))))
            } else if let Ok(transaction) = Transaction::try_owned_from(value) {
                let psst_inner: Inner = transaction.try_into()?;
                Ok(PSST::from(State::NoOp(Some(psst_inner))))
            } else {
                Err(Error::InvalidPayload)
            }
        })
    }
}

#[wasm_bindgen]
impl PSST {
    #[wasm_bindgen(constructor)]
    pub fn new(payload: CtorT) -> Result<PSST> {
        PSST::try_owned_from(payload.unchecked_into::<JsValue>().as_ref()).map_err(|err| Error::Ctor(err.to_string()))
    }

    #[wasm_bindgen(getter, js_name = "role")]
    pub fn role_getter(&self) -> String {
        self.state().as_ref().unwrap().display().to_string()
    }

    #[wasm_bindgen(getter, js_name = "payload")]
    pub fn payload_getter(&self) -> JsValue {
        let state = self.state();
        serde_wasm_bindgen::to_value(state.as_ref().unwrap()).unwrap()
    }

    fn state(&self) -> MutexGuard<Option<State>> {
        self.state.lock().unwrap()
    }

    fn take(&self) -> State {
        self.state.lock().unwrap().take().unwrap()
    }

    fn replace(&self, state: State) -> Result<PSST> {
        self.state.lock().unwrap().replace(state);
        Ok(self.clone())
    }

    /// Change role to `CREATOR`
    /// #[wasm_bindgen(js_name = toCreator)]
    pub fn creator(&self) -> Result<PSST> {
        let state = match self.take() {
            State::NoOp(inner) => match inner {
                None => State::Creator(Native::default()),
                Some(_) => Err(Error::CreateNotAllowed)?,
            },
            state => Err(Error::state(state))?,
        };

        self.replace(state)
    }

    /// Change role to `CONSTRUCTOR`
    #[wasm_bindgen(js_name = toConstructor)]
    pub fn constructor(&self) -> Result<PSST> {
        let state = match self.take() {
            State::NoOp(inner) => State::Constructor(inner.ok_or(Error::NotInitialized)?.into()),
            State::Creator(psst) => State::Constructor(psst.constructor()),
            state => Err(Error::state(state))?,
        };

        self.replace(state)
    }

    /// Change role to `UPDATER`
    #[wasm_bindgen(js_name = toUpdater)]
    pub fn updater(&self) -> Result<PSST> {
        let state = match self.take() {
            State::NoOp(inner) => State::Updater(inner.ok_or(Error::NotInitialized)?.into()),
            State::Constructor(constructor) => State::Updater(constructor.updater()),
            state => Err(Error::state(state))?,
        };

        self.replace(state)
    }

    /// Change role to `SIGNER`
    #[wasm_bindgen(js_name = toSigner)]
    pub fn signer(&self) -> Result<PSST> {
        let state = match self.take() {
            State::NoOp(inner) => State::Signer(inner.ok_or(Error::NotInitialized)?.into()),
            State::Constructor(psst) => State::Signer(psst.signer()),
            State::Updater(psst) => State::Signer(psst.signer()),
            State::Combiner(psst) => State::Signer(psst.signer()),
            state => Err(Error::state(state))?,
        };

        self.replace(state)
    }

    /// Change role to `COMBINER`
    #[wasm_bindgen(js_name = toCombiner)]
    pub fn combiner(&self) -> Result<PSST> {
        let state = match self.take() {
            State::NoOp(inner) => State::Combiner(inner.ok_or(Error::NotInitialized)?.into()),
            State::Constructor(psst) => State::Combiner(psst.combiner()),
            State::Updater(psst) => State::Combiner(psst.combiner()),
            State::Signer(psst) => State::Combiner(psst.combiner()),
            state => Err(Error::state(state))?,
        };

        self.replace(state)
    }

    /// Change role to `FINALIZER`
    #[wasm_bindgen(js_name = toFinalizer)]
    pub fn finalizer(&self) -> Result<PSST> {
        let state = match self.take() {
            State::NoOp(inner) => State::Finalizer(inner.ok_or(Error::NotInitialized)?.into()),
            State::Combiner(psst) => State::Finalizer(psst.finalizer()),
            state => Err(Error::state(state))?,
        };

        self.replace(state)
    }

    /// Change role to `EXTRACTOR`
    #[wasm_bindgen(js_name = toExtractor)]
    pub fn extractor(&self) -> Result<PSST> {
        let state = match self.take() {
            State::NoOp(inner) => State::Extractor(inner.ok_or(Error::NotInitialized)?.into()),
            State::Finalizer(psst) => State::Extractor(psst.extractor()?),
            state => Err(Error::state(state))?,
        };

        self.replace(state)
    }

    #[wasm_bindgen(js_name = fallbackLockTime)]
    pub fn fallback_lock_time(&self, lock_time: u64) -> Result<PSST> {
        let state = match self.take() {
            State::Creator(psst) => State::Creator(psst.fallback_lock_time(lock_time)),
            state => Err(Error::state(state))?,
        };

        self.replace(state)
    }

    #[wasm_bindgen(js_name = inputsModifiable)]
    pub fn inputs_modifiable(&self) -> Result<PSST> {
        let state = match self.take() {
            State::Creator(psst) => State::Creator(psst.inputs_modifiable()),
            state => Err(Error::state(state))?,
        };

        self.replace(state)
    }

    #[wasm_bindgen(js_name = outputsModifiable)]
    pub fn outputs_modifiable(&self) -> Result<PSST> {
        let state = match self.take() {
            State::Creator(psst) => State::Creator(psst.outputs_modifiable()),
            state => Err(Error::state(state))?,
        };

        self.replace(state)
    }

    #[wasm_bindgen(js_name = noMoreInputs)]
    pub fn no_more_inputs(&self) -> Result<PSST> {
        let state = match self.take() {
            State::Constructor(psst) => State::Constructor(psst.no_more_inputs()),
            state => Err(Error::state(state))?,
        };

        self.replace(state)
    }

    #[wasm_bindgen(js_name = noMoreOutputs)]
    pub fn no_more_outputs(&self) -> Result<PSST> {
        let state = match self.take() {
            State::Constructor(psst) => State::Constructor(psst.no_more_outputs()),
            state => Err(Error::state(state))?,
        };

        self.replace(state)
    }

    pub fn input(&self, input: &TransactionInputT) -> Result<PSST> {
        let input = TransactionInput::try_owned_from(input)?;
        let state = match self.take() {
            State::Constructor(psst) => State::Constructor(psst.input(input.try_into()?)),
            state => Err(Error::state(state))?,
        };

        self.replace(state)
    }

    pub fn output(&self, output: &TransactionOutputT) -> Result<PSST> {
        let output = TransactionOutput::try_owned_from(output)?;
        let state = match self.take() {
            State::Constructor(psst) => State::Constructor(psst.output(output.try_into()?)),
            state => Err(Error::state(state))?,
        };

        self.replace(state)
    }

    #[wasm_bindgen(js_name = setSequence)]
    pub fn set_sequence(&self, n: u64, input_index: usize) -> Result<PSST> {
        let state = match self.take() {
            State::Updater(psst) => State::Updater(psst.set_sequence(n, input_index)?),
            state => Err(Error::state(state))?,
        };

        self.replace(state)
    }

    #[wasm_bindgen(js_name = calculateId)]
    pub fn calculate_id(&self) -> Result<TransactionId> {
        let state = self.state();
        match state.as_ref().unwrap() {
            State::Signer(psst) => Ok(psst.calculate_id()),
            state => Err(Error::state(state))?,
        }
    }
}
