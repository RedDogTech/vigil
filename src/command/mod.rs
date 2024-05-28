use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Messages sent from the controller to the server.
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub struct ControllerMessage {
    /// Identifier of the command
    pub id: uuid::Uuid,
    /// The command to run
    pub command: Command,
}

/// Messages sent from the controller to the server.
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum VideoMode {
    TestCard(String),
    Stream(String),
}

/// Command variants
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Command {
    Ping {},
    Start { device_id: Uuid },
    Stop { device_id: Uuid },
}

/// A map of node-specific information in reply to a GetInfo command
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "lowercase")]
pub struct Device {
    pub id: Uuid,
    pub device_num: u16,
    pub state: gstreamer::State,
}

/// Messages sent from the the server to the controller.
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum CommandResult {
    /// The command resulted in an error
    Error(String),
    /// The command was successful
    Success,
    ///
    Pong,
    /// Information about one or all nodes
    Sync(Vec<Device>),
}

/// Messages sent from the the server to the controller.
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub struct ServerMessage {
    /// Identifier of the command result
    pub id: Option<uuid::Uuid>,
    /// The command result
    pub result: CommandResult,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
/// Mixer-specific information
pub struct NodeState {
    pub state: gstreamer::State,
}
