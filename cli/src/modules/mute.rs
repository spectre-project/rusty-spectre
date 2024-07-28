use crate::imports::*;

#[derive(Default, Handler)]
#[help("Toggle the notification output mute state on or off")]
pub struct Mute;

impl Mute {
    async fn main(self: Arc<Self>, ctx: &Arc<dyn Context>, _argv: Vec<String>, _cmd: &str) -> Result<()> {
        let ctx = ctx.clone().downcast_arc::<SpectreCli>()?;
        tprintln!(ctx, "Mute state is now {}", ctx.toggle_mute());
        Ok(())
    }
}
