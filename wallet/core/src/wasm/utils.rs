use crate::result::Result;
use js_sys::BigInt;
use spectre_consensus_core::network::{NetworkType, NetworkTypeT};
use wasm_bindgen::prelude::*;
use workflow_wasm::prelude::*;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(typescript_type = "bigint | number | HexString")]
    #[derive(Clone, Debug)]
    pub type ISompiToSpectre;
}

/// Convert a Spectre string to Sompi represented by bigint.
/// This function provides correct precision handling and
/// can be used to parse user input.
/// @category Wallet SDK
#[wasm_bindgen(js_name = "spectreToSompi")]
pub fn spectre_to_sompi(spectre: String) -> Option<BigInt> {
    crate::utils::try_spectre_str_to_sompi(spectre).ok().flatten().map(Into::into)
}

///
/// Convert Sompi to a string representation of the amount in Spectre.
///
/// @category Wallet SDK
///
#[wasm_bindgen(js_name = "sompiToSpectreString")]
pub fn sompi_to_spectre_string(sompi: ISompiToSpectre) -> Result<String> {
    let sompi = sompi.try_as_u64()?;
    Ok(crate::utils::sompi_to_spectre_string(sompi))
}

///
/// Format a Sompi amount to a string representation of the amount in Spectre with a suffix
/// based on the network type (e.g. `SPR` for mainnet, `TSPR` for testnet,
/// `SSPR` for simnet, `DSPR` for devnet).
///
/// @category Wallet SDK
///
#[wasm_bindgen(js_name = "sompiToSpectreStringWithSuffix")]
pub fn sompi_to_spectre_string_with_suffix(sompi: ISompiToSpectre, network: &NetworkTypeT) -> Result<String> {
    let sompi = sompi.try_as_u64()?;
    let network_type = NetworkType::try_from(network)?;
    Ok(crate::utils::sompi_to_spectre_string_with_suffix(sompi, &network_type))
}
