use crate::imports::*;

#[derive(Default, Handler)]
#[help("Select the network type: 'mainnet' or 'testnet'")]
pub struct Network;

impl Network {
    async fn main(self: Arc<Self>, ctx: &Arc<dyn Context>, argv: Vec<String>, _cmd: &str) -> Result<()> {
        let ctx = ctx.clone().downcast_arc::<SpectreCli>()?;

        if let Some(network_id) = argv.first() {
            let network_id: NetworkId = network_id.trim().parse::<NetworkId>()?;
            tprintln!(ctx, "Setting the network ID to: {network_id}");
            ctx.wallet().set_network_id(&network_id)?;
            ctx.wallet().settings().set(WalletSettings::Network, network_id).await?;
        } else {
            let network_id = ctx.wallet().network_id()?;
            tprintln!(ctx, "The current network ID is: {network_id}");
        }

        Ok(())
    }
}
