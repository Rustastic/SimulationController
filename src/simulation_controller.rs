use crossbeam_channel::{select, Receiver, Sender};
use log::{error, info};
use std::{collections::HashMap, thread};

use colored::Colorize;

use wg_2024::{
    controller::{DroneCommand, DroneEvent},
    drone::Drone,
    network::NodeId,
    packet::{Packet, PacketType},
};

use gui::commands::{GUICommands, GUIEvents};
use messages::{
    client_commands::{ChatClientCommand, ChatClientEvent, MediaClientCommand, MediaClientEvent},
    server_commands::{CommunicationServerCommand, CommunicationServerEvent, ContentServerCommand, ContentServerEvent},
};

use crate::{action, verify};

pub struct SimulationController {
    pub drones: HashMap<NodeId, (Sender<DroneCommand>, Sender<Packet>)>,
    drone_recv: Receiver<DroneEvent>,
    pub neighbor: HashMap<NodeId, Vec<NodeId>>,
    pub event_send: Sender<DroneEvent>,
    pub new_drones: Vec<Box<dyn Drone>>,

    gui_send: Sender<GUIEvents>,
    gui_recv: Receiver<GUICommands>,

    pub cclients: HashMap<NodeId, (Sender<ChatClientCommand>, Sender<Packet>)>,
    cclient_recv: Receiver<ChatClientEvent>,

    pub mclients: HashMap<NodeId, (Sender<MediaClientCommand>, Sender<Packet>)>,
    mclient_recv: Receiver<MediaClientEvent>,

    pub comm_servers: HashMap<NodeId, (Sender<CommunicationServerCommand>, Sender<Packet>)>,
    comm_server_recv: Receiver<CommunicationServerEvent>,

    pub text_servers: HashMap<NodeId, (Sender<ContentServerCommand>, Sender<Packet>)>,
    text_recv: Receiver<ContentServerEvent>,

    pub media_servers: HashMap<NodeId, (Sender<ContentServerCommand>, Sender<Packet>)>,
    media_recv: Receiver<ContentServerEvent>,
}

impl SimulationController {
    pub fn new(
        drones: HashMap<NodeId, (Sender<DroneCommand>, Sender<Packet>)>,
        drone_recv: Receiver<DroneEvent>,
        neighbor: HashMap<NodeId, Vec<NodeId>>,
        event_send: Sender<DroneEvent>,
        gui_send: Sender<GUIEvents>,
        gui_recv: Receiver<GUICommands>,
        cclients: HashMap<NodeId, (Sender<ChatClientCommand>, Sender<Packet>)>,
        cclient_recv: Receiver<ChatClientEvent>,
        mclients: HashMap<NodeId, (Sender<MediaClientCommand>, Sender<Packet>)>,
        mclient_recv: Receiver<MediaClientEvent>,
        comm_servers: HashMap<NodeId, (Sender<CommunicationServerCommand>, Sender<Packet>)>,
        comm_server_recv: Receiver<CommunicationServerEvent>,
        text_servers: HashMap<NodeId, (Sender<ContentServerCommand>, Sender<Packet>)>,
        text_recv: Receiver<ContentServerEvent>,
        media_servers: HashMap<NodeId, (Sender<ContentServerCommand>, Sender<Packet>)>,
        media_recv: Receiver<ContentServerEvent>
    ) -> Self {
        return Self {
            drones,
            drone_recv,
            neighbor,
            event_send,
            new_drones: Vec::new(),
            gui_send,
            gui_recv,
            cclients,
            cclient_recv,
            mclients,
            mclient_recv,
            comm_servers,
            comm_server_recv,
            text_servers,
            text_recv,
            media_servers,
            media_recv
        };
    }

