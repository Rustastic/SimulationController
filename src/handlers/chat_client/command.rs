use wg_2024::network::NodeId;

use colored::Colorize;
use log::{error, info};

use messages::client_commands::ChatClientCommand;

use crate::SimulationController;

impl SimulationController {
    #[allow(clippy::too_many_lines)]
    pub fn handle_chat_client_command(&mut self, chat_client: &NodeId, command: ChatClientCommand) {
        match command {
            ChatClientCommand::InitFlooding => {
                // Get correct sender channel
                if let Some((client, _)) = self.cclients.get(chat_client) {
                    // Send command and handle Result
                    match client.send(ChatClientCommand::InitFlooding) {
                        Ok(()) => {
                            info!(
                                "[ {} ]: sent a ChatClientCommand::InitFlo0ding to [ Client {} ]",
                                "Simulation Controller".green(),
                                chat_client
                            );
                        }
                        Err(e) => {
                            error!(
                                "[ {} ]: failed to send a ChatClientCommand::InitFlooding to the [ ChatClient {} ]: {}",
                                "Simulation Controller".red(),
                                chat_client,
                                e
                            );
                        }
                    }
                } else {
                    error!(
                        "[ {} ]: failed to find a Sender<ChatClientCommand> channel for the [ ChatClient {} ]",
                        "Simulation Controller".red(),
                        chat_client
                    );
                }
            }

            ChatClientCommand::StartChatClient => {
                // Get correct sender channel
                if let Some((client, _)) = self.cclients.get(chat_client) {
                    // Send command and handle Result
                    match client.send(ChatClientCommand::StartChatClient) {
                        Ok(()) => {
                            info!(
                                "[ {} ]: sent a ChatClientCommand::StartChatClient to [ ChatClient {} ]",
                                "Simulation Controller".green(),
                                chat_client
                            );
                        }
                        Err(e) => {
                            error!(
                                "[ {} ]: failed to send a ChatClientCommand::StartChatClient to the [ ChatClient {} ]: {}",
                                "Simulation Controller".red(),
                                chat_client,
                                e
                            );
                        }
                    }
                } else {
                    error!(
                        "[ {} ]: failed to find a Sender<ChatClientCommand> channel for the [ ChatClient {} ]",
                        "Simulation Controller".red(),
                        chat_client
                    );
                }
            }

            ChatClientCommand::RemoveSender(node_id) => {
                // Get neighbors of the chat client
                if let Some(neighbors) = self.neighbor.get_mut(chat_client) {
                    // Remove node from neighbors
                    neighbors.retain(|x| *x != node_id);
                    // Get correct sender channel
                    if let Some((client, _)) = self.cclients.get(chat_client) {
                        // Send command and handle Result
                        match client.send(ChatClientCommand::RemoveSender(node_id)) {
                            Ok(()) => {
                                // Send command to GUI

                                // Launch global flooding
                                self.global_flooding();

                                info!(
                                        "[ {} ]: sent a ChatClientCommand::RemoveSender({}) to [ ChatClient {} ]",
                                        "Simulation Controller".green(),
                                        node_id,
                                        chat_client
                                    );
                            }
                            Err(e) => {
                                error!(
                                        "[ {} ]: failed to send a ChatClientCommand::RemoveSender({}) to the [ ChatClient {} ]: {}",
                                        "Simulation Controller".red(),
                                        node_id,
                                        chat_client,
                                        e
                                    );
                            }
                        }
                    } else {
                        error!(
                                "[ {} ]: failed to find a Sender<ChatClientCommand> channel for the [ ChatClient {} ]",
                                "Simulation Controller".red(),
                                chat_client
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

            ChatClientCommand::AddSender(node_id, sender) => {
                // Check node we want to add is not another client
                if self.cclients.contains_key(&node_id) {
                    error!(
                        "[ {} ]: The selected NodeId: {} correspond to a Client",
                        "Simulation Controller".red(),
                        chat_client
                    );
                } else if let Some(neighbors) = self.neighbor.get_mut(chat_client) {
                    // Get neighbors of the chat client
                    // Add node to the neighbors
                    neighbors.push(node_id);
                    // Get correct sender channel
                    if let Some((client, _)) = self.cclients.get(chat_client) {
                        // Send command and handle Result
                        match client.send(ChatClientCommand::AddSender(node_id, sender.clone())) {
                            Ok(()) => {
                                // Send command to GUI

                                // Launch global flooding
                                self.global_flooding();

                                info!(
                                        "[ {} ]: sent a ChatClientCommand::AddSender({}, {:?}) to [ ChatClient {} ]",
                                        "Simulation Controller".green(),
                                        node_id,
                                        sender,
                                        chat_client
                                    );
                            }
                            Err(e) => {
                                error!(
                                        "[ {} ]: failed to send a ChatClientCommand::AddSender({}, {:?}) to the [ ChatClient {} ]: {}",
                                        "Simulation Controller".red(),
                                        node_id,
                                        sender,
                                        chat_client,
                                        e
                                    );
                            }
                        }
                    } else {
                        error!(
                                "[ {} ]: failed to find a Sender<ChatClientCommand> channel for the [ ChatClient {} ]",
                                "Simulation Controller".red(),
                                chat_client
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

            ChatClientCommand::SendMessageTo(dest, msg) => {
                // Get correct sender channel
                if let Some((client, _)) = self.cclients.get(chat_client) {
                    // Send command and handle Result
                    match client.send(ChatClientCommand::SendMessageTo(dest, msg.clone())) {
                        Ok(()) => {
                            info!(
                                "[ {} ]: sent a ChatClientCommand::SendMessageTo({}, {}) to [ ChatClient {} ]",
                                "Simulation Controller".green(),
                                dest,
                                msg,
                                chat_client
                            );
                        }
                        Err(e) => {
                            error!(
                                "[ {} ]: failed to send a ChatClientCommand::SendMessageTo({}, {}) to the [ ChatClient {} ]: {}",
                                "Simulation Controller".red(),
                                dest,
                                msg,
                                chat_client,
                                e
                            );
                        }
                    }
                } else {
                    error!(
                        "[ {} ]: failed to find a Sender<ChatClientCommand> channel for the [ ChatClient {} ]",
                        "Simulation Controller".red(),
                        chat_client
                    );
                }
            }

            ChatClientCommand::RegisterTo(server_id) => {
                // Check if the given node is a Communication Server
                if self.comm_servers.contains_key(&server_id) {
                    // Get correct sender channel
                    if let Some((client, _)) = self.cclients.get(chat_client) {
                        // Send command and handle Result
                        match client.send(ChatClientCommand::RegisterTo(server_id)) {
                            Ok(()) => {
                                info!(
                                    "[ {} ]: sent a ChatClientCommand::RegisterTo({}) to [ ChatClient {} ]",
                                    "Simulation Controller".green(),
                                    server_id,
                                    chat_client
                                );
                            }
                            Err(e) => {
                                error!(
                                    "[ {} ]: failed to send a ChatClientCommand::RegisterTo({}) to the [ ChatClient {} ]: {}",
                                    "Simulation Controller".red(),
                                    server_id,
                                    chat_client,
                                    e
                                );
                            }
                        }
                    } else {
                        error!(
                            "[ {} ]: failed to find a Sender<ChatClientCommand> channel for the [ ChatClient {} ]",
                            "Simulation Controller".red(),
                            chat_client
                        );
                    }
                } else {
                    error!(
                        "[ {} ]: The [ Node {} ] is not a CommunicationServer",
                        "Simulation Controller".red(),
                        server_id,
                    );
                }
            }

            ChatClientCommand::GetClientList => {
                // Get correct sender channel
                if let Some((client, _)) = self.cclients.get(chat_client) {
                    // Send command and handle Result
                    match client.send(ChatClientCommand::GetClientList) {
                        Ok(()) => {
                            info!(
                                "[ {} ]: sent a ChatClientCommand::GetClientList to [ ChatClient {} ]",
                                "Simulation Controller".green(),
                                chat_client
                            );
                        }
                        Err(e) => {
                            error!(
                                "[ {} ]: failed to send a ChatClientCommand::GetClientList to the [ ChatClient {} ]: {}",
                                "Simulation Controller".red(),
                                chat_client,
                                e
                            );
                        }
                    }
                } else {
                    error!(
                        "[ {} ]: failed to find a Sender<ChatClientCommand> channel for the [ ChatClient {} ]",
                        "Simulation Controller".red(),
                        chat_client
                    );
                }
            }

            ChatClientCommand::LogOut => {
                // Get correct sender channel
                if let Some((client, _)) = self.cclients.get(chat_client) {
                    // Send command and handle Result
                    match client.send(ChatClientCommand::LogOut) {
                        Ok(()) => {
                            info!(
                                "[ {} ]: sent a ChatClientCommand::LogOut to [ ChatClient {} ]",
                                "Simulation Controller".green(),
                                chat_client
                            );
                        }
                        Err(e) => {
                            error!(
                                "[ {} ]: failed to send a ChatClientCommand::LogOut to the [ ChatClient {} ]: {}",
                                "Simulation Controller".red(),
                                chat_client,
                                e
                            );
                        }
                    }
                } else {
                    error!(
                        "[ {} ]: failed to find a Sender<ChatClientCommand> channel for the [ ChatClient {} ]",
                        "Simulation Controller".red(),
                        chat_client
                    );
                }
            }

            ChatClientCommand::LogNetwork => {
                // Get correct sender channel
                if let Some((client, _)) = self.cclients.get(chat_client) {
                    // Send command and handle Result
                    match client.send(ChatClientCommand::LogNetwork) {
                        Ok(()) => {
                            info!(
                                "[ {} ]: sent a ChatClientCommand::LogNetwork to [ ChatClient {} ]",
                                "Simulation Controller".green(),
                                chat_client
                            );
                        }
                        Err(e) => {
                            error!(
                                "[ {} ]: failed to send a ChatClientCommand::LogNetwork to the [ ChatClient {} ]: {}",
                                "Simulation Controller".red(),
                                chat_client,
                                e
                            );
                        }
                    }
                } else {
                    error!(
                        "[ {} ]: failed to find a Sender<ChatClientCommand> channel for the [ ChatClient {} ]",
                        "Simulation Controller".red(),
                        chat_client
                    );
                }
            }
        }
    }
}
