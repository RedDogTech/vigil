use anyhow::{anyhow, Result};
use gst::prelude::DeviceProviderExtManual;
use gstreamer as gst;

pub fn find_decklink_devices() -> Result<u16> {
    if let Some(provider) = gst::DeviceProviderFactory::by_name("decklinkvideosink") {
        let devices = provider.devices();

        tracing::debug!("Found nume `{:?}` devices", devices.len());
    }

    Err(anyhow!("No decklink devices found"))
}
