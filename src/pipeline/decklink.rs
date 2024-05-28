use actix::prelude::*;
use actix::{Actor, Addr, Context};
use anyhow::Error;
use gst::prelude::ElementExtManual;
use gst::prelude::*;
use gstreamer as gst;
use gstreamer_video as gst_video;
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
    device_num: i32,
    /// The wrapped pipeline
    pipeline: gst::Pipeline,
    /// A helper for managing the pipeline
    pipeline_manager: Option<Addr<PipelineManager>>,
    // node_manager: Addr<NodeManager>,
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

        // ctx.run_interval(Duration::from_secs(1), |act, _| {
        //     let state = act.pipeline.current_state();
        //     println!("current_state {:?}", state);

        //     let clock = act.pipeline.current_running_time();
        //     println!("current_running_time {:?}", clock);

        //     // act.node_manager
        //     //     .clone()
        //     //     .recipient()
        //     //     .do_send(NodeStateMessage { state });
        //     //
        //     println!("++++++++++++");
        // });
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
    pub fn new(
        _node_manager: Addr<NodeManager>,
        device_id: Uuid,
        device_num: i32,
    ) -> Result<Self, Error> {
        let pipeline = gst::Pipeline::new();

        let video_caps = gst_video::VideoCapsBuilder::new()
            .width(1920)
            .height(1080)
            .framerate((60, 1).into())
            .build();

        let video_source = gst::ElementFactory::make("videotestsrc")
            .property_from_str("pattern", "smpte")
            .property("is-live", true)
            .build()?;

        let overlay = gst::ElementFactory::make("timeoverlay")
            .property_from_str("text", format!("SDI-{} Output:\n", device_num).as_str())
            .property_from_str("halignment", "center")
            .property_from_str("valignment", "center")
            .property_from_str("font-desc", "Sans, 36")
            .build()?;

        let caps = gst::ElementFactory::make("capsfilter")
            .property("caps", &video_caps)
            .build()?;

        let timecode = gst::ElementFactory::make("timecodestamper").build()?;
        let convert = gst::ElementFactory::make("videoconvert").build()?;

        let video_sink = gst::ElementFactory::make("decklinkvideosink")
            .property_from_str("mode", "1080p60")
            .property_from_str("mapping-format", "level-a")
            .property("device-number", device_num)
            .property("sync", true)
            .build()?;

        pipeline.add_many([
            &video_source,
            &overlay,
            &caps,
            &timecode,
            &convert,
            &video_sink,
        ])?;

        gst::Element::link_many([
            &video_source,
            &overlay,
            &caps,
            &timecode,
            &convert,
            &video_sink,
        ])?;

        Ok(Self {
            id: device_id,
            pipeline,
            pipeline_manager: None,
            device_num,
            //node_manager,
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
