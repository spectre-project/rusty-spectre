use crate::RpcUtxosByAddressesEntry;
use spectre_addresses::Prefix;
use spectre_consensus_core::tx::UtxoEntry;
use spectre_index_core::indexed_utxos::UtxoSetByScriptPublicKey;
use spectre_txscript::extract_script_pub_key_address;

// ----------------------------------------------------------------------------
// index to rpc_core
// ----------------------------------------------------------------------------

pub fn utxo_set_into_rpc(item: &UtxoSetByScriptPublicKey, prefix: Option<Prefix>) -> Vec<RpcUtxosByAddressesEntry> {
    item.iter()
        .flat_map(|(script_public_key, utxo_collection)| {
            let address = prefix.and_then(|x| extract_script_pub_key_address(script_public_key, x).ok());
            utxo_collection
                .iter()
                .map(|(outpoint, entry)| RpcUtxosByAddressesEntry {
                    address: address.clone(),
                    outpoint: *outpoint,
                    utxo_entry: UtxoEntry::new(entry.amount, script_public_key.clone(), entry.block_daa_score, entry.is_coinbase),
                })
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>()
}
