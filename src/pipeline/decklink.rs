use actix::prelude::*;
use actix::{Actor, Addr, Context};
use anyhow::Error;
use gst::prelude::ElementExtManual;
use gst::prelude::*;
use gstreamer as gst;
use tracing::instrument;
use tracing::{debug, error};
use uuid::Uuid;

use crate::node::{NodeManager, StartMessage, StopMessage, StoppedMessage};

use super::manager::{PipelineManager, StopManagerMessage};
use super::{make_element, ErrorMessage};

/// The pipeline and various GStreamer elements that the source
/// optionally wraps, their lifetime is not directly bound to that
/// of the source itself
#[derive(Debug)]
pub struct DecklinkStream {
    /// Unique identifier
    id: Uuid,
    /// Decklink device id num
    device_num: u16,
    /// The wrapped pipeline
    pipeline: gst::Pipeline,
    /// A helper for managing the pipeline
    pipeline_manager: Option<Addr<PipelineManager>>,
}

impl Actor for DecklinkStream {
    type Context = Context<Self>;

    #[instrument(level = "debug", name = "starting", skip(self, ctx), fields(id = %self.id))]
    fn started(&mut self, ctx: &mut Self::Context) {
        debug!("pipeline manger created");

        self.pipeline_manager = Some(
            PipelineManager::new(
                self.pipeline.clone(),
                ctx.address().downgrade().recipient(),
                self.id,
            )
            .start(),
        );
    }

    #[instrument(level = "debug", name = "stopped", skip(self, _ctx), fields(id = %self.id))]
    fn stopped(&mut self, _ctx: &mut Self::Context) {
        if let Some(manager) = self.pipeline_manager.take() {
            manager.do_send(StopManagerMessage);
        }

        NodeManager::from_registry().do_send(StoppedMessage {
            id: self.id.clone(),
        });
    }
}

impl DecklinkStream {
    pub fn new(device_id: Uuid, device_num: u16) -> Result<Self, Error> {
        let pipeline = gst::Pipeline::new();

        let src = make_element("videotestsrc", None)?;
        let sink = make_element("autovideosink", None)?;

        src.set_property("is-live", true);

        pipeline.add_many(&[&src, &sink])?;
        gst::Element::link_many(&[src, sink])?;

        Ok(Self {
            id: device_id,
            pipeline,
            pipeline_manager: None,
            device_num,
        })
    }

    /// Start our pipeline when cue_time is reached
    #[instrument(level = "debug", name = "start_pipeline", skip(self, ctx), fields(id = %self.id))]
    fn start_pipeline(&mut self, ctx: &mut Context<Self>) -> Result<(), Error> {
        let addr = ctx.address();
        let id = self.id.clone();

        self.pipeline.call_async(move |pipeline| {
            if let Err(err) = pipeline.set_state(gst::State::Playing) {
                addr.do_send(ErrorMessage(format!(
                    "Failed to start mixer {}: {}",
                    id, err
                )));
            }
        });

        Ok(())
    }
}

impl Handler<StartMessage> for DecklinkStream {
    type Result = MessageResult<StartMessage>;

    fn handle(&mut self, _: StartMessage, ctx: &mut Context<Self>) -> Self::Result {
        MessageResult(self.start_pipeline(ctx))
    }
}

impl Handler<StopMessage> for DecklinkStream {
    type Result = Result<(), Error>;

    fn handle(&mut self, _: StopMessage, ctx: &mut Context<Self>) -> Self::Result {
        ctx.stop();
        Ok(())
    }
}

impl Handler<ErrorMessage> for DecklinkStream {
    type Result = ();

    fn handle(&mut self, msg: ErrorMessage, ctx: &mut Context<Self>) -> Self::Result {
        error!("Got error message '{}' on destination {}", msg.0, self.id,);

        // NodeManager::from_registry().do_send(NodeStatusMessage::Error {
        //     id: self.id.clone(),
        //     message: msg.0,
        // });

        ctx.stop();
    }
}
