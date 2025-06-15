use colored::Colorize;
use log::{error, info};

use wg_2024::network::NodeId;

use messages::client_commands::MediaClientCommand;

use crate::SimulationController;

impl SimulationController {
    #[allow(clippy::too_many_lines)]
    pub fn handle_media_client_command(
        &mut self,
        media_client: &NodeId,
        command: MediaClientCommand,
    ) {
        match command {
            MediaClientCommand::InitFlooding => {
                // Get client's sender channel
                if let Some((client, _)) = self.mclients.get(media_client) {
                    // Send command and handle Result
                    match client.send(MediaClientCommand::InitFlooding) {
                        Ok(()) => {
                            info!(
                                "[ {} ]: sent a MediaClientCommand::InitFlo0ding to [ Client {} ]",
                                "Simulation Controller".green(),
                                media_client
                            );
                        }
                        Err(e) => {
                            error!(
                                "[ {} ]: failed to send a MediaClientCommand::InitFlooding to the [ Client {} ]: {}",
                                "Simulation Controller".red(),
                                media_client,
                                e
                            );
                        }
                    }
                } else {
                    error!(
                        "[ {} ]: failed to find a Sender<MediaClientCommand> channel for the [ Client {} ]",
                        "Simulation Controller".red(),
                        media_client
                    );
                }
            }

            MediaClientCommand::RemoveSender(node_id) => {
                // Get neighbors
                if let Some(neighbors) = self.neighbor.get_mut(media_client) {
                    // Remove neighbor
                    neighbors.retain(|x| *x != node_id);
                    // Get client's sender channel
                    if let Some((client, _)) = self.mclients.get(media_client) {
                        // Send command and handle Result
                        match client.send(MediaClientCommand::RemoveSender(node_id)) {
                            Ok(()) => {
                                // Send command to GUI
                                self.gui_send.send(GUIEvents::RemoveSender(media_client, node_id));

                                // Launch global flooding
                                self.global_flooding();

                                info!(
                                        "[ {} ]: sent a MediaClientCommand::RemoveSender({}) to [ Client {} ]",
                                        "Simulation Controller".green(),
                                        node_id,
                                        media_client
                                    );
                            }
                            Err(e) => {
                                error!(
                                        "[ {} ]: failed to send a MediaClientCommand::RemoveSender({}) to the [ Client {} ]: {}",
                                        "Simulation Controller".red(),
                                        node_id,
                                        media_client,
                                        e
                                    );
                            }
                        }
                    } else {
                        error!(
                                "[ {} ]: failed to find a Sender<MediaClientCommand> channel for the [ Client {} ]",
                                "Simulation Controller".red(),
                                media_client
                            );
                    }
                } else {
                    error!(
                        "[ {} ]: the [ Drone {} ] does not have any neighbor",
                        "Simulation Controller".red(),
                        node_id
                    );
                }
            }

            MediaClientCommand::AddSender(node_id, sender) => {
                // Check that other node is not a client
                if self.mclients.contains_key(&node_id) {
                    error!(
                        "[ {} ]: The selected NodeId: {} correspond to a Client not a Drone",
                        "Simulation Controller".red(),
                        media_client
                    );
                } else if let Some(neighbors) = self.neighbor.get_mut(media_client) {
                    // Add node to neighbor
                    neighbors.push(node_id);
                    // Get client's sender channel
                    if let Some((client, _)) = self.mclients.get(media_client) {
                        // Send command and handle Result
                        match client.send(MediaClientCommand::AddSender(node_id, sender.clone())) {
                            Ok(()) => {
                                // Send command to GUI
                                self.gui_send.send(GUIEvents::AddSender(media_client, node_id));

                                // Launch global flooding
                                self.global_flooding();

                                info!(
                                        "[ {} ]: sent a MediaClientCommand::AddSender({}, {:?}) to [ Client {} ]",
                                        "Simulation Controller".green(),
                                        node_id,
                                        sender,
                                        media_client
                                    );
                            }
                            Err(e) => {
                                error!(
                                        "[ {} ]: failed to send a MediaClientCommand::AddSender({}, {:?}) to the [ Client {} ]: {}",
                                        "Simulation Controller".red(),
                                        node_id,
                                        sender,
                                        media_client,
                                        e
                                    );
                            }
                        }
                    } else {
                        error!(
                                "[ {} ]: failed to find a Sender<MediaClientCommand> channel for the [ Client {} ]",
                                "Simulation Controller".red(),
                                media_client
                            );
                    }
                } else {
                    error!(
                        "[ {} ]: the [ Drone {} ] does not have any neighbor",
                        "Simulation Controller".red(),
                        node_id
                    );
                }
            }

            MediaClientCommand::AskFilesList(server) => {
                // Get client's sender channel
                if let Some((client, _)) = self.mclients.get(media_client) {
                    // Send command and handle Result
                    match client.send(MediaClientCommand::AskFilesList(server)) {
                        Ok(()) => {
                            info!(
                                "[ {} ]: sent a MediaClientCommand::AskFilesList({}) to [ Client {} ]",
                                "Simulation Controller".green(),
                                server,
                                media_client
                            );
                        }
                        Err(e) => {
                            error!(
                                "[ {} ]: failed to send a MediaClientCommand::AskFilesList({}) to the [ Client {} ]: {}",
                                "Simulation Controller".red(),
                                server,
                                media_client,
                                e
                            );
                        }
                    }
                } else {
                    error!(
                        "[ {} ]: failed to find a Sender<MediaClientCommand> channel for the [ Client {} ]",
                        "Simulation Controller".red(),
                        media_client
                    );
                }
            }

            MediaClientCommand::AskForFile(server, title) => {
                // Get client's sender channel
                if let Some((client, _)) = self.mclients.get(media_client) {
                    // Send command and handle Result
                    match client.send(MediaClientCommand::AskForFile(server, title.clone())) {
                        Ok(()) => {
                            info!(
                                "[ {} ]: sent a MediaClientCommand::AskForFile({}, {}) to [ Client {} ]",
                                "Simulation Controller".green(),
                                server,
                                title,
                                media_client
                            );
                        }
                        Err(e) => {
                            error!(
                                "[ {} ]: failed to send a MediaClientCommand::AskForFile({}, {}) to the [ Client {} ]: {}",
                                "Simulation Controller".red(),
                                server,
                                title,
                                media_client,
                                e
                            );
                        }
                    }
                } else {
                    error!(
                        "[ {} ]: failed to find a Sender<MediaClientCommand> channel for the [ Client {} ]",
                        "Simulation Controller".red(),
                        media_client
                    );
                }
            }

            MediaClientCommand::GetServerList => {
                // Get client's sender channel
                if let Some((client, _)) = self.mclients.get(media_client) {
                    // Send command and handle Result
                    match client.send(MediaClientCommand::GetServerList) {
                        Ok(()) => {
                            info!(
                                "[ {} ]: sent a MediaClientCommand::GetServerList to [ Client {} ]",
                                "Simulation Controller".green(),
                                media_client
                            );
                        }
                        Err(e) => {
                            error!(
                                "[ {} ]: failed to send a MediaClientCommand::GetServerList to the [ Client {} ]: {}",
                                "Simulation Controller".red(),
                                media_client,
                                e
                            );
                        }
                    }
                } else {
                    error!(
                        "[ {} ]: failed to find a Sender<MediaClientCommand> channel for the [ Client {} ]",
                        "Simulation Controller".red(),
                        media_client
                    );
                }
            }

            MediaClientCommand::AskServerType(server) => {
                // Get client's sender channel
                if let Some((client, _)) = self.mclients.get(media_client) {
                    // Send command and handle Result
                    match client.send(MediaClientCommand::AskServerType(server)) {
                        Ok(()) => {
                            info!(
                                "[ {} ]: sent a MediaClientCommand::AskServerType({}) to [ Client {} ]",
                                "Simulation Controller".green(),
                                server,
                                media_client
                            );
                        }
                        Err(e) => {
                            error!(
                                "[ {} ]: failed to send a MediaClientCommand::AskServerType({}) to the [ Client {} ]: {}",
                                "Simulation Controller".red(),
                                server,
                                media_client,
                                e
                            );
                        }
                    }
                } else {
                    error!(
                        "[ {} ]: failed to find a Sender<MediaClientCommand> channel for the [ Client {} ]",
                        "Simulation Controller".red(),
                        media_client
                    );
                }
            }
        }
    }
}
