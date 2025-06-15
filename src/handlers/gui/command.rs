use colored::Colorize;
use crossbeam_channel::TryRecvError;
use log::error;
use messages::{client_commands::{ChatClientCommand, MediaClientCommand}, gui_commands::GUICommands, server_commands::{CommunicationServerCommand, ContentServerCommand}};
use wg_2024::controller::DroneCommand;

use crate::SimulationController;

impl SimulationController {

    pub fn handle_gui_command(&mut self) {
        match self.gui_recv.try_recv() {
            Ok(command) => self.process_gui_command(command),
            Err(TryRecvError::Empty) => (),
            Err(TryRecvError::Disconnected ) => {
                error!(
                    "[ {} ]: DroneEvent receiver channel disconnected",
                    "Simulation Controller".red()
                );
            },
        }
    }

    // Handle GUI Commands
    #[allow(clippy::too_many_lines)]
    fn process_gui_command(&mut self, command: GUICommands) {
        match command {
            GUICommands::Spawn(id, connected_node_ids, pdr) => {
                match self.spawn(id, connected_node_ids, pdr) {
                    Ok(()) => (),
                    Err(e) => {
                        error!("{e}");
                    }
                }
            }
            GUICommands::Crash(drone) => match self.crash(drone) {
                Ok(()) => self.handle_drone_command(&drone, DroneCommand::Crash),
                Err(e) => error!("{e}"),
            },
            GUICommands::RemoveSender(node_id, to_remove) => {
                match self.remove_sender(node_id, to_remove) {
                    Ok(()) => {
                        if self.drones.contains_key(&node_id) {
                            self.handle_drone_command(
                                &node_id,
                                DroneCommand::RemoveSender(to_remove),
                            );
                        } else if self.cclients.contains_key(&node_id) {
                            self.handle_chat_client_command(
                                &node_id,
                                ChatClientCommand::RemoveSender(to_remove),
                            );
                        } else if self.mclients.contains_key(&node_id) {
                            self.handle_media_client_command(
                                &node_id,
                                MediaClientCommand::RemoveSender(to_remove),
                            );
                        } else if self.comm_servers.contains_key(&node_id) {
                            self.handle_communication_server_command(
                                &node_id,
                                CommunicationServerCommand::RemoveSender(to_remove),
                            );
                        }
                    }
                    Err(e) => error!("{e}"),
                }
            }
            GUICommands::AddSender(node_id, to_add) => match self.add_sender(node_id, to_add) {
                Ok(()) => {
                    let sender;
                    if self.drones.contains_key(&to_add) {
                        (_, sender) = self.drones.get(&to_add).unwrap().clone();
                    } else if self.cclients.contains_key(&to_add) {
                        (_, sender) = self.cclients.get(&to_add).unwrap().clone();
                    } else if self.mclients.contains_key(&to_add) {
                        (_, sender) = self.mclients.get(&to_add).unwrap().clone();
                    } else if self.comm_servers.contains_key(&to_add) {
                        (_, sender) = self.comm_servers.get(&to_add).unwrap().clone();
                    } else if self.text_servers.contains_key(&to_add) {
                        (_, sender) = self.text_servers.get(&to_add).unwrap().clone();
                    } else {
                        (_, sender) = self.media_servers.get(&to_add).unwrap().clone();
                    }

                    if self.drones.contains_key(&node_id) {
                        self.handle_drone_command(
                            &node_id,
                            DroneCommand::AddSender(to_add, sender),
                        );
                    } else if self.cclients.contains_key(&node_id) {
                        self.handle_chat_client_command(
                            &node_id,
                            ChatClientCommand::AddSender(to_add, sender),
                        );
                    } else if self.mclients.contains_key(&node_id) {
                        self.handle_media_client_command(
                            &node_id,
                            MediaClientCommand::AddSender(to_add, sender),
                        );
                    } else if self.comm_servers.contains_key(&node_id) {
                        self.handle_communication_server_command(
                            &node_id,
                            CommunicationServerCommand::AddSender(to_add, sender),
                        );
                    } else if self.text_servers.contains_key(&to_add) {
                        self.handle_text_server_command(
                            &node_id,
                            ContentServerCommand::AddSender(to_add, sender),
                        );
                    } else if self.media_servers.contains_key(&to_add) {
                        self.handle_media_server_command(
                            &node_id,
                            ContentServerCommand::AddSender(to_add, sender),
                        );
                    }
                }
                Err(e) => error!("{e}"),
            },
            GUICommands::SetPDR(drone, pdr) => {
                if (0.0..=1.0).contains(&pdr) {
                    self.handle_drone_command(&drone, DroneCommand::SetPacketDropRate(pdr));
                } else {
                    error!("[ ERROR ]: The PDR number is out of range. Please enter a number between 0.00 and 1.00");
                }
            }

            GUICommands::SendMessageTo(src, dest, msg) => {
                self.handle_chat_client_command(&src, ChatClientCommand::SendMessageTo(dest, msg));
            }
            GUICommands::RegisterTo(client, server) => {
                self.handle_chat_client_command(&client, ChatClientCommand::RegisterTo(server));
            }
            GUICommands::GetClientList(client) => {
                self.handle_chat_client_command(&client, ChatClientCommand::GetClientList);
            }
            GUICommands::LogOut(client, _) => {
                self.handle_chat_client_command(&client, ChatClientCommand::LogOut);
            }
            GUICommands::AskForFileList(client, server) => {
                self.handle_media_client_command(&client, MediaClientCommand::AskFilesList(server));
            }
            GUICommands::GetFile(client, server, title) => {
                self.handle_media_client_command(&client, MediaClientCommand::AskForFile(server, title));
            }
        }
    }
}
