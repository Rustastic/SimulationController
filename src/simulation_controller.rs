use crossbeam_channel::{Receiver, Sender};
use log::{error, info, warn};
use std::{collections::HashMap, thread};

use colored::Colorize;

use wg_2024::{
    controller::{DroneCommand, DroneEvent},
    drone::Drone,
    network::NodeId,
    packet::{Packet, PacketType},
};

use gui::commands::{GUICommands, GUIEvents};
use chat_client::ChatClient;
use messages::client_commands::{ChatClientCommand, ChatClientEvent, MediaClientCommand, MediaClientEvent};

use crate::{action, verify};

pub struct SimulationController {
    pub drones: HashMap<NodeId, (Sender<DroneCommand>, Sender<Packet>)>,
    receiver: Receiver<DroneEvent>,
    pub neighbor: HashMap<NodeId, Vec<NodeId>>,
    pub event_send: Sender<DroneEvent>,
    pub new_drones: Vec<Box<dyn Drone>>,
    gui_send: Sender<GUIEvents>,
    gui_recv: Receiver<GUICommands>,
    cclient_send: HashMap<NodeId, Sender<ChatClientCommand>>,
    cclient_recv: Receiver<ChatClientEvent>,
    mclient_send: HashMap<NodeId, Sender<MediaClientCommand>>,
    mclient_recv: Receiver<MediaClientEvent>
}

impl SimulationController {
    pub fn new(
        drones: HashMap<NodeId, (Sender<DroneCommand>, Sender<Packet>)>,
        receiver: Receiver<DroneEvent>,
        neighbor: HashMap<NodeId, Vec<NodeId>>,
        event_send: Sender<DroneEvent>,
        gui_send: Sender<GUIEvents>,
        gui_recv: Receiver<GUICommands>,
        cclient_send: HashMap<NodeId, Sender<ChatClientCommand>>,
        cclient_recv: Receiver<ChatClientEvent>,
        mclient_send: HashMap<NodeId, Sender<MediaClientCommand>>,
        mclient_recv: Receiver<MediaClientEvent>
    ) -> Self {
        return Self {
            drones,
            receiver,
            neighbor,
            event_send,
            new_drones: Vec::new(),
            gui_send,
            gui_recv,
            cclient_send,
            cclient_recv,
            mclient_send,
            mclient_recv,
        };
    }

    pub fn run(&mut self) {
        info!(
            "[ {} ] Starting Simulation Controller",
            "Simulation Controller".green()
        );
        // Start loop
        loop {
            // Check if any Drone events are received
            match self.receiver.try_recv() {
                Ok(drone_event) => {
                    info!(
                        "[ {} ]: DroneEvent received",
                        "Simulation Controller".green()
                    );
                    self.handle_event(drone_event);
                }
                Err(e) => match e {
                    crossbeam_channel::TryRecvError::Empty => (),
                    crossbeam_channel::TryRecvError::Disconnected => error!(
                        "[ {} ]: DroneEvent receiver channel disconnected: {}",
                        "Simulation Controller".red(),
                        e
                    ),
                },
            }

            // Check if any ChatClient events are received
            match self.cclient_recv.try_recv() {
                Ok(cclient_command) => {
                    info!(
                        "[ {} ]: ChatClientEvent received",
                        "Simulation Controller".green()
                    );
                    self.handle_cclient_event(cclient_command);
                }
                Err(e) => match e {
                    crossbeam_channel::TryRecvError::Empty => (),
                    crossbeam_channel::TryRecvError::Disconnected => error!(
                        "[ {} ]: ChatClientEvent receiver channel disconnected: {}",
                        "Simulation Controller".red(),
                        e
                    ),
                }
            }

            // Check if any MediaClient events are received
            match self.mclient_recv.try_recv() {
                Ok(mclient_command) => {
                    info!(
                        "[ {} ]: MediaClientEvent received",
                        "Simulation Controller".green()
                    );
                    self.handle_mclient_event(mclient_command);
                }
                Err(e) => match e {
                    crossbeam_channel::TryRecvError::Empty => (),
                    crossbeam_channel::TryRecvError::Disconnected => error!(
                        "[ {} ]: MediaClientEvent receiver channel disconnected: {}",
                        "Simulation Controller".red(),
                        e
                    ),
                }
            }

            // Check if any GUI commands are received
            match self.gui_recv.try_recv() {
                Ok(gui_command) => {
                    info!(
                        "[ {} ]: GUICommand received",
                        "Simulation Controller".green()
                    );
                    self.handle_gui_command(gui_command);
                }
                Err(e) => match e {
                    crossbeam_channel::TryRecvError::Empty => (),
                    crossbeam_channel::TryRecvError::Disconnected => error!(
                        "[ {} ]: GUICommands receiver channel disconnected: {}",
                        "Simulation Controller".red(),
                        e
                    ),
                },
            }

            //////////////////////////////////////////////////////////// REMOVE
            thread::sleep(std::time::Duration::from_secs(1));
        }
    }