    pub fn run(&mut self) {
        info!(
            "[ {} ] Starting Simulation Controller",
            "Simulation Controller".green()
        );

        thread::sleep(std::time::Duration::from_secs(2));

        // Init ChatClient
        for (chat_client, _) in self.cclients.clone().iter() {
            self.handle_cclient_command(chat_client, ChatClientCommand::InitFlooding);
            thread::sleep(std::time::Duration::from_secs(5));
            self.handle_cclient_command(chat_client, ChatClientCommand::StartChatClient);
            thread::sleep(std::time::Duration::from_secs(5));
        }

        // Start loop
        loop {
            select! {
                recv(self.drone_recv) -> drone_event => match drone_event {
                    Ok(drone_event) => {
                        info!("[ {} ]: DroneEvent received", "Simulation Controller".green());
                        self.handle_drone_event(drone_event);
                    }
                    Err(e) => {
                        error!("[ {} ]: DroneEvent receiver channel disconnected: {}", "Simulation Controller".red(), e);
                        break;
                    }
                },
                recv(self.cclient_recv) -> cclient_event => match cclient_event {
                    Ok(cclient_event) => {
                        info!("[ {} ]: ChatClientEvent received", "Simulation Controller".green());
                        self.handle_cclient_event(cclient_event);
                    }
                    Err(e) => {
                        error!("[ {} ]: ChatClientEvent receiver channel disconnected: {}", "Simulation Controller".red(), e);
                        break;
                    }
                },
                recv(self.mclient_recv) -> mclient_event => match mclient_event {
                    Ok(mclient_event) => {
                        info!("[ {} ]: MediaClientEvent received", "Simulation Controller".green());
                        self.handle_mclient_event(mclient_event);
                    }
                    Err(e) => {
                        error!("[ {} ]: MediaClientEvent receiver channel disconnected: {}", "Simulation Controller".red(), e);
                        break;
                    }
                },
                recv(self.comm_server_recv) -> comm_event => match comm_event {
                    Ok(comm_event) => {
                        info!("[ {} ]: CommunicationServer received", "Simulation Controller".green());
                        self.handle_commserver_event(comm_event);
                    }
                    Err(e) => {
                        error!("[ {} ]: MediaClientEvent receiver channel disconnected: {}", "Simulation Controller".red(), e);
                        break;
                    }
                },
                recv(self.text_recv) -> text_event => match text_event {
                    Ok(text_event) => {
                        info!("[ {} ]: TextContentServer received", "Simulation Controller".green());
                        self.handle_text_event(text_event);
                    }
                    Err(e) => {
                        error!("[ {} ]: MediaClientEvent receiver channel disconnected: {}", "Simulation Controller".red(), e);
                        break;
                    }
                },
                recv(self.media_recv) -> media_event => match media_event {
                    Ok(media_event) => {
                        info!("[ {} ]: MediaClientEvent received", "Simulation Controller".green());
                        self.handle_media_event(media_event);
                    }
                    Err(e) => {
                        error!("[ {} ]: MediaClientEvent receiver channel disconnected: {}", "Simulation Controller".red(), e);
                        break;
                    }
                },
                recv(self.gui_recv) -> gui_command => match gui_command {
                    Ok(gui_command) => {
                        info!("[ {} ]: GUICommand received", "Simulation Controller".green());
                        self.handle_gui_command(gui_command);
                    }
                    Err(e) => {
                        error!("[ {} ]: GUICommands receiver channel disconnected: {}", "Simulation Controller".red(), e);
                        break;
                    }
                },
            }

            //////////////////////////////////////////////////////////// REMOVE
            thread::sleep(std::time::Duration::from_secs_f32(0.01));
        }
    }

