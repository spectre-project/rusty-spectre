//!
//! Partially Signed Spectre Transaction (PSST)
//!

use serde::{Deserialize, Serialize};
use serde_repr::{Deserialize_repr, Serialize_repr};
use spectre_bip32::{secp256k1, DerivationPath, KeyFingerprint};
use spectre_consensus_core::hashing::sighash::SigHashReusedValuesUnsync;
use std::{collections::BTreeMap, fmt::Display, fmt::Formatter, future::Future, marker::PhantomData, ops::Deref};

pub use crate::error::Error;
pub use crate::global::{Global, GlobalBuilder};
pub use crate::input::{Input, InputBuilder};
pub use crate::output::{Output, OutputBuilder};
pub use crate::role::{Combiner, Constructor, Creator, Extractor, Finalizer, Signer, Updater};
use spectre_consensus_core::tx::UtxoEntry;
use spectre_consensus_core::{
    hashing::sighash_type::SigHashType,
    subnets::SUBNETWORK_ID_NATIVE,
    tx::{MutableTransaction, SignableTransaction, Transaction, TransactionId, TransactionInput, TransactionOutput},
};
use spectre_txscript::{caches::Cache, TxScriptEngine};

#[derive(Debug, Default, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Inner {
    /// The global map.
    pub global: Global,
    /// The corresponding key-value map for each input in the unsigned transaction.
    pub inputs: Vec<Input>,
    /// The corresponding key-value map for each output in the unsigned transaction.
    pub outputs: Vec<Output>,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash, Serialize_repr, Deserialize_repr)]
#[repr(u8)]
pub enum Version {
    #[default]
    Zero = 0,
}

impl Display for Version {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Version::Zero => write!(f, "{}", Version::Zero as u8),
        }
    }
}

/// Full information on the used extended public key: fingerprint of the
/// master extended public key and a derivation path from it.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct KeySource {
    #[serde(with = "spectre_utils::serde_bytes_fixed")]
    pub key_fingerprint: KeyFingerprint,
    pub derivation_path: DerivationPath,
}

impl KeySource {
    pub fn new(key_fingerprint: KeyFingerprint, derivation_path: DerivationPath) -> Self {
        Self { key_fingerprint, derivation_path }
    }
}

pub type PartialSigs = BTreeMap<secp256k1::PublicKey, Signature>;

#[derive(Debug, Serialize, Deserialize, Eq, PartialEq, Copy, Clone)]
#[serde(rename_all = "camelCase")]
pub enum Signature {
    ECDSA(secp256k1::ecdsa::Signature),
    Schnorr(secp256k1::schnorr::Signature),
}

impl Signature {
    pub fn into_bytes(self) -> [u8; 64] {
        match self {
            Signature::ECDSA(s) => s.serialize_compact(),
            Signature::Schnorr(s) => s.serialize(),
        }
    }
}

///
/// A Partially Signed Spectre Transaction (PSST) is a standardized format
/// that allows multiple participants to collaborate in creating and signing
/// a Spectre transaction. PSST enables the exchange of incomplete transaction
/// data between different wallets or entities, allowing each participant
/// to add their signature or inputs in stages. This facilitates more complex
/// transaction workflows, such as multi-signature setups or hardware wallet
/// interactions, by ensuring that sensitive data remains secure while
/// enabling cooperation across different devices or platforms without
/// exposing private keys.
///
/// Please note that due to transaction mass limits and potential of
/// a wallet aggregating large UTXO sets, the PSST [`Bundle`](crate::bundle::Bundle) primitive
/// is used to represent a collection of PSSTs and should be used for
/// PSST serialization and transport. PSST is an internal implementation
/// primitive that represents each transaction in the bundle.
///
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PSST<ROLE> {
    #[serde(flatten)]
    inner_psst: Inner,
    #[serde(skip_serializing, default)]
    role: PhantomData<ROLE>,
}

impl<ROLE> From<Inner> for PSST<ROLE> {
    fn from(inner_psst: Inner) -> Self {
        PSST { inner_psst, role: Default::default() }
    }
}

impl<ROLE> Clone for PSST<ROLE> {
    fn clone(&self) -> Self {
        PSST { inner_psst: self.inner_psst.clone(), role: Default::default() }
    }
}

impl<ROLE> Deref for PSST<ROLE> {
    type Target = Inner;

    fn deref(&self) -> &Self::Target {
        &self.inner_psst
    }
}

