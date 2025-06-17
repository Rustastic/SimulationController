use colored::Colorize;
use log::error;
use messages::{
    client_commands::{ChatClientCommand, MediaClientCommand},
    gui_commands::{GUICommands, GUIEvents},
    server_commands::{CommunicationServerCommand, ContentServerCommand},
};
use wg_2024::controller::DroneCommand;

use crate::SimulationController;

impl SimulationController {
    #[allow(clippy::too_many_lines)]
    pub fn handle_gui_command(&mut self, command: GUICommands) {
        match command {
            GUICommands::Spawn(id, connected_node_ids, pdr) => {
                // check if spawn is possible
                if self.drones.contains_key(&id) {
                    error!(
                        "[ {} ] Can not spawn the drone: drone with the NodeId: {} already exist",
                        "Simulation Controller".red(),
                        id,
                    );
                } else if !(0.0..=1.0).contains(&pdr) {
                    error!(
                        "[ {} ] Can not spawn the drone: PDR value mus be between 0.0 and 1.0",
                        "Simulation Controller".red(),
                    );
                } else {
                    // check if can add neighbors
                    for neighbor in &connected_node_ids {
                        match self.check_add(*neighbor) {
                            Ok(()) => (),
                            Err(e) => {
                                error!("[ {} ] {e}", "Simulation Controller".red());
                                return;
                            }
                        }
                    }
                    // Create drone
                    match self.spawn(id, connected_node_ids.clone(), pdr) {
                        Ok(()) => {
                            // launch global flooding
                            self.global_flooding();

                            // Send command to GUI
                            let _ =
                                self.gui_send
                                    .send(GUIEvents::Spawn(id, connected_node_ids, pdr));
                        }
                        Err(e) => {
                            error!("[ {} ] {e}", "Simulation Controller".red());
                        }
                    }
                }
            }
            GUICommands::Crash(drone) => {
                // check drone existence
                match self.check_drone_existence(drone) {
                    Ok(()) => {
                        // Check if the edges can be removed
                        if let Some(neighbors) = self.neighbor.get(&drone) {
                            for neighbor in neighbors {
                                match self.check_remove(*neighbor) {
                                    Ok(()) => (),
                                    Err(e) => {
                                        error!("[ {} ] {e}", "Simulation Controller".red());
                                        return;
                                    }
                                }
                            }
                            // Send commands to neighbors
                            match self.crash(drone) {
                                Ok(()) => {
                                    // send command to drone
                                    self.handle_drone_command(&drone, DroneCommand::Crash);

                                    // launch global flooding
                                    self.global_flooding();

                                    // Send command to GUI
                                    let _ = self.gui_send.send(GUIEvents::Crash(drone));
                                }
                                Err(e) => {
                                    error!("[ {} ] {e}", "Simulation Controller".red());
                                }
                            }
                        } else {
                            error!(
                                "[ {} ]: the [ Drone {} ] does not have any neighbor",
                                "Simulation Controller".red(),
                                drone
                            );
                        }
                    }
                    Err(e) => {
                        error!("[ {} ] {e}", "Simulation Controller".red());
                    }
                }
            }
            GUICommands::RemoveSender(node_id, to_remove) => {
                match self.check_remove(node_id) {
                    Ok(()) => (),
                    Err(e) => {
                        error!("[ {} ] {e}", "Simulation Controller".red());
                        return;
                    }
                }

                match self.check_remove(to_remove) {
                    Ok(()) => (),
                    Err(e) => {
                        error!("[ {} ] {e}", "Simulation Controller".red());
                        return;
                    }
                }

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
                    Err(e) => {
                        error!("[ {} ] {e}", "Simulation Controller".red());
                    }
                }
            }
            GUICommands::AddSender(node_id, to_add) => {
                match self.check_add(node_id) {
                    Ok(()) => (),
                    Err(e) => {
                        error!("[ {} ] {e}", "Simulation Controller".red());
                        return;
                    }
                }

                match self.check_add(to_add) {
                    Ok(()) => (),
                    Err(e) => {
                        error!("[ {} ] {e}", "Simulation Controller".red());
                        return;
                    }
                }

                match self.add_sender(node_id, to_add) {
                    Ok(()) => {
                        let sender;
                        if let Some((_, s)) = self.drones.get(&to_add) {
                            sender = s.clone();
                        } else if let Some((_, s)) = self.cclients.get(&to_add) {
                            sender = s.clone();
                        } else if let Some((_, s)) = self.mclients.get(&to_add) {
                            sender = s.clone();
                        } else if let Some((_, s)) = self.comm_servers.get(&to_add) {
                            sender = s.clone();
                        } else if let Some((_, s)) = self.text_servers.get(&to_add) {
                            sender = s.clone();
                        } else if let Some((_, s)) = self.media_servers.get(&to_add) {
                            sender = s.clone();
                        } else {
                            error!(
                                "[ {} ]: failed to find a Sender<Packet> channel for the [ Node {} ]",
                                "Simulation Controller".red(),
                                to_add
                            );
                            return;
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
                }
            }
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
                self.handle_media_client_command(
                    &client,
                    MediaClientCommand::AskForFile(server, title),
                );
            }
        }
    }
}
