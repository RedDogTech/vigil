use std::time::{Duration, Instant};

use actix::prelude::*;
use actix_web::dev::ConnectionInfo;
use actix_web_actors::ws;
use anyhow::{format_err, Error};
use serde::{Deserialize, Serialize};
use tracing::{debug, error, info, instrument, trace};

use crate::{
    command::{Command, CommandResult, ControllerMessage, ServerMessage},
    node::{CommandMessage, NodeManager},
};

#[derive(Debug)]
pub struct Controller {
    /// Address of the remote controller
    remote_addr: String,
    //Heartbeat listener
    heart_beat: Instant,
}

/// The state of a node
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum State {
    /// The node is not running yet
    Initial,
    /// The node is preparing
    Starting,
    /// The node is playing
    Started,
    /// The node is stopping
    Stopping,
    /// The node has stopped
    Stopped,
}

/// How often heartbeat pings are sent
const HEARTBEAT_INTERVAL: Duration = Duration::from_secs(5);
/// How long before lack of client response causes a timeout
const CLIENT_TIMEOUT: Duration = Duration::from_secs(10);

impl Controller {
    /// Create a new `Controller` actor.
    pub fn new(connection_info: &ConnectionInfo) -> Result<Self, Error> {
        debug!("Creating new controller {:?}", connection_info);

        let remote_addr = connection_info
            .realip_remote_addr()
            .ok_or_else(|| format_err!("WebSocket connection without remote address"))?;

        Ok(Controller {
            remote_addr: String::from(remote_addr),
            heart_beat: Instant::now(),
        })
    }

    /// Send a command to [`NodeManager`] for dispatching, then notify
    /// the remote controller
    fn send_command_future(
        &self,
        command_id: uuid::Uuid,
        command: Command,
    ) -> impl ActorFuture<Self, Output = ()> {
        let node_manager = NodeManager::from_registry();

        async move { node_manager.send(CommandMessage { command }).await }
            .into_actor(self)
            .then(move |res, _, ctx| {
                match res {
                    Ok(res) => {
                        ctx.text(
                            serde_json::to_string(&ServerMessage {
                                id: Some(command_id),
                                result: res,
                            })
                            .expect("failed to serialize CommandResult message"),
                        );
                    }
                    Err(err) => {
                        ctx.notify(ErrorMessage {
                            msg: format!("Internal server error: {}", err),
                            command_id: Some(command_id),
                        });
                    }
                }

                actix::fut::ready(())
            })
    }

    /// Handle JSON messages from the controller.
    #[instrument(level = "trace", name = "controller-message", skip(self, ctx))]
    fn handle_message(&mut self, ctx: &mut ws::WebsocketContext<Self>, text: &str) {
        trace!("Handling message: {}", text);
        match serde_json::from_str::<ControllerMessage>(text) {
            Ok(ControllerMessage { id, command }) => {
                ctx.spawn(self.send_command_future(id, command));
            }
            Err(err) => {
                error!(
                    "Controller {} has websocket error: {}",
                    self.remote_addr, err
                );
                ctx.notify(ErrorMessage {
                    msg: String::from("Internal processing error"),
                    command_id: None,
                });
            }
        }
    }

    /// Shut down the controller
    fn shutdown(&mut self, ctx: &mut ws::WebsocketContext<Self>, from_close: bool) {
        debug!("Shutting down controller {}", self.remote_addr);

        if !from_close {
            ctx.close(None);
        }
        ctx.stop();
    }

    fn heatbeat(&self, ctx: &mut <Self as Actor>::Context) {
        ctx.run_interval(HEARTBEAT_INTERVAL, |act, ctx| {
            // check client heartbeats
            if Instant::now().duration_since(act.heart_beat) > CLIENT_TIMEOUT {
                info!("Websocket Client heartbeat failed, disconnecting!");
                ctx.stop();
                return;
            }
            ctx.ping(b"");
        });
    }
}

impl Actor for Controller {
    type Context = ws::WebsocketContext<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        let node_manager = NodeManager::from_registry();
        let addr = ctx.address();
        self.heatbeat(ctx);
    }

    fn stopped(&mut self, ctx: &mut Self::Context) {
        let node_manager = NodeManager::from_registry();
        let addr = ctx.address();
    }
}

impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for Controller {
    fn handle(&mut self, msg: Result<ws::Message, ws::ProtocolError>, ctx: &mut Self::Context) {
        match msg {
            Ok(ws::Message::Ping(msg)) => {
                self.heart_beat = Instant::now();
                ctx.pong(&msg);
            }
            Ok(ws::Message::Pong(_)) => {
                self.heart_beat = Instant::now();
            }
            Ok(ws::Message::Text(text)) => {
                self.handle_message(ctx, &text);
            }
            Ok(ws::Message::Close(reason)) => {
                debug!(
                    "Controller {} websocket connection closed: {:?}",
                    self.remote_addr, reason
                );
                self.shutdown(ctx, true);
            }
            Ok(ws::Message::Binary(_binary)) => {
                error!("Unsupported binary message, ignoring");
            }
            Ok(ws::Message::Continuation(_)) => {
                error!("Unsupported continuation message, ignoring");
            }
            Ok(ws::Message::Nop) | Ok(ws::Message::Pong(_)) => {
                // Do nothing
            }
            Err(err) => {
                error!(
                    "Controller {} websocket connection error: {:?}",
                    self.remote_addr, err
                );
                self.shutdown(ctx, false);
            }
        }
    }
}

/// Sent from [`Controller` to itself to notify the remote controller
/// of an error.
#[derive(Debug)]
struct ErrorMessage {
    /// Error message
    msg: String,
    /// Identifier of the command that caused the error
    command_id: Option<uuid::Uuid>,
}

impl Message for ErrorMessage {
    type Result = ();
}

impl Handler<ErrorMessage> for Controller {
    type Result = ();

    fn handle(&mut self, msg: ErrorMessage, ctx: &mut ws::WebsocketContext<Self>) -> Self::Result {
        error!(
            "Got error message '{}' on controller {}",
            msg.msg, self.remote_addr
        );

        ctx.text(
            serde_json::to_string(&ServerMessage {
                id: msg.command_id,
                result: CommandResult::Error(msg.msg),
            })
            .expect("Failed to serialize error message"),
        );
    }
}

/// Sent from nodes to [`PipelineManager`] to tear it down
#[derive(Debug)]
pub struct SyncMessage;

impl Message for SyncMessage {
    type Result = ();
}

impl Handler<SyncMessage> for Controller {
    type Result = ();

    fn handle(&mut self, _msg: SyncMessage, ctx: &mut ws::WebsocketContext<Self>) -> Self::Result {
        println!("SyncMessage");
    }
}