impl<R> PSST<R> {
    fn unsigned_tx(&self) -> SignableTransaction {
        let tx = Transaction::new(
            self.global.tx_version,
            self.inputs
                .iter()
                .map(|Input { previous_outpoint, sequence, sig_op_count, .. }| TransactionInput {
                    previous_outpoint: *previous_outpoint,
                    signature_script: vec![],
                    sequence: sequence.unwrap_or(u64::MAX),
                    sig_op_count: sig_op_count.unwrap_or(0),
                })
                .collect(),
            self.outputs
                .iter()
                .map(|Output { amount, script_public_key, .. }: &Output| TransactionOutput {
                    value: *amount,
                    script_public_key: script_public_key.clone(),
                })
                .collect(),
            self.determine_lock_time(),
            SUBNETWORK_ID_NATIVE,
            0,
            vec![],
        );
        let entries = self.inputs.iter().filter_map(|Input { utxo_entry, .. }| utxo_entry.clone()).collect();
        SignableTransaction::with_entries(tx, entries)
    }

    fn calculate_id_internal(&self) -> TransactionId {
        self.unsigned_tx().tx.id()
    }

    fn determine_lock_time(&self) -> u64 {
        self.inputs.iter().map(|input: &Input| input.min_time).max().unwrap_or(self.global.fallback_lock_time).unwrap_or(0)
    }

    pub fn to_hex(&self) -> Result<String, Error> {
        Ok(format!("PSST{}", hex::encode(serde_json::to_string(self)?)))
    }

    pub fn from_hex(hex_data: &str) -> Result<Self, Error> {
        if let Some(hex_data) = hex_data.strip_prefix("PSST") {
            Ok(serde_json::from_slice(hex::decode(hex_data)?.as_slice())?)
        } else {
            Err(Error::PsstPrefixError)
        }
    }
}

impl Default for PSST<Creator> {
    fn default() -> Self {
        PSST { inner_psst: Default::default(), role: Default::default() }
    }
}

impl PSST<Creator> {
    /// Sets the fallback lock time.
    pub fn fallback_lock_time(mut self, fallback: u64) -> Self {
        self.inner_psst.global.fallback_lock_time = Some(fallback);
        self
    }

    // todo generic const
    /// Sets the inputs modifiable bit in the transaction modifiable flags.
    pub fn inputs_modifiable(mut self) -> Self {
        self.inner_psst.global.inputs_modifiable = true;
        self
    }
    // todo generic const
    /// Sets the outputs modifiable bit in the transaction modifiable flags.
    pub fn outputs_modifiable(mut self) -> Self {
        self.inner_psst.global.outputs_modifiable = true;
        self
    }

    pub fn constructor(self) -> PSST<Constructor> {
        PSST { inner_psst: self.inner_psst, role: Default::default() }
    }
}

impl PSST<Constructor> {
    // todo generic const
    /// Marks that the `PSST` can not have any more inputs added to it.
    pub fn no_more_inputs(mut self) -> Self {
        self.inner_psst.global.inputs_modifiable = false;
        self
    }
    // todo generic const
    /// Marks that the `PSST` can not have any more outputs added to it.
    pub fn no_more_outputs(mut self) -> Self {
        self.inner_psst.global.outputs_modifiable = false;
        self
    }

    /// Adds an input to the PSST.
    pub fn input(mut self, input: Input) -> Self {
        self.inner_psst.inputs.push(input);
        self.inner_psst.global.input_count += 1;
        self
    }

    /// Adds an output to the PSST.
    pub fn output(mut self, output: Output) -> Self {
        self.inner_psst.outputs.push(output);
        self.inner_psst.global.output_count += 1;
        self
    }

    /// Returns a PSST [`Updater`] once construction is completed.
    pub fn updater(self) -> PSST<Updater> {
        let psst = self.no_more_inputs().no_more_outputs();
        PSST { inner_psst: psst.inner_psst, role: Default::default() }
    }

    pub fn signer(self) -> PSST<Signer> {
        self.updater().signer()
    }

    pub fn combiner(self) -> PSST<Combiner> {
        PSST { inner_psst: self.inner_psst, role: Default::default() }
    }
}

impl PSST<Updater> {
    pub fn set_sequence(mut self, n: u64, input_index: usize) -> Result<Self, Error> {
        self.inner_psst.inputs.get_mut(input_index).ok_or(Error::OutOfBounds)?.sequence = Some(n);
        Ok(self)
    }

    pub fn signer(self) -> PSST<Signer> {
        PSST { inner_psst: self.inner_psst, role: Default::default() }
    }

