#![allow(unused_imports)]

use crate::imports::*;
use spectre_addresses::Prefix;
use spectre_consensus_core::tx::{TransactionOutpoint, UtxoEntry};
use spectre_wallet_core::account::pssb::finalize_psst_one_or_more_sig_and_redeem_script;
use spectre_wallet_psst::{
    prelude::{lock_script_sig_templating, script_sig_to_address, unlock_utxos_as_pssb, Bundle, Signer, PSST},
    psst::Inner,
};

#[derive(Default, Handler)]
#[help("Send a Spectre transaction to a public address")]
pub struct Pssb;

impl Pssb {
    async fn main(self: Arc<Self>, ctx: &Arc<dyn Context>, mut argv: Vec<String>, _cmd: &str) -> Result<()> {
        let ctx = ctx.clone().downcast_arc::<SpectreCli>()?;

        if !ctx.wallet().is_open() {
            return Err(Error::WalletIsNotOpen);
        }

        if argv.is_empty() {
            return self.display_help(ctx, argv).await;
        }

        let action = argv.remove(0);

        match action.as_str() {
            "create" => {
                if argv.len() < 2 || argv.len() > 3 {
                    return self.display_help(ctx, argv).await;
                }
                let (wallet_secret, payment_secret) = ctx.ask_wallet_secret(None).await?;
                let _ = ctx.notifier().show(Notification::Processing).await;

                let address = Address::try_from(argv.first().unwrap().as_str())?;
                let amount_sompi = try_parse_required_nonzero_spectre_as_sompi_u64(argv.get(1))?;
                let outputs = PaymentOutputs::from((address, amount_sompi));
                let priority_fee_sompi = try_parse_optional_spectre_as_sompi_i64(argv.get(2))?.unwrap_or(0);
                let abortable = Abortable::default();

                let account: Arc<dyn Account> = ctx.wallet().account()?;
                let signer = account
                    .pssb_from_send_generator(
                        outputs.into(),
                        priority_fee_sompi.into(),
                        None,
                        wallet_secret.clone(),
                        payment_secret.clone(),
                        &abortable,
                    )
                    .await?;

                match signer.serialize() {
                    Ok(encoded) => tprintln!(ctx, "{encoded}"),
                    Err(e) => return Err(e.into()),
                }
            }
            "script" => {
                if argv.len() < 2 || argv.len() > 4 {
                    return self.display_help(ctx, argv).await;
                }
                let subcommand = argv.remove(0);
                let payload = argv.remove(0);
                let account = ctx.wallet().account()?;
                let receive_address = account.receive_address()?;
                let (wallet_secret, payment_secret) = ctx.ask_wallet_secret(None).await?;
                let _ = ctx.notifier().show(Notification::Processing).await;

                let script_sig = match lock_script_sig_templating(payload.clone(), Some(&receive_address.payload)) {
                    Ok(value) => value,
                    Err(e) => {
                        terrorln!(ctx, "{}", e.to_string());
                        return Err(e.into());
                    }
                };

                let script_p2sh = match script_sig_to_address(&script_sig, ctx.wallet().address_prefix()?) {
                    Ok(p2sh) => p2sh,
                    Err(e) => {
                        terrorln!(ctx, "Error generating script address: {}", e.to_string());
                        return Err(e.into());
                    }
                };

                match subcommand.as_str() {
                    "lock" => {
                        let amount_sompi = try_parse_required_nonzero_spectre_as_sompi_u64(argv.first())?;
                        let outputs = PaymentOutputs::from((script_p2sh, amount_sompi));
                        let priority_fee_sompi = try_parse_optional_spectre_as_sompi_i64(argv.get(1))?.unwrap_or(0);
                        let abortable = Abortable::default();

                        let signer = account
                            .pssb_from_send_generator(
                                outputs.into(),
                                priority_fee_sompi.into(),
                                None,
                                wallet_secret.clone(),
                                payment_secret.clone(),
                                &abortable,
                            )
                            .await?;

                        match signer.serialize() {
                            Ok(encoded) => tprintln!(ctx, "{encoded}"),
                            Err(e) => return Err(e.into()),
                        }
                    }
                    "unlock" => {
                        if argv.len() != 1 {
                            return self.display_help(ctx, argv).await;
                        }

                        // Get locked UTXO set.
                        let spend_utxos: Vec<spectre_rpc_core::RpcUtxosByAddressesEntry> =
                            ctx.wallet().rpc_api().get_utxos_by_addresses(vec![script_p2sh.clone()]).await?;
                        let priority_fee_sompi = try_parse_optional_spectre_as_sompi_i64(argv.first())?.unwrap_or(0) as u64;

                        if spend_utxos.is_empty() {
                            twarnln!(ctx, "No locked UTXO set found.");
                            return Ok(());
                        }

                        let references: Vec<(UtxoEntry, TransactionOutpoint)> =
                            spend_utxos.iter().map(|entry| (entry.utxo_entry.clone().into(), entry.outpoint.into())).collect();

                        let total_locked_sompi: u64 = spend_utxos.iter().map(|entry| entry.utxo_entry.amount).sum();

                        tprintln!(
                            ctx,
                            "{} locked UTXO{} found with total amount of {} SPR",
                            spend_utxos.len(),
                            if spend_utxos.len() == 1 { "" } else { "s" },
                            sompi_to_spectre(total_locked_sompi)
                        );

                        // Sweep UTXO set.
                        match unlock_utxos_as_pssb(references, &receive_address, script_sig, priority_fee_sompi as u64) {
                            Ok(pssb) => {
                                let pssb_hex = pssb.serialize()?;
                                tprintln!(ctx, "{pssb_hex}");
                            }
                            Err(e) => tprintln!(ctx, "Error generating unlock PSSB: {}", e.to_string()),
                        }
                    }
                    "sign" => {
                        let pssb = Self::parse_input_pssb(argv.first().unwrap().as_str())?;

                        // Sign PSSB using the account's receiver address.
                        match account.pssb_sign(&pssb, wallet_secret.clone(), payment_secret.clone(), Some(&receive_address)).await {
                            Ok(signed_pssb) => {
                                let pssb_pack = String::try_from(signed_pssb)?;
                                tprintln!(ctx, "{pssb_pack}");
                            }
                            Err(e) => terrorln!(ctx, "{}", e.to_string()),
                        }
                    }
                    "address" => {
                        tprintln!(ctx, "\r\nP2SH address: {}", script_p2sh);
                    }
                    v => {
                        terrorln!(ctx, "unknown command: '{v}'\r\n");
                        return self.display_help(ctx, argv).await;
                    }
                }
            }
            "sign" => {
                if argv.len() != 1 {
                    return self.display_help(ctx, argv).await;
                }
                let (wallet_secret, payment_secret) = ctx.ask_wallet_secret(None).await?;
                let pssb = Self::parse_input_pssb(argv.first().unwrap().as_str())?;
                let account = ctx.wallet().account()?;
                match account.pssb_sign(&pssb, wallet_secret.clone(), payment_secret.clone(), None).await {
                    Ok(signed_pssb) => {
                        let pssb_pack = String::try_from(signed_pssb)?;
                        tprintln!(ctx, "{pssb_pack}");
                    }
                    Err(e) => terrorln!(ctx, "{}", e.to_string()),
                }
            }
            "send" => {
                if argv.len() != 1 {
                    return self.display_help(ctx, argv).await;
                }
                let pssb = Self::parse_input_pssb(argv.first().unwrap().as_str())?;
                let account = ctx.wallet().account()?;
                match account.pssb_broadcast(&pssb).await {
                    Ok(sent) => tprintln!(ctx, "Sent transactions {:?}", sent),
                    Err(e) => terrorln!(ctx, "Send error {:?}", e),
                }
            }
            "debug" => {
                if argv.len() != 1 {
                    return self.display_help(ctx, argv).await;
                }
                let pssb = Self::parse_input_pssb(argv.first().unwrap().as_str())?;
                tprintln!(ctx, "{:?}", pssb);
            }
            "parse" => {
                if argv.len() != 1 {
                    return self.display_help(ctx, argv).await;
                }
                let pssb = Self::parse_input_pssb(argv.first().unwrap().as_str())?;
                tprintln!(ctx, "{}", pssb.display_format(ctx.wallet().network_id()?, sompi_to_spectre_string_with_suffix));

                for (psst_index, bundle_inner) in pssb.0.iter().enumerate() {
                    tprintln!(ctx, "PSST #{:03} finalized check:", psst_index + 1);
                    let psst: PSST<Signer> = PSST::<Signer>::from(bundle_inner.to_owned());

                    let finalizer = psst.finalizer();

                    if let Ok(psst_finalizer) = finalize_psst_one_or_more_sig_and_redeem_script(finalizer) {
                        // Verify if extraction is possible.
                        match psst_finalizer.extractor() {
                            Ok(ex) => match ex.extract_tx() {
                                Ok(_) => tprintln!(
                                    ctx,
                                    "  Transaction extracted successfully: PSST is finalized with a valid script signature."
                                ),
                                Err(e) => terrorln!(ctx, "  PSST transaction extraction error: {}", e.to_string()),
                            },
                            Err(_) => twarnln!(ctx, "  PSST not finalized"),
                        }
                    } else {
                        twarnln!(ctx, "  PSST not signed");
                    }
                }
            }
            v => {
                tprintln!(ctx, "unknown command: '{v}'\r\n");
                return self.display_help(ctx, argv).await;
            }
        }
        Ok(())
    }

