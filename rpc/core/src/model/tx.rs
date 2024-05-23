use borsh::{BorshDeserialize, BorshSerialize};
use serde::{Deserialize, Serialize};
use spectre_addresses::Address;
use spectre_consensus_core::tx::{
    ScriptPublicKey, ScriptVec, TransactionId, TransactionInput, TransactionOutpoint, TransactionOutput, UtxoEntry,
};

use crate::prelude::{RpcHash, RpcScriptClass, RpcSubnetworkId};

/// Represents the ID of a Spectre transaction
pub type RpcTransactionId = TransactionId;

pub type RpcScriptVec = ScriptVec;
pub type RpcScriptPublicKey = ScriptPublicKey;
pub type RpcUtxoEntry = UtxoEntry;

/// Represents a Spectre transaction outpoint
pub type RpcTransactionOutpoint = TransactionOutpoint;

/// Represents a Spectre transaction input
#[derive(Clone, Debug, Serialize, Deserialize, BorshSerialize, BorshDeserialize)]
#[serde(rename_all = "camelCase")]
pub struct RpcTransactionInput {
    pub previous_outpoint: RpcTransactionOutpoint,
    #[serde(with = "hex::serde")]
    pub signature_script: Vec<u8>,
    pub sequence: u64,
    pub sig_op_count: u8,
    pub verbose_data: Option<RpcTransactionInputVerboseData>,
}

impl From<TransactionInput> for RpcTransactionInput {
    fn from(input: TransactionInput) -> Self {
        Self {
            previous_outpoint: input.previous_outpoint,
            signature_script: input.signature_script,
            sequence: input.sequence,
            sig_op_count: input.sig_op_count,
            verbose_data: None,
        }
    }
}

impl RpcTransactionInput {
    pub fn from_transaction_inputs(other: Vec<TransactionInput>) -> Vec<Self> {
        other.into_iter().map(Self::from).collect()
    }
}

/// Represent Spectre transaction input verbose data
#[derive(Clone, Debug, Serialize, Deserialize, BorshSerialize, BorshDeserialize)]
#[serde(rename_all = "camelCase")]
pub struct RpcTransactionInputVerboseData {}

/// Represents a Spectred transaction output
#[derive(Clone, Debug, Serialize, Deserialize, BorshSerialize, BorshDeserialize)]
#[serde(rename_all = "camelCase")]
pub struct RpcTransactionOutput {
    pub value: u64,
    pub script_public_key: RpcScriptPublicKey,
    pub verbose_data: Option<RpcTransactionOutputVerboseData>,
}

impl RpcTransactionOutput {
    pub fn from_transaction_outputs(other: Vec<TransactionOutput>) -> Vec<Self> {
        other.into_iter().map(Self::from).collect()
    }
}

impl From<TransactionOutput> for RpcTransactionOutput {
    fn from(output: TransactionOutput) -> Self {
        Self { value: output.value, script_public_key: output.script_public_key, verbose_data: None }
    }
}

/// Represent Spectre transaction output verbose data
#[derive(Clone, Debug, Serialize, Deserialize, BorshSerialize, BorshDeserialize)]
#[serde(rename_all = "camelCase")]
pub struct RpcTransactionOutputVerboseData {
    pub script_public_key_type: RpcScriptClass,
    pub script_public_key_address: Address,
}

/// Represents a Spectre transaction
#[derive(Clone, Debug, Serialize, Deserialize, BorshSerialize, BorshDeserialize)]
#[serde(rename_all = "camelCase")]
pub struct RpcTransaction {
    pub version: u16,
    pub inputs: Vec<RpcTransactionInput>,
    pub outputs: Vec<RpcTransactionOutput>,
    pub lock_time: u64,
    pub subnetwork_id: RpcSubnetworkId,
    pub gas: u64,
    #[serde(with = "hex::serde")]
    pub payload: Vec<u8>,
    pub mass: u64,
    pub verbose_data: Option<RpcTransactionVerboseData>,
}

/// Represent Spectre transaction verbose data
#[derive(Clone, Debug, Serialize, Deserialize, BorshSerialize, BorshDeserialize)]
#[serde(rename_all = "camelCase")]
pub struct RpcTransactionVerboseData {
    pub transaction_id: RpcTransactionId,
    pub hash: RpcHash,
    pub mass: u64,
    pub block_hash: RpcHash,
    pub block_time: u64,
}

/// Represents accepted transaction ids
#[derive(Clone, Debug, Serialize, Deserialize, BorshSerialize, BorshDeserialize)]
#[serde(rename_all = "camelCase")]
pub struct RpcAcceptedTransactionIds {
    pub accepting_block_hash: RpcHash,
    pub accepted_transaction_ids: Vec<RpcTransactionId>,
}
