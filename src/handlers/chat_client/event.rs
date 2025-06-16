use crossbeam_channel::TryRecvError;
use wg_2024::packet::PacketType;

use colored::Colorize;
use log::{error, info};

use messages::{client_commands::ChatClientEvent, gui_commands::GUIEvents};

use crate::SimulationController;

impl SimulationController {
    #[allow(clippy::too_many_lines)]
    pub fn handle_chat_client_event(&mut self, event: ChatClientEvent) {
        match event {
            ChatClientEvent::CommunicationServerList(items) => {
                info!(
                    "[ {} ]: The Client retrieved the CommunicationServers list: {:?}",
                    "Simulation Controller".green(),
                    items
                );
            }

            ChatClientEvent::MessageReceived(src, dest, msg) => {
                info!(
                    "[ {} ]: [ Client: {} ]: received the message {:?} from [ Server {} ]",
                    "Simulation Controller".green(),
                    dest,
                    msg,
                    src
                );

                // Send to GUI
                match self
                    .gui_send
                    .send(GUIEvents::MessageReceived(src, dest, msg.clone()))
                {
                    Ok(()) => {
                        info!(
                            "[ {} ]: successfully sent a GUIEvents::MessageReceived({}, {}, {:?}) from the Simulation Controller to the GUI",
                            "Simulation Controller".green(),
                            src,
                            dest,
                            msg
                        );
                    }
                    Err(e) => {
                        error!(
                            "[ {} ]: failed to sent a GUIEvents::MessageReceived({}, {}, {:?}) from the Simulation Controller to the GUI: {}",
                            "Simulation Controller".green(),
                            src,
                            dest,
                            msg,
                            e
                        );
                    }
                }
            }

            ChatClientEvent::SuccessfulRegistration(server) => {
                info!(
                    "[ {} ]: The Client successfully register to the [ Server {}]",
                    "Simulation Controller".green(),
                    server
                );
            }

            ChatClientEvent::ClientList(client, client_list) => {
                info!(
                    "[ {} ]: The Client retrieved the Client list: {:?}",
                    "Simulation Controller".green(),
                    client_list
                );

                // Send to GUI
                match self
                    .gui_send
                    .send(GUIEvents::ClientList(client, client_list.clone()))
                {
                    Ok(()) => {
                        info!(
                            "[ {} ]: successfully sent a GUIEvents::ClientList({}, {:?}) from the Simulation Controller to the GUI",
                            "Simulation Controller".green(),
                            client,
                            client_list
                        );
                    }
                    Err(e) => {
                        error!(
                            "[ {} ]: failed to sent a GUIEvents::ClientList({}, {:?}) from the Simulation Controller to the GUI: {}",
                            "Simulation Controller".green(),
                            client,
                            client_list,
                            e
                        );
                    }
                }
            }

            ChatClientEvent::SuccessfulLogOut => {
                info!(
                    "[ {} ]: The Client successfully logged out from the server",
                    "Simulation Controller".green(),
                );
            }

            ChatClientEvent::UnreachableClient(client) => {
                error!(
                    "[ {} ]: received an error message: [ Client {} ] is not register on the selected server",
                    "Simulation Controller".red(),
                    client,
                );
            }

            ChatClientEvent::ErrorNotRunning => {
                error!(
                    "[ {} ]: received an error message: The client tried to register without before starting",
                    "Simulation Controller".red(),
                );
            }

            ChatClientEvent::ErrorNotRegistered => {
                error!(
                    "[ {} ]: received an error message: The Client is not register to a server",
                    "Simulation Controller".red(),
                );
            }

            ChatClientEvent::ControllerShortcut(packet) => {
                // Get destination of the packet
                if let Some(dest) = packet.routing_header.hops.last() {
                    // Get destination's packet  channel
                    let packet_channel;
                    if self.drones.contains_key(dest) {
                        (_, packet_channel) = self.drones.get(dest).unwrap().clone();
                    } else if self.cclients.contains_key(dest) {
                        (_, packet_channel) = self.cclients.get(dest).unwrap().clone();
                    } else if self.mclients.contains_key(dest) {
                        (_, packet_channel) = self.mclients.get(dest).unwrap().clone();
                    } else if self.comm_servers.contains_key(dest) {
                        (_, packet_channel) = self.comm_servers.get(dest).unwrap().clone();
                    } else if self.text_servers.contains_key(dest) {
                        (_, packet_channel) = self.text_servers.get(dest).unwrap().clone();
                    } else if self.media_servers.contains_key(dest) {
                        (_, packet_channel) = self.media_servers.get(dest).unwrap().clone();
                    } else {
                        error!(
                            "[ {} ]: failed to find a Sender<Packet> channel for the [ ChatClient {} ]",
                            "Simulation Controller".red(),
                            dest
                        );
                        return;
                    }

                    // Send Packet to destination
                    match packet.pack_type {
                        PacketType::MsgFragment(_) => {
                            panic!("Impossible how the hell did u do this");
                        }
                        _ => {
                            packet_channel.send(packet.clone()).unwrap();
                        }
                    }
                } else {
                    error!(
                        "[ {} ]: failed to find a ChatClient to send the ChatClientCommand::ControllerShortcut",
                        "Simulation Controller".red()
                    );
                }
            }
        }
    }
}
