use wg_2024::network::NodeId;

use colored::Colorize;
use log::{error, info};

use messages::server_commands::ContentServerCommand;

use crate::SimulationController;

impl SimulationController {
    #[allow(clippy::too_many_lines)]
    pub fn handle_text_server_command(
        &mut self,
        text_server: &NodeId,
        command: ContentServerCommand,
    ) {
        match command {
            ContentServerCommand::InitFlooding => {
                // Get server's sender channel
                if let Some((server, _)) = self.text_servers.get(text_server) {
                    match server.send(ContentServerCommand::InitFlooding) {
                        Ok(()) => {
                            info!(
                                "[ {} ]: sent a ContentServerCommand::InitFlooding to [ TextContentServer {} ]",
                                "Simulation Controller".green(),
                                text_server
                            );
                        }
                        Err(e) => {
                            error!(
                                "[ {} ]: failed to send a ContentServerCommand::InitFlooding to the [ TextContentServer {} ]: {}",
                                "Simulation Controller".red(),
                                text_server,
                                e
                            );
                        }
                    }
                } else {
                    error!(
                        "[ {} ]: failed to find a Sender<ContentServerCommand> channel for the [ TextContentServer {} ]",
                        "Simulation Controller".red(),
                        text_server
                    );
                }
            }
            ContentServerCommand::AddSender(node_id, sender) => {
                // Get server's sender channel
                if let Some((server, _)) = self.text_servers.get(text_server) {
                    // get neighbors
                    if let Some(vec) = self.neighbor.get_mut(text_server) {
                        // Add node to neighbor
                        vec.push(node_id);

                        match server.send(ContentServerCommand::AddSender(node_id, sender)) {
                            Ok(()) => {
                                // Send command to GUI

                                // Launch global flooding
                                self.global_flooding();

                                info!(
                                    "[ {} ]: sent a ContentServerCommand::AddSender({}, sender_channel) to [ TextContentServer {} ]",
                                    "Simulation Controller".green(),
                                    node_id,
                                    text_server
                                );
                            }
                            Err(e) => {
                                error!(
                                    "[ {} ]: failed to send a ContentServerCommand::AddSender({}, sender_channel) to the [ TextContentServer {} ]: {}",
                                    "Simulation Controller".red(),
                                    node_id,
                                    text_server,
                                    e
                                );
                            }
                        }
                    } else {
                        error!(
                            "[ {} ]: the [ TextContentServer {} ] does not have any neighbor",
                            "Simulation Controller".red(),
                            text_server
                        );
                    }
                } else {
                    error!(
                        "[ {} ]: failed to find a Sender<ContentServerCommand> channel for the [ TextContentServer {} ]",
                        "Simulation Controller".red(),
                        text_server
                    );
                }
            }
            ContentServerCommand::RemoveSender(node_id) => {
                // Get server's sender channel
                if let Some((server, _)) = self.text_servers.get(text_server) {
                    //Get neighbors
                    if let Some(vec) = self.neighbor.get_mut(text_server) {
                        // Remove node from neighbor
                        vec.retain(|x| *x != node_id);

                        match server.send(ContentServerCommand::RemoveSender(node_id)) {
                            Ok(()) => {

                                // send command to GUI

                                // Launch global flooding
                                self.global_flooding();

                                info!(
                                    "[ {} ]: sent a ContentServerCommand::RemoveSender({}) to [ TextContentServer {} ]",
                                    "Simulation Controller".green(),
                                    node_id,
                                    text_server
                                );
                            },
                            Err(e) => error!(
                                "[ {} ]: failed to send a ContentServerCommand::RemoveSender({}) to the [ TextContentServer {} ]: {}",
                                "Simulation Controller".red(),
                                node_id,
                                text_server,
                                e
                            ),
                        }
                    } else {
                        error!(
                            "[ {} ]: the [ TextContentServer {} ] does not have any neighbor",
                            "Simulation Controller".red(),
                            text_server
                        );
                    }
                } else {
                    error!(
                        "[ {} ]: failed to find a Sender<ContentServerCommand> channel for the [ TextContentServer {} ]",
                        "Simulation Controller".red(),
                        text_server
                    );
                }
            }
        }
    }
}
