use std::collections::HashMap;

use actix::{Actor, Context, Handler, Message, ResponseActFuture, SystemService};
use tracing::info;

use crate::command::{Command, CommandResult};

#[derive(Default)]
pub struct PipelineManager {
    /// All nodes by id
    nodes: HashMap<String, Pipeline>,
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

struct Pipeline {}

impl Actor for PipelineManager {
    type Context = Context<Self>;
}

impl actix::Supervised for PipelineManager {}

impl SystemService for PipelineManager {
    fn service_started(&mut self, _ctx: &mut Context<Self>) {
        info!("Node manager coming online");
    }
}

impl PipelineManager {}

impl Handler<CommandMessage> for PipelineManager {
    type Result = ResponseActFuture<Self, CommandResult>;

    fn handle(&mut self, msg: CommandMessage, ctx: &mut Self::Context) -> Self::Result {
        match msg.command {
            Command::Ping {} => Box::pin(actix::fut::ready(CommandResult::Success)),
            Command::Start { id } => todo!(),
            Command::Stop { id } => todo!(),
            Command::GetInfo { id } => todo!(),
        }
    }
}
