use crossbeam_channel::{Receiver, Sender};
use log::{error, info, warn};
use std::{collections::HashMap, io::Write};

use colored::Colorize;

use wg_2024::{
    controller::{DroneCommand, DroneEvent},
    network::NodeId,
    packet::{Packet, PacketType},
};

use gui::{GUICommands, GUIEvents};

//use chat_client::ChatClient;

use crate::action;

#[derive(Clone)]
pub struct SimulationController {
    pub drones: HashMap<NodeId, (Sender<DroneCommand>, Sender<Packet>)>,
    receiver: Receiver<DroneEvent>,
    pub neighbor: HashMap<NodeId, Vec<NodeId>>,
    pub event_send: Sender<DroneEvent>,
    gui_send: Sender<GUIEvents>,
    gui_recv: Receiver<GUICommands>
}

impl SimulationController {
    pub fn new(
        drones: HashMap<NodeId, (Sender<DroneCommand>, Sender<Packet>)>,
        receiver: Receiver<DroneEvent>,
        neighbor: HashMap<NodeId, Vec<NodeId>>,
        event_send: Sender<DroneEvent>,
        gui_send: Sender<GUIEvents>,
        gui_recv: Receiver<GUICommands>
    ) -> Self {
        return Self {
            drones,
            receiver,
            neighbor,
            event_send,
            gui_send,
            gui_recv
        };
    }

    pub fn run(&mut self) {
        info!("[ {} ] Starting Simulation Controller", "Simulation Controller".green());
        // Start loop
        loop {

            // Check if any events are received
            match self.receiver.try_recv() {
                Ok(drone_event) => {
                    info!("[ {} ]: DroneEvent received", "Simulation Controller".green());
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

            // Check if any commands are received
            match self.gui_recv.try_recv() {
                Ok(gui_command) => {
                    info!("[ {} ]: GUICommand received", "Simulation Controller".green());
                    std::io::stdout().flush().unwrap();
                    self.handle_gui_command(gui_command);
                },
                Err(e) => match e {
                    crossbeam_channel::TryRecvError::Empty => warn!("[ {} ] Nothing", "Simulation Controller".yellow()),
                    crossbeam_channel::TryRecvError::Disconnected => error!(
                        "[ {} ]: GUICommands receiver channel disconnected: {}",
                        "Simulation Controller".red(),
                        e
                    ),
                },
            }
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
                match self.gui_send.send(GUIEvents::PacketSent(*src, *dest, gui_packet)) {
                    Ok(()) => info!(
                        "[ Simulation Controller ]: sent a GUIEvent: PacketSent({}, {}) to GUI",
                        src,
                        dest
                    ),
                    Err(e) => error!(
                        "[ {} ]: failed to send GUIEvent: PacketSent({}, {}) to GUI: {}",
                        "Simulation Controller".red(),
                        src,
                        dest,
                        e
                    ),
                }

                info!(
                    "[ Drone: {} ]: Sent a Packet: {} to Drone {}",
                    src,
                    packet_type,
                    dest
                );
            },
            DroneEvent::PacketDropped(packet) => {
                let gui_packet = packet.clone();

                let drone = packet
                    .routing_header
                    .hops
                    .get(packet.routing_header.hop_index)
                    .unwrap();

                let session_id = packet.session_id;

                // GUI
                match self.gui_send.send(GUIEvents::PacketDropped(*drone, gui_packet)) {
                    Ok(()) => info!(
                        "[ Simulation Controller ]: sent a GUIEvent: PacketDropped({}) sent to GUI",
                        drone
                    ),
                    Err(e) => error!(
                        "[ {} ]: failed to send GUIEvent: PacketDropped({}) sent to GUI: {}",
                        "Simulation Controller".red(),
                        drone,
                        e
                    ),
                }

                info!(
                    "[ Drone: {} ]: Dropped the packet with session_id: {}",
                    drone,
                    session_id
                );
            },
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
                        "[ {} ]: failed to find a Drone to send the DroneEvent: ControllerShortcut",
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
                                "[ Simulation Controller ]: sent a DroneCommand: RemoveSender({}) to [ Drone {} ]",
                                node_id,
                                drone
                            ),
                            Err(e) => error!(
                                "[ {} ]: failed to send a DroneCommand: RemoveSender({}) to the [ Drone {} ]: {}",
                                "Simulation Controller".red(),
                                node_id,
                                drone,
                                e
                            ),
                        }

                        action::remove_sender(self, *drone, node_id);

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
                                "[ Simulation Controller ]: sent a DroneCommand: AddSender({}, sender_channel) to [ Drone {} ]",
                                node_id,
                                drone
                            ),
                            Err(e) => error!(
                                "[ {} ]: failed to send a DroneCommand: AddSender({}, sender_channel) to the [ Drone {} ]: {}",
                                "Simulation Controller".red(),
                                node_id,
                                drone,
                                e
                            ),
                        }

                        action::add_sender(self, *drone, node_id);

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
                            "[ Simulation Controller ]: sent a DroneCommand: SetPacketDropRate({}) to [ Drone {} ]",
                            pdr,
                            drone
                        ),
                        Err(e) => error!(
                            "[ {} ]: failed to send a DroneCommand: SetPacketDropRate({}) to the [ Drone {} ]: {}",
                            "Simulation Controller".red(),
                            pdr,
                            drone,
                            e
                        ),
                    }
                }
                DroneCommand::Crash => {
                    action::crash(self, *drone);

                    if let Some((command_send, packet_send)) = self.drones.get(drone) {
                        let _ = drop(command_send);
                        let _ = drop(packet_send);
                    }

                    let drone_entry = self.drones.remove(drone);

                    self.neighbor.remove(drone);

                    if let Some((command_channel, _)) = drone_entry {
                        match command_channel.send(DroneCommand::Crash) {
                            Ok(()) => info!(
                                "[ Simulation Controller ]: sent a DroneCommand: Crash() to [ Drone {} ]",
                                drone
                            ),
                            Err(e) => error!(
                                "[ {} ]: failed to send a DroneCommand: Crash() to the [ Drone {} ]: {}",
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
            GUICommands::Spawn(id, connected_drone_ids, pdr) => return,
            GUICommands::Crash(drone) => self.handle_command(&drone, DroneCommand::Crash),
            GUICommands::RemoveSender(drone, neighbor) => self.handle_command(&drone, DroneCommand::RemoveSender(neighbor)),
            GUICommands::AddSender(drone, neighbor) => return,
            GUICommands::SetPDR(drone, pdr) => self.handle_command(&drone, DroneCommand::SetPacketDropRate(pdr)),
        }
    }
}