    pub fn combiner(self) -> PSST<Combiner> {
        PSST { inner_psst: self.inner_psst, role: Default::default() }
    }
}

impl PSST<Signer> {
    // todo use iterator instead of vector
    pub fn pass_signature_sync<SignFn, E>(mut self, sign_fn: SignFn) -> Result<Self, E>
    where
        E: Display,
        SignFn: FnOnce(SignableTransaction, Vec<SigHashType>) -> Result<Vec<SignInputOk>, E>,
    {
        let unsigned_tx = self.unsigned_tx();
        let sighashes = self.inputs.iter().map(|input| input.sighash_type).collect();
        self.inner_psst.inputs.iter_mut().zip(sign_fn(unsigned_tx, sighashes)?).for_each(
            |(input, SignInputOk { signature, pub_key, key_source })| {
                input.bip32_derivations.insert(pub_key, key_source);
                input.partial_sigs.insert(pub_key, signature);
            },
        );

        Ok(self)
    }
    // todo use iterator instead of vector
    pub async fn pass_signature<SignFn, Fut, E>(mut self, sign_fn: SignFn) -> Result<Self, E>
    where
        E: Display,
        Fut: Future<Output = Result<Vec<SignInputOk>, E>>,
        SignFn: FnOnce(SignableTransaction, Vec<SigHashType>) -> Fut,
    {
        let unsigned_tx = self.unsigned_tx();
        let sighashes = self.inputs.iter().map(|input| input.sighash_type).collect();
        self.inner_psst.inputs.iter_mut().zip(sign_fn(unsigned_tx, sighashes).await?).for_each(
            |(input, SignInputOk { signature, pub_key, key_source })| {
                input.bip32_derivations.insert(pub_key, key_source);
                input.partial_sigs.insert(pub_key, signature);
            },
        );
        Ok(self)
    }

    pub fn calculate_id(&self) -> TransactionId {
        self.calculate_id_internal()
    }

    pub fn finalizer(self) -> PSST<Finalizer> {
        PSST { inner_psst: self.inner_psst, role: Default::default() }
    }

