use serde::{Deserialize, Serialize};

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
    Start { device_num: i16 },
    Stop { device_num: i16 },
    Sync {},
}

/// A map of node-specific information in reply to a GetInfo command
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub struct Info {
    pub devices: Vec<String>,
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
    Sync(Info),
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
