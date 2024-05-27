use actix::{
    Actor, ActorFutureExt, Addr, AsyncContext, Context, Handler, Message, MessageResult,
    ResponseActFuture, ResponseFuture, SystemService, WrapFuture,
};
use anyhow::{anyhow, Error};
use std::collections::HashMap;
use std::time::Duration;
use tracing::{info, instrument};
use tracing_actix::ActorInstrument;
use uuid::Uuid;

use crate::command::{Command, CommandResult, Device, VideoMode};
use crate::controller::{Controller, SyncMessage};
use crate::pipeline::decklink::DecklinkStream;

#[derive(Default)]
pub struct NodeManager {
    /// All nodes by id
    nodes: HashMap<Uuid, Addr<DecklinkStream>>,
    ///
    devices: HashMap<Uuid, Device>,
    /// connected socket sessions
    sessions: HashMap<Uuid, Addr<Controller>>,
}

/// Sent from [`controllers`](crate::controller::Controller), this is our
/// public interface.
#[derive(Debug)]
pub struct CommandMessage {
    /// The command to run
    pub command: Command,
}

impl Message for CommandMessage {
    type Result = CommandResult;
}

impl Actor for NodeManager {
    type Context = Context<Self>;
}

impl actix::Supervised for NodeManager {}

impl SystemService for NodeManager {
    fn service_started(&mut self, ctx: &mut Context<Self>) {
        info!("Node manager coming online");

        let devices: u16 = 4;

        for device_num in 0..devices {
            let device_id = Uuid::new_v4();
            if let Ok(stream) = DecklinkStream::new(device_id, device_num) {
                let addr = stream.start();

                self.nodes.insert(device_id, addr.clone());
                self.devices.insert(
                    device_id,
                    Device {
                        id: device_id,
                        device_num,
                        state: gstreamer::State::Null,
                    },
                );
            }
        }

        ctx.run_interval(Duration::from_secs(2), |act, _| {
            let sessions = act.sessions.clone();
            for (_, controller) in sessions.into_iter() {
                let devices = act.devices.clone();
                controller.do_send(SyncMessage {
                    device: devices.values().cloned().collect(),
                });
            }
        });
    }
}

impl NodeManager {
    fn start_source(
        &mut self,
        device_id: &Uuid,
        _mode: &crate::command::VideoMode,
    ) -> ResponseActFuture<Self, CommandResult> {
        if let Some(node) = self.nodes.get(device_id) {
            let node = node.clone();
            if let Some(device) = self.devices.get_mut(device_id) {
                device.state = gstreamer::State::Playing;
            };
            Box::pin(
                {
                    async move {
                        match node.recipient().send(StartMessage {}).await {
                            Ok(res) => res,
                            Err(err) => Err(anyhow!("Internal server error {}", err)),
                        }
                    }
                    .into_actor(self)
                    .then(move |res, _slf, _ctx| {
                        actix::fut::ready(match res {
                            Ok(_) => CommandResult::Success,
                            Err(err) => CommandResult::Error(format!("{}", err)),
                        })
                    })
                }
                .in_current_actor_span(),
            )
        } else {
            Box::pin(actix::fut::ready(CommandResult::Error(format!(
                "No node with id {}",
                device_id
            ))))
        }
    }

    /// Tell a node to stop, by id
    fn stop_source(&mut self, device_id: &Uuid) -> CommandResult {
        if let Some(node) = self.nodes.get_mut(device_id) {
            node.clone().recipient().do_send(StopMessage);

            if let Some(device) = self.devices.get_mut(device_id) {
                device.state = gstreamer::State::Null;
            }

            CommandResult::Success
        } else {
            CommandResult::Error(format!("No node with id {}", device_id))
        }
    }
}

impl Handler<CommandMessage> for NodeManager {
    type Result = ResponseActFuture<Self, CommandResult>;

    fn handle(&mut self, msg: CommandMessage, _ctx: &mut Self::Context) -> Self::Result {
        match msg.command {
            Command::Ping {} => Box::pin(actix::fut::ready(CommandResult::Pong)),
            Command::Start { device_id } => {
                self.start_source(&device_id, &VideoMode::TestCard("test".to_string()))
            }
            Command::Stop { device_id } => {
                Box::pin(actix::fut::ready(self.stop_source(&device_id)))
            }
        }
    }
}

#[derive(Debug)]
pub struct StopMessage;

impl Message for StopMessage {
    type Result = Result<(), Error>;
}

impl Handler<StoppedMessage> for NodeManager {
    type Result = MessageResult<StoppedMessage>;

    #[instrument(level = "debug", name = "removing-node", skip(self, _ctx, msg), fields(id = %msg.id))]
    fn handle(&mut self, msg: StoppedMessage, _ctx: &mut Context<Self>) -> Self::Result {
        MessageResult(())
    }
}

impl Handler<StopMessage> for NodeManager {
    type Result = ResponseFuture<Result<(), Error>>;

    #[instrument(level = "info", name = "stopping manager", skip(self, _ctx, _msg))]
    fn handle(&mut self, _msg: StopMessage, _ctx: &mut Context<Self>) -> Self::Result {
        for (_id, node) in self.nodes.iter_mut() {
            node.clone().recipient().do_send(StopMessage);
        }

        Box::pin(async move {
            info!("Stopped all nodes");

            Ok(())
        });
    }
}

/// A node has stopped, sent from any node to [`NodeManager`]
#[derive(Debug)]
pub struct StoppedMessage {
    /// Unique identifier of the node
    pub id: Uuid,
}

impl Message for StoppedMessage {
    type Result = ();
}

/// Start a node, sent from [`NodeManager`] to any [`Node`]
#[derive(Debug)]
pub struct StartMessage {}

impl Message for StartMessage {
    type Result = Result<(), Error>;
}

#[derive(Debug, Clone)]
pub enum WebsocketMessage {
    /// Node state changed
    Connection { id: Uuid, addr: Addr<Controller> },
    /// Node encountered an error
    Disconect { id: Uuid },
}

impl Message for WebsocketMessage {
    type Result = Result<(), Error>;
}

impl Handler<WebsocketMessage> for NodeManager {
    type Result = MessageResult<WebsocketMessage>;

    fn handle(&mut self, msg: WebsocketMessage, _ctx: &mut Self::Context) -> Self::Result {
        match msg {
            WebsocketMessage::Connection { id, addr } => {
                self.sessions.insert(id, addr);
                //Ok(())
            }
            WebsocketMessage::Disconect { id } => {
                self.sessions.remove(&id);
                //Ok(())
            }
        }

        MessageResult(Ok(()))
    }
}