    pub fn combiner(self) -> PSST<Combiner> {
        PSST { inner_psst: self.inner_psst, role: Default::default() }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SignInputOk {
    pub signature: Signature,
    pub pub_key: secp256k1::PublicKey,
    pub key_source: Option<KeySource>,
}

impl<R> std::ops::Add<PSST<R>> for PSST<Combiner> {
    type Output = Result<Self, CombineError>;

    fn add(mut self, mut rhs: PSST<R>) -> Self::Output {
        self.inner_psst.global = (self.inner_psst.global + rhs.inner_psst.global)?;
        macro_rules! combine {
            ($left:expr, $right:expr, $err: ty) => {
                if $left.len() > $right.len() {
                    $left.iter_mut().zip($right.iter_mut()).try_for_each(|(left, right)| -> Result<(), $err> {
                        *left = (std::mem::take(left) + std::mem::take(right))?;
                        Ok(())
                    })?;
                    $left
                } else {
                    $right.iter_mut().zip($left.iter_mut()).try_for_each(|(left, right)| -> Result<(), $err> {
                        *left = (std::mem::take(left) + std::mem::take(right))?;
                        Ok(())
                    })?;
                    $right
                }
            };
        }
        // todo add sort to build deterministic combination
        self.inner_psst.inputs = combine!(self.inner_psst.inputs, rhs.inner_psst.inputs, crate::input::CombineError);
        self.inner_psst.outputs = combine!(self.inner_psst.outputs, rhs.inner_psst.outputs, crate::output::CombineError);
        Ok(self)
    }
}

impl PSST<Combiner> {
    pub fn signer(self) -> PSST<Signer> {
        PSST { inner_psst: self.inner_psst, role: Default::default() }
    }
    pub fn finalizer(self) -> PSST<Finalizer> {
        PSST { inner_psst: self.inner_psst, role: Default::default() }
    }
}

impl PSST<Finalizer> {
    pub fn finalize_sync<E: Display>(
        self,
        final_sig_fn: impl FnOnce(&Inner) -> Result<Vec<Vec<u8>>, E>,
    ) -> Result<Self, FinalizeError<E>> {
        let sigs = final_sig_fn(&self);
        self.finalize_internal(sigs)
    }

    pub async fn finalize<F, Fut, E>(self, final_sig_fn: F) -> Result<Self, FinalizeError<E>>
    where
        E: Display,
        F: FnOnce(&Inner) -> Fut,
        Fut: Future<Output = Result<Vec<Vec<u8>>, E>>,
    {
        let sigs = final_sig_fn(&self).await;
        self.finalize_internal(sigs)
    }

    pub fn id(&self) -> Option<TransactionId> {
        self.global.id
    }

    pub fn extractor(self) -> Result<PSST<Extractor>, TxNotFinalized> {
        if self.global.id.is_none() {
            Err(TxNotFinalized {})
        } else {
            Ok(PSST { inner_psst: self.inner_psst, role: Default::default() })
        }
    }

    fn finalize_internal<E: Display>(mut self, sigs: Result<Vec<Vec<u8>>, E>) -> Result<Self, FinalizeError<E>> {
        let sigs = sigs?;
        if sigs.len() != self.inputs.len() {
            return Err(FinalizeError::WrongFinalizedSigsCount { expected: self.inputs.len(), actual: sigs.len() });
        }
        self.inner_psst.inputs.iter_mut().enumerate().zip(sigs).try_for_each(|((idx, input), sig)| {
            if sig.is_empty() {
                return Err(FinalizeError::EmptySignature(idx));
            }
            input.sequence = Some(input.sequence.unwrap_or(u64::MAX)); // todo discussable
            input.final_script_sig = Some(sig);
            Ok(())
        })?;
        self.inner_psst.global.id = Some(self.calculate_id_internal());
        Ok(self)
    }
}

impl PSST<Extractor> {
    pub fn extract_tx_unchecked(self) -> Result<impl FnOnce(u64) -> (Transaction, Vec<Option<UtxoEntry>>), TxNotFinalized> {
        let tx = self.unsigned_tx();
        let entries = tx.entries;
        let mut tx = tx.tx;
        tx.inputs.iter_mut().zip(self.inner_psst.inputs).try_for_each(|(dest, src)| {
            dest.signature_script = src.final_script_sig.ok_or(TxNotFinalized {})?;
            Ok(())
        })?;
        Ok(move |mass| {
            tx.set_mass(mass);
            (tx, entries)
        })
    }

    pub fn extract_tx(self) -> Result<impl FnOnce(u64) -> (Transaction, Vec<Option<UtxoEntry>>), ExtractError> {
        let (tx, entries) = self.extract_tx_unchecked()?(0);

        let tx = MutableTransaction::with_entries(tx, entries.into_iter().flatten().collect());
        use spectre_consensus_core::tx::VerifiableTransaction;
        {
            let tx = tx.as_verifiable();
            let cache = Cache::new(10_000);
            let reused_values = SigHashReusedValuesUnsync::new();

            tx.populated_inputs().enumerate().try_for_each(|(idx, (input, entry))| {
                TxScriptEngine::from_transaction_input(&tx, input, idx, entry, &reused_values, &cache)?.execute()?;
                <Result<(), ExtractError>>::Ok(())
            })?;
        }
        let entries = tx.entries;
        let tx = tx.tx;
        let closure = move |mass| {
            tx.set_mass(mass);
            (tx, entries)
        };
        Ok(closure)
    }
}

/// Error combining psst.
#[derive(thiserror::Error, Debug, Clone, PartialEq, Eq)]
pub enum CombineError {
    #[error(transparent)]
    Global(#[from] crate::global::CombineError),
    #[error(transparent)]
    Inputs(#[from] crate::input::CombineError),
    #[error(transparent)]
    Outputs(#[from] crate::output::CombineError),
}

#[derive(thiserror::Error, Debug, Clone, PartialEq, Eq)]
pub enum FinalizeError<E> {
    #[error("Signatures count mismatch")]
    WrongFinalizedSigsCount { expected: usize, actual: usize },
    #[error("Signatures at index: {0} is empty")]
    EmptySignature(usize),
    #[error(transparent)]
    FinalaziCb(#[from] E),
}

#[derive(thiserror::Error, Debug, Clone, PartialEq, Eq)]
pub enum ExtractError {
    #[error(transparent)]
    TxScriptError(#[from] spectre_txscript_errors::TxScriptError),
    #[error(transparent)]
    TxNotFinalized(#[from] TxNotFinalized),
}

#[derive(thiserror::Error, Debug, Clone, PartialEq, Eq)]
#[error("Transaction is not finalized")]
pub struct TxNotFinalized {}

#[cfg(test)]
mod tests {

    // #[test]
    // fn it_works() {
    //     let result = add(2, 2);
    //     assert_eq!(result, 4);
    // }
}
