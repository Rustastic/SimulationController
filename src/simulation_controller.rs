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
    server_commands::{CommunicationServerCommand, CommunicationServerEvent},
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
            thread::sleep(std::time::Duration::from_secs(2));
            self.handle_cclient_command(chat_client, ChatClientCommand::StartChatClient);

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
                recv(self.mclient_recv) -> mclient_command => match mclient_command { // Uncommented if needed
                    Ok(mclient_command) => {
                        info!("[ {} ]: MediaClientEvent received", "Simulation Controller".green());
                        self.handle_mclient_event(mclient_command);
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
                info!(
                    "[ {} ] Is a {}",
                    "Simulation Controller".yellow(),
                    packet.pack_type
                );
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
            }
            GUICommands::RegisterTo(client, server) => {
                self.handle_cclient_command(&client, ChatClientCommand::RegisterTo(server))
            }
            GUICommands::LogOut(client, server) => {
                self.handle_cclient_command(&client, ChatClientCommand::LogOut)
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
            MediaClientEvent::ReceveidFile(node_id, file_id, file_response) => {
                info!(
                    "[ {} ]: received a file from [ MediaClient {} ]",
                    "Simulation Controller".green(),
                    node_id,
                );
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
            _ => (),
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
            _ => (),
        }
    }

    // Handle Server Command
    fn handle_commserver_event(&mut self, event: CommunicationServerEvent) {
        match event {
            CommunicationServerEvent::ServerStarted => {
                info!(
                    "[ {} ]: Server started successfully",
                    "Simulation Controller".green(),
                )
            }
            CommunicationServerEvent::ServerStopped => {
                info!(
                    "[ {} ]: Server stopped successfully",
                    "Simulation Controller".green(),
                )
            }
            CommunicationServerEvent::ClientRegistered(client) => {
                info!(
                    "[ {} ]: Server registered [ Client {} ]",
                    "Simulation Controller".green(),
                    client,
                )
            }
            CommunicationServerEvent::ClientDeregistered(client) => {
                info!(
                    "[ {} ]: Server deregistered [ Client {} ]",
                    "Simulation Controller".green(),
                    client,
                )
            }
            CommunicationServerEvent::MessageForwarded(dest, msg) => {
                info!(
                    "[ {} ]: Server forwarded the message {:?} to [ Client {} ]",
                    "Simulation Controller".green(),
                    msg,
                    dest
                )
            }
            CommunicationServerEvent::MessageReceived(src, msg) => info!(
                "[ {} ]: Server received the message {:?} from [ Client {} ]",
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
            CommunicationServerEvent::DestinationIsDrone(drone) => (),
            CommunicationServerEvent::ErrorPacketCache(_, _) => (),
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
                            "[ {} ]: sent a CommunicationServerCommand::InitFlooding to [ Server {} ]",
                            "Simulation Controller".green(),
                            comm_server
                        ),
                        Err(e) => error!(
                            "[ {} ]: failed to send a CommunicationServerCommand::InitFlooding to the [ Server {} ]: {}",
                            "Simulation Controller".red(),
                            comm_server,
                            e
                        ),
                    }
                } else {
                    error!(
                        "[ {} ]: failed to find a Sender<CommunicationServerCommand> channel for the [ Server {} ]",
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
                                "[ {} ]: sent a CommunicationServerCommand::AddSender({}, sender_channel) to [ Server {} ]",
                                "Simulation Controller".green(),
                                node_id,
                                comm_server
                            ),
                            Err(e) => error!(
                                "[ {} ]: failed to send a CommunicationServerCommand::AddSender({}, sender_channel) to the [ Server {} ]: {}",
                                "Simulation Controller".red(),
                                node_id,
                                comm_server,
                                e
                            ),
                        }
                    } else {
                        error!(
                            "[ {} ]: the [ Server {} ] does not have any neighbor",
                            "Simulation Controller".red(),
                            comm_server
                        );
                    }
                } else {
                    error!(
                        "[ {} ]: failed to find a Sender<CommunicationServerCommand> channel for the [ Server {} ]",
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
                                    "[ {} ]: sent a CommunicationServerCommand::RemoveSender({}) to [ Server {} ]",
                                    "Simulation Controller".green(),
                                    node_id,
                                    comm_server
                                );},
                                Err(e) => error!(
                                    "[ {} ]: failed to send a CommunicationServerCommand::RemoveSender({}) to the [ Server {} ]: {}",
                                    "Simulation Controller".red(),
                                    node_id,
                                    comm_server,
                                    e
                                ),
                            }
                        } else {
                            error!(
                                "[ {} ]: the [ Server {} ] must be connected to at least two nodes",
                                "Simulation Controller".red(),
                                comm_server
                            );
                        }
                    } else {
                        error!(
                            "[ {} ]: the [ Server {} ] does not have any neighbor",
                            "Simulation Controller".red(),
                            comm_server
                        );
                    }
                } else {
                    error!(
                        "[ {} ]: failed to find a Sender<CommunicationServerCommand> channel for the [ Server {} ]",
                        "Simulation Controller".red(),
                        comm_server
                    );
                }
            }
        }
    }
}
