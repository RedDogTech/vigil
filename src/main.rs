use anyhow::Error;
use gstreamer as gst;
use tracing_subscriber::layer::SubscriberExt;

mod command;
mod controller;
mod device;
mod node;
mod pipeline;
mod server;

fn main() -> Result<(), Error> {
    tracing_log::LogTracer::init().expect("Failed to set logger");
    let env_filter = tracing_subscriber::EnvFilter::try_from_env("VIGIL_LOG")
        .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("warn"));

    let fmt_layer = tracing_subscriber::fmt::layer()
        .with_thread_ids(true)
        .with_target(true)
        .with_span_events(
            tracing_subscriber::fmt::format::FmtSpan::NEW
                | tracing_subscriber::fmt::format::FmtSpan::CLOSE,
        );

    let subscriber = tracing_subscriber::Registry::default()
        .with(env_filter)
        .with(fmt_layer);
    tracing::subscriber::set_global_default(subscriber).expect("Failed to set subscriber");

    gst::init()?;

    //find_decklink_devices()?;

    let system = actix_rt::System::new();
    system.block_on(server::run())?;
    Ok(())
}
