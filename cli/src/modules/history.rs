use crate::imports::*;
use spectre_consensus_core::tx::TransactionId;
use spectre_wallet_core::error::Error as WalletError;
use spectre_wallet_core::storage::Binding;

#[derive(Default, Handler)]
#[help("Display transaction history")]
pub struct History;

impl History {
    async fn main(self: Arc<Self>, ctx: &Arc<dyn Context>, mut argv: Vec<String>, _cmd: &str) -> Result<()> {
        let ctx = ctx.clone().downcast_arc::<SpectreCli>()?;

        if argv.is_empty() {
            self.display_help(ctx, argv).await?;
            return Ok(());
        }

        let account = ctx.account().await?;
        let network_id = ctx.wallet().network_id()?;
        let binding = Binding::from(&account);
        let current_daa_score = ctx.wallet().current_daa_score();

        let (last, include_utxo) = match argv.remove(0).as_str() {
            "lookup" => {
                if argv.is_empty() {
                    tprintln!(ctx, "Usage: history lookup <transaction id>");
                    return Ok(());
                }
                let transaction_id = argv.remove(0);
                let txid = TransactionId::from_hex(transaction_id.as_str())?;
                let store = ctx.wallet().store().as_transaction_record_store()?;
                match store.load_single(&binding, &network_id, &txid).await {
                    Ok(tx) => {
                        let lines = tx
                            .format_transaction_with_args(&ctx.wallet(), None, current_daa_score, true, true, Some(account.clone()))
                            .await;
                        lines.iter().for_each(|line| tprintln!(ctx, "{line}"));
                    }
                    Err(_) => {
                        tprintln!(ctx, "Transaction not found");
                    }
                }
                return Ok(());
            }
            "list" => {
                let last = if argv.is_empty() { None } else { argv[0].parse::<usize>().ok() };
                (last, false)
            }
            "details" => {
                let last = if argv.is_empty() { None } else { argv[0].parse::<usize>().ok() };
                (last, true)
            }
            v => {
                tprintln!(ctx, "Unknown command: '{v}'");
                self.display_help(ctx, argv).await?;
                return Ok(());
            }
        };

        let store = ctx.wallet().store().as_transaction_record_store()?;
        let mut ids = match store.transaction_id_iter(&binding, &network_id).await {
            Ok(ids) => ids,
            Err(err) => {
                if matches!(err, WalletError::NoRecordsFound) {
                    tprintln!(ctx, "No transactions found for this account.");
                } else {
                    terrorln!(ctx, "{err}");
                }
                return Ok(());
            }
        };

        let length = ids.size_hint().0;
        let skip = if let Some(last) = last {
            if last > length {
                0
            } else {
                length - last
            }
        } else {
            0
        };
        let mut index = 0;
        let page = 25;

        tprintln!(ctx);

        while let Some(id) = ids.try_next().await? {
            if index >= skip {
                if index > 0 && index % page == 0 {
                    tprintln!(ctx);
                    let prompt = format!(
                        "Displaying transactions {} to {} of {} (press any key to continue, 'Q' to abort)",
                        index.separated_string(),
                        (index + page).separated_string(),
                        length.separated_string()
                    );
                    let query = ctx.term().kbhit(Some(&prompt)).await?;
                    tprintln!(ctx);
                    if query.to_lowercase() == "q" {
                        return Ok(());
                    }
                }

                match store.load_single(&binding, &network_id, &id).await {
                    Ok(tx) => {
                        let lines = tx
                            .format_transaction_with_args(
                                &ctx.wallet(),
                                None,
                                current_daa_score,
                                include_utxo,
                                true,
                                Some(account.clone()),
                            )
                            .await;
                        lines.iter().for_each(|line| tprintln!(ctx, "{line}"));
                    }
                    Err(err) => {
                        terrorln!(ctx, "Unable to read transaction data: {err}");
                    }
                }
            }
            index += 1;
        }

        tprintln!(ctx, "{} transactions", length.separated_string());
        tprintln!(ctx);

        Ok(())
    }

    async fn display_help(self: Arc<Self>, ctx: Arc<SpectreCli>, _argv: Vec<String>) -> Result<()> {
        ctx.term().help(
            &[
                ("list [<last N transactions>]", "List the last N transactions"),
                ("details [<last N transactions>]", "List the last N transactions with UTXO details"),
                ("lookup <transaction id>", "Look up a transaction by its ID"),
            ],
            None,
        )?;
        Ok(())
    }
}
