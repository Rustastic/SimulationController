use colored::Colorize;
use log::{error, info};
use wg_2024::{network::NodeId, packet::PacketType};

use messages::{
    client_commands::{ChatClientCommand, ChatClientEvent},
    gui_commands::GUIEvents,
};

use crate::SimulationController;

impl SimulationController {
    // Handle ChatClient Event
    #[allow(clippy::too_many_lines)]
    pub fn handle_cclient_event(&mut self, event: ChatClientEvent) {
        info!("[ {} ] Is a {:?}", "Simulation Controller".yellow(), event);
        match event {
            ChatClientEvent::CommunicationServerList(items) => {
                info!("The Client retrieved the CommunicationServers list: {items:?}");
            }
            ChatClientEvent::MessageReceived(src, dest, msg) => {
                info!("[ Client: {src} ]: received the message {msg:?} from [ Server {src} ]");
                match self.gui_send.send(GUIEvents::MessageReceived(src, dest, msg.clone())) {
                    Ok(()) => info!(
                        "[ {} ]: successfully sent a GUIEvents::MessageReceived({}, {}, {:?}) from the Simulation Controller to the GUI",
                        "Simulation Controller".green(),
                        src,
                        dest,
                        msg
                    ),
                    Err(e) => error!(
                        "[ {} ]: failed to sent a GUIEvents::MessageReceived({}, {}, {:?}) from the Simulation Controller to the GUI: {}",
                        "Simulation Controller".green(),
                        src,
                        dest,
                        msg,
                        e
                    ),
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
                info!("The Client retrieved the Client list: {client_list:?}");
                match self.gui_send.send(GUIEvents::ClientList(client, client_list.clone())) {
                    Ok(()) => info!(
                        "[ {} ]: successfully sent a GUIEvents::ClientList({}, {:?}) from the Simulation Controller to the GUI",
                        "Simulation Controller".green(),
                        client,
                        client_list
                    ),
                    Err(e) => error!(
                        "[ {} ]: failed to sent a GUIEvents::ClientList({}, {:?}) from the Simulation Controller to the GUI: {}",
                        "Simulation Controller".green(),
                        client,
                        client_list,
                        e
                    ),
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
                if let Some(dest) = packet.routing_header.hops.last() {
                    // Get destination node channel
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

    // Handle ChatClient Command
    #[allow(clippy::too_many_lines)]
    pub fn handle_cclient_command(&mut self, chat_client: &NodeId, command: ChatClientCommand) {
        match command {
            ChatClientCommand::InitFlooding => {
                if let Some((client, _)) = self.cclients.get(chat_client) {
                    match client.send(ChatClientCommand::InitFlooding) {
                        Ok(()) => info!(
                            "[ {} ]: sent a ChatClientCommand::InitFlo0ding to [ Client {} ]",
                            "Simulation Controller".green(),
                            chat_client
                        ),
                        Err(e) => error!(
                            "[ {} ]: failed to send a ChatClientCommand::InitFlooding to the [ ChatClient {} ]: {}",
                            "Simulation Controller".red(),
                            chat_client,
                            e
                        ),
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
                if let Some((client, _)) = self.cclients.get(chat_client) {
                    match client.send(ChatClientCommand::StartChatClient) {
                        Ok(()) => info!(
                            "[ {} ]: sent a ChatClientCommand::StartChatClient to [ ChatClient {} ]",
                            "Simulation Controller".green(),
                            chat_client
                        ),
                        Err(e) => error!(
                            "[ {} ]: failed to send a ChatClientCommand::StartChatClient to the [ ChatClient {} ]: {}",
                            "Simulation Controller".red(),
                            chat_client,
                            e
                        ),
                    }
                } else {
                    error!(
                        "[ {} ]: failed to find a Sender<ChatClientCommand> channel for the [ ChatClient {} ]",
                        "Simulation Controller".red(),
                        chat_client
                    );
                }
            }
            ChatClientCommand::RemoveSender(drone) => {
                if let Some(neighbors) = self.neighbor.get_mut(chat_client) {
                    // Max 2 neighbor, Min 1 neighbor
                    if neighbors.len() == 2 {
                        neighbors.retain(|x| *x != drone);
                        if let Some((client, _)) = self.cclients.get(chat_client) {
                            match client.send(ChatClientCommand::RemoveSender(drone)) {
                                Ok(()) => {
                                    self.send_re_init_flooding();
                                    info!(
                                        "[ {} ]: sent a ChatClientCommand::RemoveSender({}) to [ ChatClient {} ]",
                                        "Simulation Controller".green(),
                                        drone,
                                        chat_client
                                    );
                                },
                                Err(e) => error!(
                                    "[ {} ]: failed to send a ChatClientCommand::RemoveSender({}) to the [ ChatClient {} ]: {}",
                                    "Simulation Controller".red(),
                                    drone,
                                    chat_client,
                                    e
                                ),
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
                            "[ {} ]: failed to send a ChatClientCommand::RemoveSender({}) to the [ ChatClient {} ]: {}",
                            "Simulation Controller".red(),
                            drone,
                            chat_client,
                            "Each client must remain connected to at least one drone"
                        );
                    }
                } else {
                    error!(
                        "[ {} ]: the [ Drone {} ] does not have any neighbor",
                        "Simulation Controller".red(),
                        drone
                    );
                }
            }
            ChatClientCommand::AddSender(drone, sender) => {
                // Can't connect to a client
                if self.cclients.contains_key(&drone) {
                    error!(
                        "[ {} ]: The selected NodeId: {} correspond to a Client not a Drone",
                        "Simulation Controller".red(),
                        chat_client
                    );
                } else if let Some(neighbors) = self.neighbor.get_mut(chat_client) {
                    // Max 2 neighbor, Min 1 neighbor
                    if neighbors.len() == 1 {
                        neighbors.push(drone);
                        if let Some((client, _)) = self.cclients.get(chat_client) {
                            match client.send(ChatClientCommand::AddSender(drone, sender.clone())) {
                                Ok(()) => {
                                    self.send_re_init_flooding();
                                    info!(
                                        "[ {} ]: sent a ChatClientCommand::AddSender({}, {:?}) to [ ChatClient {} ]",
                                        "Simulation Controller".green(),
                                        drone,
                                        sender,
                                        chat_client
                                    );
                                },
                                Err(e) => error!(
                                    "[ {} ]: failed to send a ChatClientCommand::AddSender({}, {:?}) to the [ ChatClient {} ]: {}",
                                    "Simulation Controller".red(),
                                    drone,
                                    sender,
                                    chat_client,
                                    e
                                ),
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
                            "[ {} ]: failed to send a ChatClientCommand::AddSender({}, {:?}) to the [ ChatClient {} ]: {}",
                            "Simulation Controller".red(),
                            drone,
                            sender,
                            chat_client,
                            "Each client must be connected to at most two drones"
                        );
                    }
                } else {
                    error!(
                        "[ {} ]: the [ Drone {} ] does not have any neighbor",
                        "Simulation Controller".red(),
                        drone
                    );
                }
            }
            ChatClientCommand::SendMessageTo(dest, msg) => {
                if let Some((client, _)) = self.cclients.get(chat_client) {
                    match client.send(ChatClientCommand::SendMessageTo(dest, msg.clone())) {
                        Ok(()) => info!(
                            "[ {} ]: sent a ChatClientCommand::SendMessageTo({}, {}) to [ ChatClient {} ]",
                            "Simulation Controller".green(),
                            dest,
                            msg,
                            chat_client
                        ),
                        Err(e) => error!(
                            "[ {} ]: failed to send a ChatClientCommand::SendMessageTo({}, {}) to the [ ChatClient {} ]: {}",
                            "Simulation Controller".red(),
                            dest,
                            msg,
                            chat_client,
                            e
                        ),
                    }
                } else {
                    error!(
                        "[ {} ]: failed to find a Sender<ChatClientCommand> channel for the [ ChatClient {} ]",
                        "Simulation Controller".red(),
                        chat_client
                    );
                }
            }
            ChatClientCommand::RegisterTo(server) => {
                if self.comm_servers.contains_key(&server) {
                    if let Some((client, _)) = self.cclients.get(chat_client) {
                        match client.send(ChatClientCommand::RegisterTo(server)) {
                            Ok(()) => info!(
                                "[ {} ]: sent a ChatClientCommand::RegisterTo({}) to [ ChatClient {} ]",
                                "Simulation Controller".green(),
                                server,
                                chat_client
                            ),
                            Err(e) => error!(
                                "[ {} ]: failed to send a ChatClientCommand::RegisterTo({}) to the [ ChatClient {} ]: {}",
                                "Simulation Controller".red(),
                                server,
                                chat_client,
                                e
                            ),
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
                        server,
                    );
                }
            }
            ChatClientCommand::GetClientList => {
                if let Some((client, _)) = self.cclients.get(chat_client) {
                    match client.send(ChatClientCommand::GetClientList) {
                        Ok(()) => info!(
                            "[ {} ]: sent a ChatClientCommand::GetClientList to [ ChatClient {} ]",
                            "Simulation Controller".green(),
                            chat_client
                        ),
                        Err(e) => error!(
                            "[ {} ]: failed to send a ChatClientCommand::GetClientList to the [ ChatClient {} ]: {}",
                            "Simulation Controller".red(),
                            chat_client,
                            e
                        ),
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
                if let Some((client, _)) = self.cclients.get(chat_client) {
                    match client.send(ChatClientCommand::LogOut) {
                        Ok(()) => info!(
                            "[ {} ]: sent a ChatClientCommand::LogOut to [ ChatClient {} ]",
                            "Simulation Controller".green(),
                            chat_client
                        ),
                        Err(e) => error!(
                            "[ {} ]: failed to send a ChatClientCommand::LogOut to the [ ChatClient {} ]: {}",
                            "Simulation Controller".red(),
                            chat_client,
                            e
                        ),
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
                if let Some((client, _)) = self.cclients.get(chat_client) {
                    match client.send(ChatClientCommand::LogNetwork) {
                        Ok(()) => info!(
                            "[ {} ]: sent a ChatClientCommand::LogNetwork to [ ChatClient {} ]",
                            "Simulation Controller".green(),
                            chat_client
                        ),
                        Err(e) => error!(
                            "[ {} ]: failed to send a ChatClientCommand::LogNetwork to the [ ChatClient {} ]: {}",
                            "Simulation Controller".red(),
                            chat_client,
                            e
                        ),
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
