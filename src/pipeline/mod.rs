use actix::Message;
use anyhow::{anyhow, Error};
use gstreamer as gst;

pub mod decklink;
pub mod manager;

/// Wrapper around `gst::ElementFactory::make` with a better error
/// message
pub fn make_element(element: &str, name: Option<&str>) -> Result<gst::Element, Error> {
    gst::ElementFactory::make_with_name(element, name)
        .map_err(|err| anyhow!("Failed to make element {}: {}", element, err.message))
}

/// Sent from [`PipelineManager`] to nodes to signal an error
#[derive(Debug)]
pub struct ErrorMessage(pub String);

impl Message for ErrorMessage {
    type Result = ();
}

pub struct Pipeline {}