    pub fn handle_event(&self, drone_event: DroneEvent) {
        match drone_event {
            DroneEvent::PacketSent(packet) => {
                let gui_packet = packet.clone();

                let src = packet
                    .routing_header
                    .hops
                    .get(packet.routing_header.hop_index)
                    .unwrap();

                let dest = packet
                    .routing_header
                    .hops
                    .get(packet.routing_header.hop_index + 1)
                    .unwrap();

                let packet_type = packet.clone().pack_type;

                // GUI
                match self
                    .gui_send
                    .send(GUIEvents::PacketSent(*src, *dest, gui_packet))
                {
                    Ok(()) => info!(
                        "[ {} ]: sent a GUIEvent::PacketSent({}, {}) to GUI",
                        "Simulation Controller".green(),
                        src,
                        dest
                    ),
                    Err(e) => error!(
                        "[ {} ]: failed to send GUIEvent::PacketSent({}, {}) to GUI: {}",
                        "Simulation Controller".red(),
                        src,
                        dest,
                        e
                    ),
                }

                info!(
                    "[ Drone: {} ]: Sent a Packet: {} to Drone {}",
                    src, packet_type, dest
                );
            }
            DroneEvent::PacketDropped(packet) => {
                let gui_packet = packet.clone();

                let drone = packet
                    .routing_header
                    .hops
                    .get(packet.routing_header.hop_index)
                    .unwrap();

                let session_id = packet.session_id;

                // GUI
                match self
                    .gui_send
                    .send(GUIEvents::PacketDropped(*drone, gui_packet))
                {
                    Ok(()) => info!(
                        "[ {} ]: sent a GUIEvent::PacketDropped({}) sent to GUI",
                        "Simulation Controller".green(),
                        drone
                    ),
                    Err(e) => error!(
                        "[ {} ]: failed to send GUIEvent::PacketDropped({}) sent to GUI: {}",
                        "Simulation Controller".red(),
                        drone,
                        e
                    ),
                }

                info!(
                    "[ Drone: {} ]: Dropped the packet with session_id: {}",
                    drone, session_id
                );
            }
            DroneEvent::ControllerShortcut(packet) => {
                // Get packet destination node
                if let Some(dest) = packet
                    .routing_header
                    .hops
                    .get(packet.routing_header.len() - 1)
                {
                    // Get destination node channel
                    if let Some((_, packet_channel)) = self.drones.get(dest) {
                        // Send Packet t destination
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
                            "[ {} ]: failed to find a Sender<Packet> channel for the [ Drone {} ]",
                            "Simulation Controller".red(),
                            dest
                        );
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

    pub fn handle_command(&mut self, drone: &NodeId, drone_command: DroneCommand) {
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
                        drop(command_send);
                        drop(packet_send);
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
                Ok(()) => self.handle_command(&drone, DroneCommand::Crash),
                Err(e) => error!("{}", e),
            },
            GUICommands::RemoveSender(drone, to_remove) => {
                match action::remove_sender(self, &drone, &to_remove) {
                    Ok(()) => self.handle_command(&drone, DroneCommand::RemoveSender(to_remove)),
                    Err(e) => error!("{}", e),
                }
            }
            GUICommands::AddSender(drone, to_add) => {
                match action::add_sender(self, &drone, &to_add) {
                    Ok(()) => {
                        let (_, sender) = self.drones.get(&to_add).unwrap().clone();
                        self.handle_command(&drone, DroneCommand::AddSender(to_add, sender));
                    }
                    Err(e) => error!("{}", e),
                }
            }
            GUICommands::SetPDR(drone, pdr) => match verify::valid_pdr(pdr) {
                Ok(value) => self.handle_command(&drone, DroneCommand::SetPacketDropRate(value)),
                Err(e) => error!("{}", e),
            },
        }
    }

    fn handle_cclient_event(&mut self, event: ChatClientEvent) {
        match event {
            ChatClientEvent::CommunicationServerList(items) => (),
            ChatClientEvent::MessageReceived(src, msg) => {
                match self.gui_send.send(GUIEvents::MessageReceived(src, src, msg.clone())) {
                    Ok(()) => info!(
                        "[ {} ]: sent a GUIEvent::PacketReceived({}, {}, {}) to GUI",
                        "Simulation Controller".green(),
                        src,
                        src,
                        msg,
                    ),
                    Err(e) => error!(
                        "[ {} ]: failed to send GUIEvent::PacketReceived({}, {}, {}) to GUI: {}",
                        "Simulation Controller".red(),
                        src,
                        src,
                        msg,
                        e
                    ),
                }

                info!(
                    "[ Client: {} ]: received the message {:?} from [ Server {} ]",
                    src,
                    msg,
                    src
                );
            },
            ChatClientEvent::SuccessfulRegistration(_) => (),
            ChatClientEvent::ClientList(items) => (),
            ChatClientEvent::SuccessfulLogOut => (),
            ChatClientEvent::UnreachableClient(client) => {
                match self.gui_send.send(GUIEvents::UnreachableClient(client)) {
                    Ok(()) => info!(
                        "[ {} ]: sent a GUIEvent::UnreachableClient({}) to GUI",
                        "Simulation Controller".green(),
                        client
                    ),
                    Err(e) => error!(
                        "[ {} ]: failed to send GUIEvent::UnreachableClient({}) to GUI: {}",
                        "Simulation Controller".red(),
                        client,
                        e
                    ),
                }

                error!(
                    "[ {} ]: received an error message: [ Client {} ] is unreachable",
                    "Simulation Controller".red(),
                    client,
                );
            },
            ChatClientEvent::ErrorNotRunning =>  {
                match self.gui_send.send(GUIEvents::ErrorNotRunning) {
                    Ok(()) => info!(
                        "[ {} ]: sent a GUIEvent::ErrorNotRunning to GUI",
                        "Simulation Controller".green(),
                    ),
                    Err(e) => error!(
                        "[ {} ]: failed to send GUIEvent::ErrorNotRunning to GUI: {}",
                        "Simulation Controller".red(),
                        e
                    ),
                }

                error!(
                    "[ {} ]: received an error message: The Client is not running",
                    "Simulation Controller".red(),
                );
            },
            ChatClientEvent::ErrorNotRegistered => (),
            ChatClientEvent::ControllerShortcut(packet) => {
                if let Some(dest) = packet
                    .routing_header
                    .hops
                    .get(packet.routing_header.len() - 1)
                {
                    // Get destination node channel
                    if let Some((_, packet_channel)) = self.drones.get(dest) {
                        // Send Packet t destination
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
                            "[ {} ]: failed to find a Sender<Packet> channel for the [ Drone {} ]",
                            "Simulation Controller".red(),
                            dest
                        );
                    }
                } else {
                    error!(
                        "[ {} ]: failed to find a Drone to send the ChatClientCommand::ControllerShortcut",
                        "Simulation Controller".red()
                    );
                }
            },
        }
    }

    fn handle_cclient_command(&mut self, chat_client: &NodeId, command: ChatClientCommand) {
        match command {
            ChatClientCommand::InitFlooding => {
                if let Some(client) = self.cclient_send.get(chat_client) {
                    match client.send(ChatClientCommand::InitFlooding) {
                        Ok(()) => info!(
                            "[ {} ]: sent a ChatClientCommand::InitFlo0ding to [ Client {} ]",
                            "Simulation Controller".green(),
                            chat_client
                        ),
                        Err(e) => error!(
                            "[ {} ]: failed to send a ChatClientCommand::InitFlooding to the [ Client {} ]: {}",
                            "Simulation Controller".red(),
                            chat_client,
                            e
                        ),
                    }
                } else {
                    error!(
                        "[ {} ]: failed to find a Sender<ChatClientCommand> channel for the [ Client {} ]",
                        "Simulation Controller".red(),
                        chat_client
                    );
                }
            },
            ChatClientCommand::StartChatClient => {
                if let Some(client) = self.cclient_send.get(chat_client) {
                    match client.send(ChatClientCommand::StartChatClient) {
                        Ok(()) => info!(
                            "[ {} ]: sent a ChatClientCommand::StartChatClient to [ Client {} ]",
                            "Simulation Controller".green(),
                            chat_client
                        ),
                        Err(e) => error!(
                            "[ {} ]: failed to send a ChatClientCommand::StartChatClient to the [ Client {} ]: {}",
                            "Simulation Controller".red(),
                            chat_client,
                            e
                        ),
                    }
                } else {
                    error!(
                        "[ {} ]: failed to find a Sender<ChatClientCommand> channel for the [ Client {} ]",
                        "Simulation Controller".red(),
                        chat_client
                    );
                }
            },
            ChatClientCommand::RemoveSender(drone) => {
                if let Some(neighbors) = self.neighbor.get(chat_client) {
                    if neighbors.len() == 2 {
                        if let Some(client) = self.cclient_send.get(chat_client) {
                            match client.send(ChatClientCommand::RemoveSender(drone)) {
                                Ok(()) => info!(
                                    "[ {} ]: sent a ChatClientCommand::RemoveSender({}) to [ Client {} ]",
                                    "Simulation Controller".green(),
                                    drone,
                                    chat_client
                                ),
                                Err(e) => error!(
                                    "[ {} ]: failed to send a ChatClientCommand::RemoveSender({}) to the [ Client {} ]: {}",
                                    "Simulation Controller".red(),
                                    drone,
                                    chat_client,
                                    e
                                ),
                            }
                        } else {
                            error!(
                                "[ {} ]: failed to find a Sender<ChatClientCommand> channel for the [ Client {} ]",
                                "Simulation Controller".red(),
                                chat_client
                            );
                        }
                    } else {
                        error!(
                            "[ {} ]: failed to send a ChatClientCommand::RemoveSender({}) to the [ Client {} ]: {}",
                            "Simulation Controller".red(),
                            drone,
                            chat_client,
                            "Each client must remain connected to at least one and at most two drones"
                        );
                    }
                } else {
                    error!(
                        "[ {} ]: the [ Drone {} ] does not have any neighbor",
                        "Simulation Controller".red(),
                        drone
                    );
                }
            },
            ChatClientCommand::AddSender(drone, sender) => {
                if !self.cclient_send.contains_key(&drone) {
                    if let Some(neighbors) = self.neighbor.get(chat_client) {
                        if neighbors.len() == 2 {
                            if let Some(client) = self.cclient_send.get(chat_client) {
                                match client.send(ChatClientCommand::AddSender(drone, sender.clone())) {
                                    Ok(()) => info!(
                                        "[ {} ]: sent a ChatClientCommand::AddSender({}, {:?}) to [ Client {} ]",
                                        "Simulation Controller".green(),
                                        drone,
                                        sender,
                                        chat_client
                                    ),
                                    Err(e) => error!(
                                        "[ {} ]: failed to send a ChatClientCommand::AddSender({}, {:?}) to the [ Client {} ]: {}",
                                        "Simulation Controller".red(),
                                        drone,
                                        sender,
                                        chat_client,
                                        e
                                    ),
                                }
                            } else {
                                error!(
                                    "[ {} ]: failed to find a Sender<ChatClientCommand> channel for the [ Client {} ]",
                                    "Simulation Controller".red(),
                                    chat_client
                                );
                            }
                        } else {
                            error!(
                                "[ {} ]: failed to send a ChatClientCommand::RemoveSender({}) to the [ Client {} ]: {}",
                                "Simulation Controller".red(),
                                drone,
                                chat_client,
                                "Each client must remain connected to at least one and at most two drones"
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
            },
            ChatClientCommand::SendMessageTo(dest, msg) => {
                if let Some(client) = self.cclient_send.get(chat_client) {
                    match client.send(ChatClientCommand::SendMessageTo(dest, msg.clone())) {
                        Ok(()) => info!(
                            "[ {} ]: sent a ChatClientCommand::SendMessageTo({}, {}) to [ Client {} ]",
                            "Simulation Controller".green(),
                            dest,
                            msg,
                            chat_client
                        ),
                        Err(e) => error!(
                            "[ {} ]: failed to send a ChatClientCommand::SendMessageTo({}, {}) to the [ Client {} ]: {}",
                            "Simulation Controller".red(),
                            dest,
                            msg,
                            chat_client,
                            e
                        ),
                    }
                } else {
                    error!(
                        "[ {} ]: failed to find a Sender<ChatClientCommand> channel for the [ Client {} ]",
                        "Simulation Controller".red(),
                        chat_client
                    );
                }
            },
            ChatClientCommand::RegisterTo(server) => (),
            ChatClientCommand::GetClientList => (),
            ChatClientCommand::LogOut => (),
        }
    }

    fn handle_mclient_event(&mut self, event: MediaClientEvent) {
        match event {
            MediaClientEvent::ReceveidFloodResponse => todo!(),
            MediaClientEvent::RemovedSender(_) => todo!(),
            MediaClientEvent::AddedSender(_) => todo!(),
            MediaClientEvent::UnreachableNode(_) => todo!(),
            MediaClientEvent::SendError(send_error) => todo!(),
            MediaClientEvent::ReceveidServerType(server_type) => todo!(),
            MediaClientEvent::ReceveidFileList(items) => todo!(),
            MediaClientEvent::ReceveidFile(_, buf_reader) => todo!(),
            MediaClientEvent::ReceivedMedia(_, buf_reader) => todo!(),
        }
    }

    fn handle_mclient_event(&mut self, client: MediaClient, command: MediaClientCommand) {
        match command {
            MediaClientCommand::InitFlooding => todo!(),
            MediaClientCommand::RemoveSender(_) => todo!(),
            MediaClientCommand::AddSender(_, sender) => todo!(),
            MediaClientCommand::AskServerType(_) => todo!(),
            MediaClientCommand::AskFilesList(_) => todo!(),
            MediaClientCommand::AskForFile(_, _) => todo!(),
            MediaClientCommand::AskForMedia(_, _) => todo!(),
        }
    }
}