    // Handle Drone Events
    pub fn handle_drone_event(&self, drone_event: DroneEvent) {
        match drone_event {
            DroneEvent::PacketSent(packet) => {
                /*info!(
                    "[ {} ] Is a {}",
                    "Simulation Controller".yellow(),
                    packet.pack_type
                );*/
            }
            DroneEvent::PacketDropped(packet) => {
                info!(
                    "[ {} ] Is a {}",
                    "Simulation Controller".yellow(),
                    packet.pack_type
                );
            }
            DroneEvent::ControllerShortcut(packet) => {
                info!(
                    "[ {} ] Is a {}",
                    "Simulation Controller".yellow(),
                    packet.pack_type
                );

                // Get packet destination node
                if let Some(dest) = packet
                    .routing_header
                    .hops
                    .get(packet.routing_header.len() - 1)
                {
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
                            "[ {} ]: failed to find a Sender<Packet> channel for the [ Drone {} ]",
                            "Simulation Controller".red(),
                            dest
                        );
                        return;
                    }
                    
                    // Send Packet to destination
                    match packet.pack_type {
                        PacketType::MsgFragment(_) => {
                            panic!("Impossible how the hell did u do this")
                        }
                        _ => {
                            packet_channel.send(packet.clone()).unwrap();
                        }
                    }
                } else {
                    error!(
                        "[ {} ]: failed to find a Drone to send the DroneEvent::ControllerShortcut",
                        "Simulation Controller".red()
                    );
                }
            }
        }
    }

    // Handle Drone Commands
    pub fn handle_drone_command(&mut self, drone: &NodeId, drone_command: DroneCommand) {
        // Get drone channel
        if let Some((command_channel, _)) = self.drones.get(drone) {
            match drone_command {
                DroneCommand::RemoveSender(node_id) => {
                    if let Some(vec) = self.neighbor.get_mut(drone) {
                        vec.retain(|x| *x != node_id);
                        match command_channel.send(DroneCommand::RemoveSender(node_id)) {
                            Ok(()) => info!(
                                "[ {} ]: sent a DroneCommand::RemoveSender({}) to [ Drone {} ]",
                                "Simulation Controller".green(),
                                node_id,
                                drone
                            ),
                            Err(e) => error!(
                                "[ {} ]: failed to send a DroneCommand::RemoveSender({}) to the [ Drone {} ]: {}",
                                "Simulation Controller".red(),
                                node_id,
                                drone,
                                e
                            ),
                        }
                    } else {
                        error!(
                            "[ {} ]: the [ Drone {} ] does not have any neighbor",
                            "Simulation Controller".red(),
                            drone
                        );
                    }
                }
                DroneCommand::AddSender(node_id, sender) => {
                    if let Some(vec) = self.neighbor.get_mut(drone) {
                        vec.push(node_id);
                        match command_channel.send(DroneCommand::AddSender(node_id, sender)) {
                            Ok(()) => info!(
                                "[ {} ]: sent a DroneCommand::AddSender({}, sender_channel) to [ Drone {} ]",
                                "Simulation Controller".green(),
                                node_id,
                                drone
                            ),
                            Err(e) => error!(
                                "[ {} ]: failed to send a DroneCommand::AddSender({}, sender_channel) to the [ Drone {} ]: {}",
                                "Simulation Controller".red(),
                                node_id,
                                drone,
                                e
                            ),
                        }
                    } else {
                        error!(
                            "[ {} ]: the [ Drone {} ] does not have any neighbor",
                            "Simulation Controller".red(),
                            drone
                        );
                    }
                }
                DroneCommand::SetPacketDropRate(pdr) => {
                    match command_channel.send(DroneCommand::SetPacketDropRate(pdr)) {
                        Ok(()) => info!(
                            "[ {} ]: sent a DroneCommand::SetPacketDropRate({}) to [ Drone {} ]",
                            "Simulation Controller".green(),
                            pdr,
                            drone
                        ),
                        Err(e) => error!(
                            "[ {} ]: failed to send a DroneCommand::SetPacketDropRate({}) to the [ Drone {} ]: {}",
                            "Simulation Controller".red(),
                            pdr,
                            drone,
                            e
                        ),
                    }
                }
                DroneCommand::Crash => {
                    if let Some((command_send, packet_send)) = self.drones.get(drone) {
                        let _ = drop(command_send);
                        let _ = drop(packet_send);
                    }

                    let drone_entry = self.drones.remove(drone);

                    self.neighbor.remove(drone);

                    if let Some((command_channel, _)) = drone_entry {
                        match command_channel.send(DroneCommand::Crash) {
                            Ok(()) => info!(
                                "[ {} ]: sent a DroneCommand::Crash() to [ Drone {} ]",
                                "Simulation Controller".green(),
                                drone
                            ),
                            Err(e) => error!(
                                "[ {} ]: failed to send a DroneCommand::Crash() to the [ Drone {} ]: {}",
                                "Simulation Controller".red(),
                                drone,
                                e
                            ),
                        }
                    } else {
                        error!(
                            "[ {} ]: the [ Drone {} ] was not found in the drones map",
                            "Simulation Controller".red(),
                            drone
                        );
                    }
                }
            }
        } else {
            error!(
                "[ {} ]: failed to find a Sender<DroneCommand> channel for the [ Drone {} ]",
                "Simulation Controller".red(),
                drone
            );
        }
    }

    // Handle GUI Commands
    fn handle_gui_command(&mut self, command: GUICommands) {
        match command {
            GUICommands::Spawn(id, connected_node_ids, pdr) => {
                match action::spawn(self, id, connected_node_ids, pdr) {
                    Ok(()) => return,
                    Err(e) => {
                        error!("{}", e);
                    }
                }
            }
            GUICommands::Crash(drone) => match action::crash(self, drone) {
                Ok(()) => self.handle_drone_command(&drone, DroneCommand::Crash),
                Err(e) => error!("{}", e),
            },
            GUICommands::RemoveSender(node_id, to_remove) => {
                match action::remove_sender(self, &node_id, &to_remove) {
                    Ok(()) => {
                        if self.drones.contains_key(&node_id) {
                            self.handle_drone_command(
                                &node_id,
                                DroneCommand::RemoveSender(to_remove),
                            )
                        } else if self.cclients.contains_key(&node_id) {
                            self.handle_cclient_command(
                                &node_id,
                                ChatClientCommand::RemoveSender(to_remove),
                            )
                        } else if self.mclients.contains_key(&node_id) {
                            self.handle_mclient_command(
                                &node_id,
                                MediaClientCommand::RemoveSender(to_remove),
                            )
                        } else if self.comm_servers.contains_key(&node_id) {
                            self.handle_commserver_command(
                                &node_id,
                                CommunicationServerCommand::RemoveSender(to_remove),
                            )
                        }
                    }
                    Err(e) => error!("{}", e),
                }
            }
            GUICommands::AddSender(node_id, to_add) => {
                match action::add_sender(self, &node_id, &to_add) {
                    Ok(()) => {
                        let sender;
                        if self.drones.contains_key(&to_add) {
                            (_, sender) = self.drones.get(&to_add).unwrap().clone();
                        } else if self.cclients.contains_key(&to_add) {
                            (_, sender) = self.cclients.get(&to_add).unwrap().clone();
                        } else if self.mclients.contains_key(&to_add) {
                            (_, sender) = self.mclients.get(&to_add).unwrap().clone();
                        } else {
                            (_, sender) = self.comm_servers.get(&to_add).unwrap().clone();
                        }

                        if self.drones.contains_key(&node_id) {
                            self.handle_drone_command(
                                &node_id,
                                DroneCommand::AddSender(to_add, sender),
                            );
                        } else if self.cclients.contains_key(&node_id) {
                            self.handle_cclient_command(
                                &node_id,
                                ChatClientCommand::AddSender(to_add, sender),
                            );
                        } else if self.mclients.contains_key(&node_id) {
                            self.handle_mclient_command(
                                &node_id,
                                MediaClientCommand::AddSender(to_add, sender),
                            );
                        } else if self.comm_servers.contains_key(&node_id) {
                            self.handle_commserver_command(
                                &node_id,
                                CommunicationServerCommand::AddSender(to_add, sender),
                            );
                        }
                    }
                    Err(e) => error!("{}", e),
                }
            }
            GUICommands::SetPDR(drone, pdr) => match verify::valid_pdr(pdr) {
                Ok(value) => {
                    self.handle_drone_command(&drone, DroneCommand::SetPacketDropRate(value))
                }
                Err(e) => error!("{}", e),
            },

            GUICommands::SendMessageTo(src, dest, msg) => {
                self.handle_cclient_command(&src, ChatClientCommand::SendMessageTo(dest, msg))
            },
            GUICommands::RegisterTo(client, server) => {
                self.handle_cclient_command(&client, ChatClientCommand::RegisterTo(server))
            },
            GUICommands::LogOut(client, server) => {
                self.handle_cclient_command(&client, ChatClientCommand::LogOut)
            },
            GUICommands::AskForFileList(client, server) => {
                self.handle_mclient_command(&client, MediaClientCommand::AskFilesList(server));
            },
            GUICommands::GetFile(client, server, title) => {
                self.handle_mclient_command(&client, MediaClientCommand::AskForFile(server, title));
            }
        }
    }

    // Handle ChatClient Event
    fn handle_cclient_event(&mut self, event: ChatClientEvent) {
        match event {
            ChatClientEvent::CommunicationServerList(items) => {
                info!(
                    "The Client retrieved the CommunicationServers list: {:?}",
                    items
                );
            }
            ChatClientEvent::MessageReceived(src, msg) => {
                info!(
                    "[ Client: {} ]: received the message {:?} from [ Server {} ]",
                    src, msg, src
                );
            }
            ChatClientEvent::SuccessfulRegistration(server) => info!(
                "[ {} ]: The Client successfully register to [ Server {}]",
                "Simulation Controller".green(),
                server
            ),
            ChatClientEvent::ClientList(client_list) => {
                info!("The Client retrieved the Client list: {:?}", client_list);
            }
            ChatClientEvent::SuccessfulLogOut => info!(
                "[ {} ]: The Client successfully logged out from server",
                "Simulation Controller".green(),
            ),
            ChatClientEvent::UnreachableClient(client) => {
                error!(
                    "[ {} ]: received an error message: [ Client {} ] is not register on the selected server",
                    "Simulation Controller".red(),
                    client,
                );
            }
            ChatClientEvent::ErrorNotRunning => {
                error!(
                    "[ {} ]: received an error message: The Client tried to register without previously running ChatClientCommand::StartChatClient",
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
                if let Some(dest) = packet
                    .routing_header
                    .hops
                    .get(packet.routing_header.len() - 1)
                {
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
                            panic!("Impossible how the hell did u do this")
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
                                Ok(()) => info!(
                                    "[ {} ]: sent a ChatClientCommand::RemoveSender({}) to [ ChatClient {} ]",
                                    "Simulation Controller".green(),
                                    drone,
                                    chat_client
                                ),
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
                if !self.cclients.contains_key(&drone) {
                    if let Some(neighbors) = self.neighbor.get_mut(chat_client) {
                        // Max 2 neighbor, Min 1 neighbor
                        if neighbors.len() == 1 {
                            neighbors.push(drone);
                            if let Some((client, _)) = self.cclients.get(chat_client) {
                                match client.send(ChatClientCommand::AddSender(drone, sender.clone())) {
                                    Ok(()) => info!(
                                        "[ {} ]: sent a ChatClientCommand::AddSender({}, {:?}) to [ ChatClient {} ]",
                                        "Simulation Controller".green(),
                                        drone,
                                        sender,
                                        chat_client
                                    ),
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
                } else {
                    error!(
                        "[ {} ]: The selected NodeId: {} correspond to a Client not a Drone",
                        "Simulation Controller".red(),
                        chat_client
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
        }
    }

    // Handle MediaClient Event
    fn handle_mclient_event(&mut self, event: MediaClientEvent) {
        match event {
            MediaClientEvent::ReceveidFloodResponse => {
                info!(
                    "[ {} ]: The media client retrieved a FloodResponse",
                    "Simulation Controller".green()
                )
            }
            MediaClientEvent::RemovedSender(drone) => {
                info!(
                    "[ {} ]: The media client removed the neighbor [ Drone {} ]",
                    "Simulation Controller".green(),
                    drone
                )
            }
            MediaClientEvent::AddedSender(drone) => {
                info!(
                    "[ {} ]: The media client added the neighbor [ Drone {} ]",
                    "Simulation Controller".green(),
                    drone
                )
            }
            MediaClientEvent::UnreachableNode(node) => {
                error!(
                    "[ {} ]: received an error message: The [ Node {} ] is not reachable",
                    "Simulation Controller".red(),
                    node
                );
            }
            MediaClientEvent::DestinationIsDrone => {
                error!(
                    "[ {} ]: received an error message: The selected destination is a drone",
                    "Simulation Controller".red(),
                );
            }
            MediaClientEvent::ErrorPacketCache(session_id, fragment_index) => {
                error!(
                    "[ {} ]: received an error message: Error in the packet cache [ session_id : {}, fragment_index: {} ]",
                    "Simulation Controller".red(),
                    session_id,
                    fragment_index
                );
            }
            MediaClientEvent::SendError(e) => {
                error!(
                    "[ {} ]: received an error message: It has verified a SenderError: {}",
                    "Simulation Controller".red(),
                    e
                );
            }
            MediaClientEvent::ReceveidFileList(server, items) => {
                info!(
                    "[ {} ]: received the file list of [ TextServer {} ]",
                    "Simulation Controller".green(),
                    server,
                );
                self.gui_send.send(GUIEvents::FileList(server, items));
            },
            MediaClientEvent::ReceveidFile(node_id, file_id, file_response) => {
                info!(
                    "[ {} ]: received a file from [ MediaClient {} ]",
                    "Simulation Controller".green(),
                    node_id,
                );
                self.gui_send.send(GUIEvents::MessageReceived(node_id, file_response));
            }
            MediaClientEvent::ControllerShortcut(packet) => {
                if let Some(dest) = packet
                    .routing_header
                    .hops
                    .get(packet.routing_header.len() - 1)
                {
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
                            "[ {} ]: failed to find a Sender<Packet> channel for the [ Drone {} ]",
                            "Simulation Controller".red(),
                            dest
                        );
                        return;
                    }
                    
                    // Send Packet to destination
                    match packet.pack_type {
                        PacketType::MsgFragment(_) => {
                            panic!("Impossible how the hell did u do this")
                        }
                        _ => {
                            packet_channel.send(packet.clone()).unwrap();
                        }
                    }
                } else {
                    error!(
                        "[ {} ]: failed to find a MediaClient to send the MediaClientCommand::ControllerShortcut",
                        "Simulation Controller".red()
                    );
                }
            }
            //MediaClientEvent::ServerList(items) => todo!(),
            //MediaClientEvent::ReceveidServerType(_, server_type) => todo!(),
            _ => {
                error!("NOPE -> Not Implemented");
            }            
        }
    }

    // Handle MediaClient Command
    pub fn handle_mclient_command(&mut self, media_client: &NodeId, command: MediaClientCommand) {
        match command {
            MediaClientCommand::InitFlooding => {
                if let Some((client, _)) = self.mclients.get(media_client) {
                    match client.send(MediaClientCommand::InitFlooding) {
                        Ok(()) => info!(
                            "[ {} ]: sent a MediaClientCommand::InitFlo0ding to [ Client {} ]",
                            "Simulation Controller".green(),
                            media_client
                        ),
                        Err(e) => error!(
                            "[ {} ]: failed to send a MediaClientCommand::InitFlooding to the [ Client {} ]: {}",
                            "Simulation Controller".red(),
                            media_client,
                            e
                        ),
                    }
                } else {
                    error!(
                        "[ {} ]: failed to find a Sender<MediaClientCommand> channel for the [ Client {} ]",
                        "Simulation Controller".red(),
                        media_client
                    );
                }
            }
            MediaClientCommand::RemoveSender(drone) => {
                if let Some(neighbors) = self.neighbor.get_mut(media_client) {
                    // Max 2 neighbor, Min 1 neighbor
                    if neighbors.len() == 2 {
                        neighbors.retain(|x| *x != drone);
                        if let Some((client, _)) = self.mclients.get(media_client) {
                            match client.send(MediaClientCommand::RemoveSender(drone)) {
                                Ok(()) => info!(
                                    "[ {} ]: sent a MediaClientCommand::RemoveSender({}) to [ Client {} ]",
                                    "Simulation Controller".green(),
                                    drone,
                                    media_client
                                ),
                                Err(e) => error!(
                                    "[ {} ]: failed to send a MediaClientCommand::RemoveSender({}) to the [ Client {} ]: {}",
                                    "Simulation Controller".red(),
                                    drone,
                                    media_client,
                                    e
                                ),
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
                            "[ {} ]: failed to send a MediaClientCommand::RemoveSender({}) to the [ Client {} ]: {}",
                            "Simulation Controller".red(),
                            drone,
                            media_client,
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
            MediaClientCommand::AddSender(drone, sender) => {
                // cant connect to a client
                if !self.mclients.contains_key(&drone) {
                    if let Some(neighbors) = self.neighbor.get_mut(media_client) {
                        // Max 2 neighbor, Min 1 neighbor
                        if neighbors.len() == 1 {
                            neighbors.push(drone);
                            if let Some((client, _)) = self.mclients.get(media_client) {
                                match client.send(MediaClientCommand::AddSender(drone, sender.clone())) {
                                    Ok(()) => info!(
                                        "[ {} ]: sent a MediaClientCommand::AddSender({}, {:?}) to [ Client {} ]",
                                        "Simulation Controller".green(),
                                        drone,
                                        sender,
                                        media_client
                                    ),
                                    Err(e) => error!(
                                        "[ {} ]: failed to send a MediaClientCommand::AddSender({}, {:?}) to the [ Client {} ]: {}",
                                        "Simulation Controller".red(),
                                        drone,
                                        sender,
                                        media_client,
                                        e
                                    ),
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
                                "[ {} ]: failed to send a MediaClientCommand::AddSender({}, {:?}) to the [ Client {} ]: {}",
                                "Simulation Controller".red(),
                                drone,
                                sender,
                                media_client,
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
                } else {
                    error!(
                        "[ {} ]: The selected NodeId: {} correspond to a Client not a Drone",
                        "Simulation Controller".red(),
                        media_client
                    );
                }
            }
            MediaClientCommand::AskFilesList(server) => {
                if let Some((client, _)) = self.mclients.get(media_client) {
                    match client.send(MediaClientCommand::AskFilesList(server)) {
                        Ok(()) => info!(
                            "[ {} ]: sent a MediaClientCommand::AskFilesList({}) to [ Client {} ]",
                            "Simulation Controller".green(),
                            server,
                            media_client
                        ),
                        Err(e) => error!(
                            "[ {} ]: failed to send a MediaClientCommand::AskFilesList({}) to the [ Client {} ]: {}",
                            "Simulation Controller".red(),
                            server,
                            media_client,
                            e
                        ),
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
                if let Some((client, _)) = self.mclients.get(media_client) {
                    match client.send(MediaClientCommand::AskForFile(server, title.clone())) {
                        Ok(()) => info!(
                            "[ {} ]: sent a MediaClientCommand::AskForFile({}, {}) to [ Client {} ]",
                            "Simulation Controller".green(),
                            server,
                            title,
                            media_client
                        ),
                        Err(e) => error!(
                            "[ {} ]: failed to send a MediaClientCommand::AskForFile({}, {}) to the [ Client {} ]: {}",
                            "Simulation Controller".red(),
                            server,
                            title,
                            media_client,
                            e
                        ),
                    }
                } else {
                    error!(
                        "[ {} ]: failed to find a Sender<MediaClientCommand> channel for the [ Client {} ]",
                        "Simulation Controller".red(),
                        media_client
                    );
                }
            }
            //MediaClientCommand::GetServerList => todo!(),
            //MediaClientCommand::AskServerType(_) => todo!(),
            _ => {
                error!("NOPE -> Not Implemented");
            }
        }
    }

    // Handle Server Command
    fn handle_commserver_event(&mut self, event: CommunicationServerEvent) {
        match event {
            CommunicationServerEvent::ServerStarted => {
                info!(
                    "[ {} ]: CommunicationServer started successfully",
                    "Simulation Controller".green(),
                )
            }
            CommunicationServerEvent::ServerStopped => {
                info!(
                    "[ {} ]: CommunicationServer stopped successfully",
                    "Simulation Controller".green(),
                )
            }
            CommunicationServerEvent::ClientRegistered(client) => {
                info!(
                    "[ {} ]: CommunicationServer registered [ Client {} ]",
                    "Simulation Controller".green(),
                    client,
                )
            }
            CommunicationServerEvent::ClientDeregistered(client) => {
                info!(
                    "[ {} ]: CommunicationServer deregistered [ Client {} ]",
                    "Simulation Controller".green(),
                    client,
                )
            }
            CommunicationServerEvent::MessageForwarded(dest, msg) => {
                info!(
                    "[ {} ]: CommunicationServer forwarded the message {:?} to [ Client {} ]",
                    "Simulation Controller".green(),
                    msg,
                    dest
                )
            }
            CommunicationServerEvent::MessageReceived(src, msg) => info!(
                "[ {} ]: CommunicationServer received the message {:?} from [ Client {} ]",
                "Simulation Controller".green(),
                msg,
                src
            ),
            CommunicationServerEvent::UnreachableClient(client) => {
                error!(
                    "[ {} ]: received an error message: [ Client {} ] is unreachable",
                    "Simulation Controller".red(),
                    client,
                );
            }
            CommunicationServerEvent::SendError(e) => {
                error!(
                    "[ {} ]: received an error message: It has verified a SenderError: {}",
                    "Simulation Controller".red(),
                    e
                );
            }
            CommunicationServerEvent::ControllerShortcut(packet) => {
                if let Some(dest) = packet
                    .routing_header
                    .hops
                    .get(packet.routing_header.len() - 1)
                {
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
                            "[ {} ]: failed to find a Sender<Packet> channel for the [ CommunicationServer {} ]",
                            "Simulation Controller".red(),
                            dest
                        );
                        return;
                    }
                    
                    // Send Packet to destination
                    match packet.pack_type {
                        PacketType::MsgFragment(_) => {
                            panic!("Impossible how the hell did u do this")
                        }
                        _ => {
                            packet_channel.send(packet.clone()).unwrap();
                        }
                    }
                } else {
                    error!(
                        "[ {} ]: failed to find a CommunicationServer to send the CommunicationServerCommand::ControllerShortcut",
                        "Simulation Controller".red()
                    );
                }
            },
            CommunicationServerEvent::DestinationIsDrone(drone) => {
                error!(
                    "[ {} ]: received an error message: The selected destination is a drone [ Drone {} ]",
                    "Simulation Controller".red(),
                    drone
                );
            },
            CommunicationServerEvent::ErrorPacketCache(session_id, fragment_index) => {
                error!(
                    "[ {} ]: received an error message: Error in the packet cache [ session_id : {}, fragment_index: {} ]",
                    "Simulation Controller".red(),
                    session_id,
                    fragment_index
                );
            },
        }
    }

    // Handle Server Command
    pub fn handle_commserver_command(
        &mut self,
        comm_server: &NodeId,
        command: CommunicationServerCommand,
    ) {
        match command {
            CommunicationServerCommand::InitFlooding => {
                if let Some((server, _)) = self.comm_servers.get(comm_server) {
                    match server.send(CommunicationServerCommand::InitFlooding) {
                        Ok(()) => info!(
                            "[ {} ]: sent a CommunicationServerCommand::InitFlooding to [ CommunicationServer {} ]",
                            "Simulation Controller".green(),
                            comm_server
                        ),
                        Err(e) => error!(
                            "[ {} ]: failed to send a CommunicationServerCommand::InitFlooding to the [ CommunicationServer {} ]: {}",
                            "Simulation Controller".red(),
                            comm_server,
                            e
                        ),
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
                if let Some((server, _)) = self.comm_servers.get(comm_server) {
                    if let Some(vec) = self.neighbor.get_mut(comm_server) {
                        vec.push(node_id);
                        match server.send(CommunicationServerCommand::AddSender(node_id, sender)) {
                            Ok(()) => info!(
                                "[ {} ]: sent a CommunicationServerCommand::AddSender({}, sender_channel) to [ CommunicationServer {} ]",
                                "Simulation Controller".green(),
                                node_id,
                                comm_server
                            ),
                            Err(e) => error!(
                                "[ {} ]: failed to send a CommunicationServerCommand::AddSender({}, sender_channel) to the [ CommunicationServer {} ]: {}",
                                "Simulation Controller".red(),
                                node_id,
                                comm_server,
                                e
                            ),
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
                if let Some((server, _)) = self.comm_servers.get(comm_server) {
                    if let Some(vec) = self.neighbor.get_mut(comm_server) {
                        if vec.len() > 2 {
                            vec.retain(|x| *x != node_id);
                            match server.send(CommunicationServerCommand::RemoveSender(node_id)) {
                                Ok(()) => {
                                    info!(
                                    "[ {} ]: sent a CommunicationServerCommand::RemoveSender({}) to [ CommunicationServer {} ]",
                                    "Simulation Controller".green(),
                                    node_id,
                                    comm_server
                                );},
                                Err(e) => error!(
                                    "[ {} ]: failed to send a CommunicationServerCommand::RemoveSender({}) to the [ CommunicationServer {} ]: {}",
                                    "Simulation Controller".red(),
                                    node_id,
                                    comm_server,
                                    e
                                ),
                            }
                        } else {
                            error!(
                                "[ {} ]: the [ CommunicationServer {} ] must be connected to at least two nodes",
                                "Simulation Controller".red(),
                                comm_server
                            );
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
        }
    }

    fn handle_text_event(&mut self, event: ContentServerEvent) {
        match event {
            ContentServerEvent::ServerStarted => {
                info!(
                    "[ {} ]: TextContentServer started successfully",
                    "Simulation Controller".green(),
                )
            }
            ContentServerEvent::ServerStopped => {
                info!(
                    "[ {} ]: TextContentServer stopped successfully",
                    "Simulation Controller".green(),
                )
            }
            ContentServerEvent::MessageForwarded(dest, msg) => {
                info!(
                    "[ {} ]: TextContentServer forwarded the message {:?} to [ Client {} ]",
                    "Simulation Controller".green(),
                    msg,
                    dest
                )
            }
            ContentServerEvent::MessageReceived(src, msg) => info!(
                "[ {} ]: TextContentServer received the message {:?} from [ Client {} ]",
                "Simulation Controller".green(),
                msg,
                src
            ),
            ContentServerEvent::SendError(e) => {
                error!(
                    "[ {} ]: received an error message: It has verified a SenderError: {}",
                    "Simulation Controller".red(),
                    e
                );
            }
            ContentServerEvent::DestinationIsDrone(drone) => {
                error!(
                    "[ {} ]: received an error message: The selected destination is a drone [ Drone {} ]",
                    "Simulation Controller".red(),
                    drone
                );
            },
            ContentServerEvent::ErrorPacketCache(session_id, fragment_index) => {
                error!(
                    "[ {} ]: received an error message: Error in the packet cache [ session_id : {}, fragment_index: {} ]",
                    "Simulation Controller".red(),
                    session_id,
                    fragment_index
                );
            },
            ContentServerEvent::ControllerShortcut(packet) => {
                if let Some(dest) = packet
                    .routing_header
                    .hops
                    .get(packet.routing_header.len() - 1)
                {
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
                            "[ {} ]: failed to find a Sender<Packet> channel for the [ CommunicationServer {} ]",
                            "Simulation Controller".red(),
                            dest
                        );
                        return;
                    }
                    
                    // Send Packet to destination
                    match packet.pack_type {
                        PacketType::MsgFragment(_) => {
                            panic!("Impossible how the hell did u do this")
                        }
                        _ => {
                            packet_channel.send(packet.clone()).unwrap();
                        }
                    }
                } else {
                    error!(
                        "[ {} ]: failed to find a CommunicationServer to send the CommunicationServerCommand::ControllerShortcut",
                        "Simulation Controller".red()
                    );
                }
            }
            ContentServerEvent::UnreachableClient(_) => {
                error!("NOPE -> Not Implemented");
            }
        }
    }

    pub fn handle_text_command(&mut self, text_server: &NodeId, command: ContentServerCommand) {
        match command {
            ContentServerCommand::InitFlooding => {
                if let Some((server, _)) = self.text_servers.get(text_server) {
                    match server.send(ContentServerCommand::InitFlooding) {
                        Ok(()) => info!(
                            "[ {} ]: sent a ContentServerCommand::InitFlooding to [ TextContentServer {} ]",
                            "Simulation Controller".green(),
                            text_server
                        ),
                        Err(e) => error!(
                            "[ {} ]: failed to send a ContentServerCommand::InitFlooding to the [ TextContentServer {} ]: {}",
                            "Simulation Controller".red(),
                            text_server,
                            e
                        ),
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
                if let Some((server, _)) = self.text_servers.get(text_server) {
                    if let Some(vec) = self.neighbor.get_mut(text_server) {
                        vec.push(node_id);
                        match server.send(ContentServerCommand::AddSender(node_id, sender)) {
                            Ok(()) => info!(
                                "[ {} ]: sent a ContentServerCommand::AddSender({}, sender_channel) to [ TextContentServer {} ]",
                                "Simulation Controller".green(),
                                node_id,
                                text_server
                            ),
                            Err(e) => error!(
                                "[ {} ]: failed to send a ContentServerCommand::AddSender({}, sender_channel) to the [ TextContentServer {} ]: {}",
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
            ContentServerCommand::RemoveSender(node_id) => {
                if let Some((server, _)) = self.text_servers.get(text_server) {
                    if let Some(vec) = self.neighbor.get_mut(text_server) {
                        if vec.len() > 2 {
                            vec.retain(|x| *x != node_id);
                            match server.send(ContentServerCommand::RemoveSender(node_id)) {
                                Ok(()) => {
                                    info!(
                                    "[ {} ]: sent a ContentServerCommand::RemoveSender({}) to [ TextContentServer {} ]",
                                    "Simulation Controller".green(),
                                    node_id,
                                    text_server
                                );},
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
                                "[ {} ]: the [ TextContentServer {} ] must be connected to at least two nodes",
                                "Simulation Controller".red(),
                                text_server
                            );
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

    fn handle_media_event(&mut self, event: ContentServerEvent) {
        match event {
            ContentServerEvent::ServerStarted => {
                info!(
                    "[ {} ]: MediaContentServer started successfully",
                    "Simulation Controller".green(),
                )
            }
            ContentServerEvent::ServerStopped => {
                info!(
                    "[ {} ]: MediaContentServer stopped successfully",
                    "Simulation Controller".green(),
                )
            }
            ContentServerEvent::MessageForwarded(dest, msg) => {
                info!(
                    "[ {} ]: MediaContentServer forwarded the message {:?} to [ Client {} ]",
                    "Simulation Controller".green(),
                    msg,
                    dest
                )
            }
            ContentServerEvent::MessageReceived(src, msg) => info!(
                "[ {} ]: MediaContentServer received the message {:?} from [ Client {} ]",
                "Simulation Controller".green(),
                msg,
                src
            ),
            ContentServerEvent::SendError(e) => {
                error!(
                    "[ {} ]: received an error message: It has verified a SenderError: {}",
                    "Simulation Controller".red(),
                    e
                );
            }
            ContentServerEvent::DestinationIsDrone(drone) => {
                error!(
                    "[ {} ]: received an error message: The selected destination is a drone [ Drone {} ]",
                    "Simulation Controller".red(),
                    drone
                );
            },
            ContentServerEvent::ErrorPacketCache(session_id, fragment_index) => {
                error!(
                    "[ {} ]: received an error message: Error in the packet cache [ session_id : {}, fragment_index: {} ]",
                    "Simulation Controller".red(),
                    session_id,
                    fragment_index
                );
            },
            ContentServerEvent::ControllerShortcut(packet) => {
                if let Some(dest) = packet
                    .routing_header
                    .hops
                    .get(packet.routing_header.len() - 1)
                {
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
                            "[ {} ]: failed to find a Sender<Packet> channel for the [ CommunicationServer {} ]",
                            "Simulation Controller".red(),
                            dest
                        );
                        return;
                    }
                    
                    // Send Packet to destination
                    match packet.pack_type {
                        PacketType::MsgFragment(_) => {
                            panic!("Impossible how the hell did u do this")
                        }
                        _ => {
                            packet_channel.send(packet.clone()).unwrap();
                        }
                    }
                } else {
                    error!(
                        "[ {} ]: failed to find a CommunicationServer to send the CommunicationServerCommand::ControllerShortcut",
                        "Simulation Controller".red()
                    );
                }
            }
            ContentServerEvent::UnreachableClient(_) => {
                error!("NOPE -> Not Implemented");
            }
        }
    }

    pub fn handle_media_command(&mut self, media_server: &NodeId, command: ContentServerCommand) {
        match command {
            ContentServerCommand::InitFlooding => {
                if let Some((server, _)) = self.media_servers.get(media_server) {
                    match server.send(ContentServerCommand::InitFlooding) {
                        Ok(()) => info!(
                            "[ {} ]: sent a ContentServerCommand::InitFlooding to [ MediaContentServer {} ]",
                            "Simulation Controller".green(),
                            media_server
                        ),
                        Err(e) => error!(
                            "[ {} ]: failed to send a ContentServerCommand::InitFlooding to the [ MediaContentServer {} ]: {}",
                            "Simulation Controller".red(),
                            media_server,
                            e
                        ),
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
                if let Some((server, _)) = self.media_servers.get(media_server) {
                    if let Some(vec) = self.neighbor.get_mut(media_server) {
                        vec.push(node_id);
                        match server.send(ContentServerCommand::AddSender(node_id, sender)) {
                            Ok(()) => info!(
                                "[ {} ]: sent a ContentServerCommand::AddSender({}, sender_channel) to [ MediaContentServer {} ]",
                                "Simulation Controller".green(),
                                node_id,
                                media_server
                            ),
                            Err(e) => error!(
                                "[ {} ]: failed to send a ContentServerCommand::AddSender({}, sender_channel) to the [ MediaContentServer {} ]: {}",
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
            ContentServerCommand::RemoveSender(node_id) => {
                if let Some((server, _)) = self.media_servers.get(media_server) {
                    if let Some(vec) = self.neighbor.get_mut(media_server) {
                        if vec.len() > 2 {
                            vec.retain(|x| *x != node_id);
                            match server.send(ContentServerCommand::RemoveSender(node_id)) {
                                Ok(()) => {
                                    info!(
                                    "[ {} ]: sent a ContentServerCommand::RemoveSender({}) to [ MediaContentServer {} ]",
                                    "Simulation Controller".green(),
                                    node_id,
                                    media_server
                                );},
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
                                "[ {} ]: the [ MediaContentServer {} ] must be connected to at least two nodes",
                                "Simulation Controller".red(),
                                media_server
                            );
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
