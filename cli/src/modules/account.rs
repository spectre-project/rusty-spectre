use spectre_wallet_core::account::BIP32_ACCOUNT_KIND;
use spectre_wallet_core::account::LEGACY_ACCOUNT_KIND;
use spectre_wallet_core::account::MULTISIG_ACCOUNT_KIND;

use crate::imports::*;
use crate::wizards;

#[derive(Default, Handler)]
#[help("Account management operations")]
pub struct Account;

impl Account {
    async fn main(self: Arc<Self>, ctx: &Arc<dyn Context>, mut argv: Vec<String>, _cmd: &str) -> Result<()> {
        let ctx = ctx.clone().downcast_arc::<SpectreCli>()?;
        let wallet = ctx.wallet();

        if !wallet.is_open() {
            return Err(Error::WalletIsNotOpen);
        }

        if argv.is_empty() {
            return self.display_help(ctx, argv).await;
        }

        let action = argv.remove(0);

        match action.as_str() {
            "name" => {
                if argv.len() != 1 {
                    tprintln!(ctx, "Usage: 'account name <name>' or 'account name remove'");
                    return Ok(());
                } else {
                    let (wallet_secret, _) = ctx.ask_wallet_secret(None).await?;
                    let _ = ctx.notifier().show(Notification::Processing).await;
                    let account = ctx.select_account().await?;
                    let name = argv.remove(0);
                    if name == "remove" {
                        account.rename(&wallet_secret, None).await?;
                    } else {
                        account.rename(&wallet_secret, Some(name.as_str())).await?;
                    }
                }
            }
            "create" => {
                let account_kind = if argv.is_empty() {
                    BIP32_ACCOUNT_KIND.into()
                } else {
                    let kind = argv.remove(0);
                    kind.parse::<AccountKind>()?
                };

                let account_name = if argv.is_empty() {
                    None
                } else {
                    let name = argv.remove(0).trim().to_string();
                    Some(name)
                };

                let prv_key_data_info = ctx.select_private_key().await?;

                let account_name = account_name.as_deref();
                wizards::account::create(&ctx, prv_key_data_info, account_kind, account_name).await?;
            }
            "import" => {
                if argv.is_empty() {
                    tprintln!(ctx, "Usage: 'account import <import-type> <key-type> [extra keys]'");
                    tprintln!(ctx, "\nExamples:");
                    tprintln!(ctx, "");
                    ctx.term().help(
                        &[
                            (
                                "account import legacy-data",
                                "Import Spectre Desktop keydata file or web wallet data on the same domain",
                            ),
                            (
                                "account import mnemonic bip32",
                                "Import Bip32 (12 or 24 word mnemonics used by spectrewallet, spectre-mobile, etc.)",
                            ),
                            (
                                "account import mnemonic legacy",
                                "Import accounts 12 word mnemonic used by legacy applications (Spectre Desktop and web wallet)",
                            ),
                            (
                                "account import mnemonic multisig [additional keys]",
                                "Import mnemonic and additional keys for a multisig account",
                            ),
                        ],
                        None,
                    )?;
                    return Ok(());
                }

                let import_kind = argv.remove(0);
                match import_kind.as_ref() {
                    "legacy-data" => {
                        if !argv.is_empty() {
                            tprintln!(ctx, "Usage: 'account import legacy-data'");
                            tprintln!(ctx, "Too many arguments: {}\r\n", argv.join(" "));
                            return Ok(());
                        }

                        if exists_legacy_v0_keydata().await? {
                            let import_secret = Secret::new(
                                ctx.term()
                                    .ask(true, "Enter the password for the account you are importing: ")
                                    .await?
                                    .trim()
                                    .as_bytes()
                                    .to_vec(),
                            );
                            let wallet_secret =
                                Secret::new(ctx.term().ask(true, "Enter wallet password: ").await?.trim().as_bytes().to_vec());
                            let ctx_ = ctx.clone();
                            wallet
                                .import_legacy_keydata(
                                    &import_secret,
                                    &wallet_secret,
                                    None,
                                    Some(Arc::new(move |processed: usize, _, balance, txid| {
                                        if let Some(txid) = txid {
                                            tprintln!(
                                                ctx_,
                                                "Scan detected {} SPR at index {}; transfer txid: {}",
                                                sompi_to_spectre_string(balance),
                                                processed,
                                                txid
                                            );
                                        } else if processed > 0 {
                                            tprintln!(
                                                ctx_,
                                                "Scanned {} derivations, found {} SPR",
                                                processed,
                                                sompi_to_spectre_string(balance)
                                            );
                                        } else {
                                            tprintln!(ctx_, "Please wait... scanning for account UTXOs...");
                                        }
                                    })),
                                )
                                .await?;
                        } else if application_runtime::is_web() {
                            return Err("Web wallet storage not found at this domain name".into());
                        } else {
                            return Err("Spectre Desktop keydata file not found".into());
                        }
                    }
                    "mnemonic" => {
                        if argv.is_empty() {
                            tprintln!(ctx, "Usage: 'account import mnemonic <bip32|legacy|multisig>'");
                            tprintln!(ctx, "Please specify the mnemonic type");
                            tprintln!(ctx, "Use 'legacy' for 12-word Spectre Desktop and web wallet mnemonics\r\n");
                            return Ok(());
                        }

                        let account_kind = argv.remove(0);
                        let account_kind = account_kind.parse::<AccountKind>()?;

                        match account_kind.as_ref() {
                            LEGACY_ACCOUNT_KIND | BIP32_ACCOUNT_KIND => {
                                if !argv.is_empty() {
                                    tprintln!(ctx, "Too many arguments: {}\r\n", argv.join(" "));
                                    return Ok(());
                                }
                                crate::wizards::import::import_with_mnemonic(&ctx, account_kind, &argv).await?;
                            }
                            MULTISIG_ACCOUNT_KIND => {
                                crate::wizards::import::import_with_mnemonic(&ctx, account_kind, &argv).await?;
                            }
                            _ => {
                                tprintln!(ctx, "Account import is not supported for this account type: '{}'\r\n", account_kind);
                                return Ok(());
                            }
                        }

                        return Ok(());
                    }
                    _ => {
                        tprintln!(ctx, "Unknown account import type: '{}'", import_kind);
                        tprintln!(ctx, "Supported import types are: 'mnemonic' or 'legacy-data'\r\n");
                        return Ok(());
                    }
                }
            }
            "scan" | "sweep" => {
                let len = argv.len();
                let mut start = 0;
                let mut count = 100_000;
                let window = 128;
                if len >= 2 {
                    start = argv.remove(0).parse::<usize>()?;
                    count = argv.remove(0).parse::<usize>()?;
                } else if len == 1 {
                    count = argv.remove(0).parse::<usize>()?;
                }

                count = count.max(1);

                let sweep = action.eq("sweep");

                self.derivation_scan(&ctx, start, count, window, sweep).await?;
            }
            v => {
                tprintln!(ctx, "Unknown command: '{}'\r\n", v);
                return self.display_help(ctx, argv).await;
            }
        }

        Ok(())
    }

