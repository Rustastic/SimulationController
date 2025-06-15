use wg_2024::network::NodeId;

use colored::Colorize;
use log::{error, info};

use messages::server_commands::ContentServerCommand;

use crate::SimulationController;

impl SimulationController {
    #[allow(clippy::too_many_lines)]
    pub fn handle_media_server_command(
        &mut self,
        media_server: &NodeId,
        command: ContentServerCommand,
    ) {
        match command {
            ContentServerCommand::InitFlooding => {
                // Get server's sender channel
                if let Some((server, _)) = self.media_servers.get(media_server) {
                    // Send command and handle Result
                    match server.send(ContentServerCommand::InitFlooding) {
                        Ok(()) => {
                            info!(
                                "[ {} ]: sent a ContentServerCommand::InitFlooding to [ MediaContentServer {} ]",
                                "Simulation Controller".green(),
                                media_server
                            );
                        }
                        Err(e) => {
                            error!(
                                "[ {} ]: failed to send a ContentServerCommand::InitFlooding to the [ MediaContentServer {} ]: {}",
                                "Simulation Controller".red(),
                                media_server,
                                e
                            );
                        }
                    }
                } else {
                    error!(
                        "[ {} ]: failed to find a Sender<ContentServerCommand> channel for the [ MediaContentServer {} ]",
                        "Simulation Controller".red(),
                        media_server
                    );
                }
            }
            ContentServerCommand::AddSender(node_id, sender) => {
                // Get server's sender channel
                if let Some((server, _)) = self.media_servers.get(media_server) {
                    // Get neighbors
                    if let Some(vec) = self.neighbor.get_mut(media_server) {
                        // Add node to neighbor
                        vec.push(node_id);
                        // Send command and handle Result
                        match server.send(ContentServerCommand::AddSender(node_id, sender)) {
                            Ok(()) => {
                                // Send command to GUI
                                self.gui_send.send(GUIEvents::AddSender(media_server, node_id));

                                // Launch global flooding
                                self.global_flooding();

                                info!(
                                    "[ {} ]: sent a ContentServerCommand::AddSender({}, sender_channel) to [ MediaContentServer {} ]",
                                    "Simulation Controller".green(),
                                    node_id,
                                    media_server
                                );
                            }
                            Err(e) => {
                                error!(
                                    "[ {} ]: failed to send a ContentServerCommand::AddSender({}, sender_channel) to the [ MediaContentServer {} ]: {}",
                                    "Simulation Controller".red(),
                                    node_id,
                                    media_server,
                                    e
                                );
                            }
                        }
                    } else {
                        error!(
                            "[ {} ]: the [ MediaContentServer {} ] does not have any neighbor",
                            "Simulation Controller".red(),
                            media_server
                        );
                    }
                } else {
                    error!(
                        "[ {} ]: failed to find a Sender<ContentServerCommand> channel for the [ MediaContentServer {} ]",
                        "Simulation Controller".red(),
                        media_server
                    );
                }
            }
            ContentServerCommand::RemoveSender(node_id) => {
                // Get server's sender channel
                if let Some((server, _)) = self.media_servers.get(media_server) {
                    // Get neighbor
                    if let Some(vec) = self.neighbor.get_mut(media_server) {
                        // Remove node from neighbor
                        vec.retain(|x| *x != node_id);
                        // Send command and handle Result
                        match server.send(ContentServerCommand::RemoveSender(node_id)) {
                            Ok(()) => {
                                // Send command to gui
                                self.gui_send.send(GUIEvents::RemoveSender(media_server, node_id));

                                // Launch global flooding
                                self.global_flooding();

                                info!(
                                    "[ {} ]: sent a ContentServerCommand::RemoveSender({}) to [ MediaContentServer {} ]",
                                    "Simulation Controller".green(),
                                    node_id,
                                    media_server
                                );
                            },
                            Err(e) => error!(
                                "[ {} ]: failed to send a ContentServerCommand::RemoveSender({}) to the [ MediaContentServer {} ]: {}",
                                "Simulation Controller".red(),
                                node_id,
                                media_server,
                                e
                            ),
                        }
                    } else {
                        error!(
                            "[ {} ]: the [ MediaContentServer {} ] does not have any neighbor",
                            "Simulation Controller".red(),
                            media_server
                        );
                    }
                } else {
                    error!(
                        "[ {} ]: failed to find a Sender<ContentServerCommand> channel for the [ MediaContentServer {} ]",
                        "Simulation Controller".red(),
                        media_server
                    );
                }
            }
        }
    }
}
