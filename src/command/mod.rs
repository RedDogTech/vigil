use std::collections::HashMap;

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

/// Command variants
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Command {
    Ping {},
    /// Reschedule any node
    Start {
        /// Identifier of an existing node
        id: String,
    },
    /// Remove a node
    Stop {
        /// Identifier of an existing node
        id: String,
    },
    /// Retrieve the info of one or all nodes
    GetInfo {
        /// The id of an existing node, or None, in which case the info
        /// of all nodes in the system will be gathered
        id: Option<String>,
    },
}

/// A map of node-specific information in reply to a GetInfo command
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub struct Info {
    pub nodes: HashMap<String, String>,
}

/// Messages sent from the the server to the controller.
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum CommandResult {
    /// The command resulted in an error
    Error(String),
    /// The command was successful
    Success,
    /// Information about one or all nodes
    Info(Info),
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