    async fn display_help(self: Arc<Self>, ctx: Arc<SpectreCli>, _argv: Vec<String>) -> Result<()> {
        ctx.term().help(
            &[
                ("create [<type>] [<name>]", "Create a new account (types: 'bip32' (default), 'legacy', 'multisig')"),
                (
                    "import <import-type> [<key-type> [extra keys]]",
                    "Import accounts from a private key using 24 or 12 word mnemonic or legacy data \
                (Spectre Desktop and web wallet). Use 'account import' for additional help.",
                ),
                ("name <name>", "Name or rename the selected account (use 'remove' to remove the name)"),
                ("scan [<derivations>] or scan [<start>] [<derivations>]", "Scan extended address derivation chain (legacy accounts)"),
                (
                    "sweep [<derivations>] or sweep [<start>] [<derivations>]",
                    "Sweep extended address derivation chain (legacy accounts)",
                ),
            ],
            None,
        )?;

        Ok(())
    }

    async fn derivation_scan(
        self: &Arc<Self>,
        ctx: &Arc<SpectreCli>,
        start: usize,
        count: usize,
        window: usize,
        sweep: bool,
    ) -> Result<()> {
        let account = ctx.account().await?;
        let (wallet_secret, payment_secret) = ctx.ask_wallet_secret(Some(&account)).await?;
        let _ = ctx.notifier().show(Notification::Processing).await;
        let abortable = Abortable::new();
        let ctx_ = ctx.clone();

        let account = account.as_derivation_capable()?;

        account
            .derivation_scan(
                wallet_secret,
                payment_secret,
                start,
                start + count,
                window,
                sweep,
                &abortable,
                Some(Arc::new(move |processed: usize, _, balance, txid| {
                    if let Some(txid) = txid {
                        tprintln!(
                            ctx_,
                            "Scan detected {} SPR at index {}; transfer txid: {}",
                            sompi_to_spectre_string(balance),
                            processed,
                            txid
                        );
                    } else {
                        tprintln!(ctx_, "Scanned {} derivations, found {} SPR", processed, sompi_to_spectre_string(balance));
                    }
                })),
            )
            .await?;

        Ok(())
    }
}
