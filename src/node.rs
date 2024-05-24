use actix::{
    Actor, ActorFuture, ActorFutureExt, Addr, Context, Handler, Message, MessageResult,
    ResponseActFuture, ResponseFuture, SystemService, WeakRecipient, WrapFuture,
};
use anyhow::Error;
use std::collections::HashMap;
use tracing::{debug, info, instrument, trace};
use tracing_actix::ActorInstrument;

use crate::command::{Command, CommandResult, Info, VideoMode};
use crate::controller::State;
use crate::pipeline::decklink::DecklinkStream;

#[derive(Default)]
pub struct NodeManager {
    /// All nodes by id
    nodes: HashMap<i16, Addr<DecklinkStream>>,
    /// Any listeners to events
    listeners: HashMap<String, WeakRecipient<NodeStatusMessage>>,
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
    }
}

impl NodeManager {
    fn sync_info_future(
        &self,
    ) -> std::pin::Pin<Box<dyn ActorFuture<NodeManager, Output = CommandResult>>> {
        let nodes = vec!["test".to_string(), "test".to_string()];

        Box::pin(actix::fut::ready(CommandResult::Sync(Info {
            devices: nodes,
        })))
    }

    fn start_source(
        &mut self,
        device_num: &i16,
        _mode: &crate::command::VideoMode,
    ) -> ResponseActFuture<Self, CommandResult> {
        if self.nodes.contains_key(device_num) {
            return Box::pin(actix::fut::ready(CommandResult::Error(format!(
                "A node already exists with id {}",
                device_num
            ))));
        }

        let stream = match DecklinkStream::new(device_num) {
            Ok(stream) => stream,
            Err(err) => {
                return Box::pin(actix::fut::ready(CommandResult::Error(format!(
                    "Failed to start {}",
                    err
                ))));
            }
        };

        let addr = stream.start();

        self.nodes.insert(device_num.clone(), addr.clone());

        Box::pin(
            {
                async move { addr.recipient().send(StartMessage {}).await }
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
    }

    fn remove_node(&mut self, id: &i16) {
        let _ = self.nodes.remove(id);
    }

    /// Tell a node to stop, by id
    fn stop_source(&mut self, device_num: &i16) -> CommandResult {
        if let Some(node) = self.nodes.get_mut(device_num) {
            node.clone().recipient().do_send(StopMessage);
            CommandResult::Success
        } else {
            CommandResult::Error(format!("No node with id {}", device_num))
        }
    }

    fn notify_listeners(&mut self, message: NodeStatusMessage) {
        self.listeners.retain(|_id, recipient| {
            if let Some(recipient) = recipient.upgrade() {
                recipient.do_send(message.clone());
                true
            } else {
                false
            }
        })
    }
}

impl Handler<CommandMessage> for NodeManager {
    type Result = ResponseActFuture<Self, CommandResult>;

    fn handle(&mut self, msg: CommandMessage, _ctx: &mut Self::Context) -> Self::Result {
        match msg.command {
            Command::Ping {} => Box::pin(actix::fut::ready(CommandResult::Pong)),
            Command::Start { device_num } => {
                self.start_source(&device_num, &VideoMode::TestCard("test".to_string()))
            }
            Command::Stop { device_num } => {
                Box::pin(actix::fut::ready(self.stop_source(&device_num)))
            }
            Command::Sync {} => self.sync_info_future(),
        }
    }
}

impl Handler<NodeStatusMessage> for NodeManager {
    type Result = ();

    #[instrument(level = "trace", name = "notifying listeners", skip(self, _ctx))]
    fn handle(&mut self, msg: NodeStatusMessage, _ctx: &mut Context<Self>) -> Self::Result {
        self.notify_listeners(msg)
    }
}

/// Sent from [`Node`] to [`NodeManager`] so that it can inform listeners
/// of nodes' status
#[derive(Debug, Clone)]
pub enum NodeStatusMessage {
    /// Node state changed
    State { id: String, state: State },
    /// Node encountered an error
    Error { id: String, message: String },
}

impl Message for NodeStatusMessage {
    type Result = ();
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
        self.remove_node(&msg.id);

        debug!("node {} removed from NodeManager", msg.id);

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
    pub id: i16,
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
