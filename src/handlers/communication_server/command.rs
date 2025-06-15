use wg_2024::network::NodeId;

use colored::Colorize;
use log::{error, info};

use messages::{gui_commands::GUIEvents, server_commands::CommunicationServerCommand};

use crate::SimulationController;

impl SimulationController {
    #[allow(clippy::too_many_lines)]
    pub fn handle_communication_server_command(
        &mut self,
        comm_server: &NodeId,
        command: CommunicationServerCommand,
    ) {
        match command {
            CommunicationServerCommand::InitFlooding => {
                // Get server's sender channel
                if let Some((server, _)) = self.comm_servers.get(comm_server) {
                    // Send command and handle Result
                    match server.send(CommunicationServerCommand::InitFlooding) {
                        Ok(()) => {
                            info!(
                                "[ {} ]: sent a CommunicationServerCommand::InitFlooding to [ CommunicationServer {} ]",
                                "Simulation Controller".green(),
                                comm_server
                            );
                        }
                        Err(e) => {
                            error!(
                                "[ {} ]: failed to send a CommunicationServerCommand::InitFlooding to the [ CommunicationServer {} ]: {}",
                                "Simulation Controller".red(),
                                comm_server,
                                e
                            );
                        }
                    }
                } else {
                    error!(
                        "[ {} ]: failed to find a Sender<CommunicationServerCommand> channel for the [ CommunicationServer {} ]",
                        "Simulation Controller".red(),
                        comm_server
                    );
                }
            }
            CommunicationServerCommand::AddSender(node_id, sender) => {
                // Get server's sender channel
                if let Some((server, _)) = self.comm_servers.get(comm_server) {
                    // Get neighbors
                    if let Some(vec) = self.neighbor.get_mut(comm_server) {
                        // Add node to neighbor
                        vec.push(node_id);
                        // Send command and handle Result
                        match server.send(CommunicationServerCommand::AddSender(node_id, sender)) {
                            Ok(()) => {
                                // Send command to GUI
                                let _ = self.gui_send.send(GUIEvents::AddSender(*comm_server, node_id));

                                // Launch globla flooding
                                self.global_flooding();

                                info!(
                                    "[ {} ]: sent a CommunicationServerCommand::AddSender({}, sender_channel) to [ CommunicationServer {} ]",
                                    "Simulation Controller".green(),
                                    node_id,
                                    comm_server
                                );
                            }
                            Err(e) => {
                                error!(
                                    "[ {} ]: failed to send a CommunicationServerCommand::AddSender({}, sender_channel) to the [ CommunicationServer {} ]: {}",
                                    "Simulation Controller".red(),
                                    node_id,
                                    comm_server,
                                    e
                                );
                            }
                        }
                    } else {
                        error!(
                            "[ {} ]: the [ CommunicationServer {} ] does not have any neighbor",
                            "Simulation Controller".red(),
                            comm_server
                        );
                    }
                } else {
                    error!(
                        "[ {} ]: failed to find a Sender<CommunicationServerCommand> channel for the [ CommunicationServer {} ]",
                        "Simulation Controller".red(),
                        comm_server
                    );
                }
            }
            CommunicationServerCommand::RemoveSender(node_id) => {
                // Get server's sender channel
                if let Some((server, _)) = self.comm_servers.get(comm_server) {
                    // Get neighbors
                    if let Some(vec) = self.neighbor.get_mut(comm_server) {
                        // Remove node from neighbors
                        vec.retain(|x| *x != node_id);
                        // Send command and handle Result
                        match server.send(CommunicationServerCommand::RemoveSender(node_id)) {
                            Ok(()) => {
                                // Send command to GUI
                                let _ = self.gui_send.send(GUIEvents::RemoveSender(*comm_server, node_id));

                                // Launch global flooding
                                self.global_flooding();

                                info!(
                                        "[ {} ]: sent a CommunicationServerCommand::RemoveSender({}) to [ CommunicationServer {} ]",
                                        "Simulation Controller".green(),
                                        node_id,
                                        comm_server
                                    );
                            }
                            Err(e) => {
                                error!(
                                        "[ {} ]: failed to send a CommunicationServerCommand::RemoveSender({}) to the [ CommunicationServer {} ]: {}",
                                        "Simulation Controller".red(),
                                        node_id,
                                        comm_server,
                                        e
                                    );
                            }
                        }
                    } else {
                        error!(
                            "[ {} ]: the [ CommunicationServer {} ] does not have any neighbor",
                            "Simulation Controller".red(),
                            comm_server
                        );
                    }
                } else {
                    error!(
                        "[ {} ]: failed to find a Sender<CommunicationServerCommand> channel for the [ CommunicationServer {} ]",
                        "Simulation Controller".red(),
                        comm_server
                    );
                }
            }
            CommunicationServerCommand::LogNetwork => {
                // Get server's sender channel
                if let Some((server, _)) = self.comm_servers.get(comm_server) {
                    // Send command and handle Result
                    match server.send(CommunicationServerCommand::LogNetwork) {
                        Ok(()) => {
                            info!(
                                "[ {} ]: sent a CommunicationServerCommand::LogNetwork to [ CommunicationServer {} ]",
                                "Simulation Controller".green(),
                                comm_server
                            );
                        }
                        Err(e) => {
                            error!(
                                "[ {} ]: failed to send a CommunicationServerCommand::LogNetwork to the [ CommunicationServer {} ]: {}",
                                "Simulation Controller".red(),
                                comm_server,
                                e
                            );
                        }
                    }
                } else {
                    error!(
                        "[ {} ]: failed to find a Sender<CommunicationServerCommand> channel for the [ CommunicationServer {} ]",
                        "Simulation Controller".red(),
                        comm_server
                    );
                }
            }
        }
    }
}
