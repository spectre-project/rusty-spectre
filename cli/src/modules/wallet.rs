use crate::imports::*;
use crate::wizards;

#[derive(Default, Handler)]
#[help("Wallet management operations")]
pub struct Wallet;

impl Wallet {
    async fn main(self: Arc<Self>, ctx: &Arc<dyn Context>, mut argv: Vec<String>, cmd: &str) -> Result<()> {
        let ctx = ctx.clone().downcast_arc::<SpectreCli>()?;

        if argv.is_empty() {
            return self.display_help(ctx, argv).await;
        }

        let op = argv.remove(0);
        match op.as_str() {
            "list" => {
                let wallets = ctx.store().wallet_list().await?;
                if wallets.is_empty() {
                    tprintln!(ctx, "No wallets found");
                } else {
                    tprintln!(ctx, "\nWallets:\n");
                    for wallet in wallets {
                        if let Some(title) = wallet.title {
                            tprintln!(ctx, "  {}: {}", wallet.filename, title);
                        } else {
                            tprintln!(ctx, "  {}", wallet.filename);
                        }
                    }
                    tprintln!(ctx, "");
                }
            }
            "create" | "import" => {
                let wallet_name = if argv.is_empty() {
                    None
                } else {
                    let name = argv.remove(0).trim().to_string();
                    if name.to_lowercase() == "wallet" {
                        return Err(Error::custom("Wallet name cannot be 'wallet'"));
                    }
                    Some(name)
                };

                let wallet_name = wallet_name.as_deref();
                let import_with_mnemonic = op == "import";
                wizards::wallet::create(&ctx, wallet_name, import_with_mnemonic).await?;
            }
            "open" => {
                let name = if let Some(name) = argv.first().cloned() {
                    if name.to_lowercase() == "wallet" {
                        tprintln!(ctx, "You cannot have a wallet named 'wallet'.");
                        tprintln!(ctx, "Perhaps you are looking to use 'open <name>'");
                        return Ok(());
                    }
                    Some(name)
                } else {
                    ctx.wallet().settings().get(WalletSettings::Wallet).clone()
                };

                let (wallet_secret, _) = ctx.ask_wallet_secret(None).await?;
                let _ = ctx.notifier().show(Notification::Processing).await;
                let args = WalletOpenArgs::default_with_legacy_accounts();
                ctx.wallet().open(&wallet_secret, name, args).await?;
                ctx.wallet().activate_accounts(None).await?;
            }
            "close" => {
                ctx.wallet().close().await?;
            }
            "hint" => {
                if !argv.is_empty() {
                    let re = regex::Regex::new(r"wallet\s+hint\s+").unwrap();
                    let hint = re.replace(cmd, "").trim().to_string();
                    let store = ctx.store();
                    if hint == "remove" {
                        tprintln!(ctx, "Hint is empty - removing wallet hint");
                        store.set_user_hint(None).await?;
                    } else {
                        store.set_user_hint(Some(spectre_wallet_core::storage::Hint { text: hint })).await?;
                    }
                } else {
                    tprintln!(ctx, "Usage:\n'wallet hint <text>' or 'wallet hint remove' to remove the hint");
                }
            }
            v => {
                tprintln!(ctx, "Unknown command: '{}'", v);
                return self.display_help(ctx, argv).await;
            }
        }

        Ok(())
    }

    async fn display_help(self: Arc<Self>, ctx: Arc<SpectreCli>, _argv: Vec<String>) -> Result<()> {
        ctx.term().help(
            &[
                ("list", "List available local wallet files."),
                ("create [<name>]", "Create a new BIP32 wallet."),
                (
                    "import [<name>]",
                    "Create a wallet from an existing mnemonic (BIP32 only). \n\n\
                    To import legacy wallets (Spectre Desktop or web wallet), please create \
                    a new BIP32 wallet and use the 'account import' command. \
                    Legacy wallets can only be imported as accounts.\n",
                ),
                ("open [<name>]", "Open an existing wallet (shorthand: 'open [<name>]')"),
                ("close", "Close an opened wallet (shorthand: 'close')"),
                ("hint", "Change the wallet phishing hint"),
            ],
            None,
        )?;

        Ok(())
    }
}
