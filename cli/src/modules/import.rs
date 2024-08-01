use crate::imports::*;

#[derive(Default, Handler)]
#[help("Import a wallet using a mnemonic or a private key")]
pub struct Import;

impl Import {
    async fn main(self: Arc<Self>, ctx: &Arc<dyn Context>, argv: Vec<String>, _cmd: &str) -> Result<()> {
        let ctx = ctx.clone().downcast_arc::<SpectreCli>()?;
        let wallet = ctx.wallet();

        if argv.is_empty() {
            self.display_help(ctx).await?;
            return Ok(());
        }

        let command = argv.get(0).unwrap();
        match command.as_str() {
            "mnemonic" => {
                let account_kind = argv.get(1).map_or(Ok(AccountKind::Bip32), |arg| arg.parse::<AccountKind>())?;
                let additional_keys = if argv.len() > 2 { &argv[2..] } else { &[] };
                crate::wizards::import::import_with_mnemonic(&ctx, account_kind, additional_keys).await?;
            }
            "legacy" => {
                if exists_legacy_v0_keydata().await? {
                    let import_secret = Secret::new(ctx.term().ask(true, "Enter the password for the account you are importing: ").await?.trim().as_bytes().to_vec());
                    let wallet_secret = Secret::new(ctx.term().ask(true, "Enter wallet password: ").await?.trim().as_bytes().to_vec());
                    wallet.import_gen0_keydata(import_secret, wallet_secret, None).await?;
                } else if application_runtime::is_web() {
                    return Err("Web wallet storage not found at this domain name".into());
                } else {
                    return Err("Spectre Desktop/web wallet keydata file not found".into());
                }
            }
            // todo "read-only" => {}
            // "core" => {}
            unknown => {
                tprintln!(ctx, "Unknown command: '{unknown}'\r\n");
                return self.display_help(ctx).await;
            }
        }

        Ok(())
    }

    async fn display_help(self: Arc<Self>, ctx: Arc<SpectreCli>) -> Result<()> {
        ctx.term().help(
            &[
                ("mnemonic [<type>] [<additional xpub keys>]", "Import a mnemonic (12 or 24 words). Supported types: 'bip32' (default), 'legacy', 'multisig'."),
                ("legacy", "Import a legacy wallet (local Spectre Desktop)."),
                // ("purge", "Purge an account from the wallet."),
            ],
            None,
        )?;

        Ok(())
    }
}
