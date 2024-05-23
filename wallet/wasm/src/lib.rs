use spectre_cli_lib::spectre_cli;
use wasm_bindgen::prelude::*;
use workflow_terminal::Options;
use workflow_terminal::Result;

#[wasm_bindgen]
pub async fn load_spectre_wallet_cli() -> Result<()> {
    let options = Options { ..Options::default() };
    spectre_cli(options, None).await?;
    Ok(())
}