    fn parse_input_pssb(input: &str) -> Result<Bundle> {
        match Bundle::try_from(input) {
            Ok(bundle) => Ok(bundle),
            Err(e) => Err(Error::custom(format!("Error while parsing input PSSB {}", e))),
        }
    }

    async fn display_help(self: Arc<Self>, ctx: Arc<SpectreCli>, _argv: Vec<String>) -> Result<()> {
        ctx.term().help(
            &[
                ("pssb create <address> <amount> <priority fee>", "Create a PSSB from single send transaction"),
                ("pssb sign <pssb>", "Sign given PSSB"),
                ("pssb send <pssb>", "Broadcast bundled transactions"),
                ("pssb debug <payload>", "Print PSSB debug view"),
                ("pssb parse <payload>", "Print PSSB formatted view"),
                ("pssb script lock <payload> <amount> [priority fee]", "Generate a PSSB with one send transaction to given P2SH payload. Optional public key placeholder in payload: {{pubkey}}"),
                ("pssb script unlock <payload> <fee>", "Generate a PSSB to unlock UTXOS one by one from given P2SH payload. Fee amount will be applied to every spent UTXO, meaning every transaction. Optional public key placeholder in payload: {{pubkey}}"),
                ("pssb script sign <pssb>", "Sign all PSSB's P2SH locked inputs"),
                ("pssb script sign <pssb>", "Sign all PSSB's P2SH locked inputs"),
                ("pssb script address <pssb>", "Prints P2SH address"),
            ],
            None,
        )?;

        Ok(())
    }
}
