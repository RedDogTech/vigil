use anyhow::Error;
use gstreamer as gst;

mod server;

fn main() -> Result<(), Error> {
    gst::init()?;

    let system = actix_rt::System::new();
    system.block_on(server::run())?;
    Ok(())
}
