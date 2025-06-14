use messages::gui_commands::GUIEvents;
use wg_2024::{
    controller::{DroneCommand, DroneEvent},
    network::NodeId,
    packet::PacketType,
};

use colored::Colorize;
use log::{error, info};

use crate::SimulationController;

impl SimulationController {
    // Handle Drone Events
    #[allow(clippy::too_many_lines)]
    pub fn handle_drone_event(&self, drone_event: DroneEvent) {
        match drone_event {
            DroneEvent::PacketSent(packet) => {
                /*info!(
                    "[ {} ] Is a {}",
                    "Simulation Controller".yellow(),
                    packet.pack_type
                );*/

                if let Some(src) = packet
                    .routing_header
                    .hops
                    .get(packet.routing_header.hop_index)
                {
                    if let Some(dest) = packet
                        .routing_header
                        .hops
                        .get(packet.routing_header.hop_index + 1)
                    {
                        match self.gui_send.send(GUIEvents::PacketSent(*src, *dest, packet.clone())) {
                            Ok(()) => info!(
                                "[ {} ]: successfully sent a GUIEvents::PacketSent({}, {}, {:?}) from the Simulation Controller to the GUI",
                                "Simulation Controller".green(),
                                src,
                                dest,
                                packet
                            ),
                            Err(e) => error!(
                                "[ {} ]: failed to sent a GUIEvents::PacketSent({}, {}, {:?}) from the Simulation Controller to the GUI: {}",
                                "Simulation Controller".green(),
                                src,
                                dest,
                                packet,
                                e
                            ),
                        }
                    }
                }
            }
            DroneEvent::PacketDropped(packet) => {
                info!(
                    "[ {} ] Is a {}",
                    "Simulation Controller".yellow(),
                    packet.pack_type
                );

                if let Some(src) = packet
                    .routing_header
                    .hops
                    .get(packet.routing_header.hop_index)
                {
                    match self.gui_send.send(GUIEvents::PacketDropped(*src, packet.clone())) {
                        Ok(()) => info!(
                            "[ {} ]: successfully sent a GUIEvents::PacketDropped({}, {:?}) from the Simulation Controller to the GUI",
                            "Simulation Controller".green(),
                            src,
                            packet
                        ),
                        Err(e) => error!(
                            "[ {} ]: failed to sent a GUIEvents::PacketDropped({}, {:?}) from the Simulation Controller to the GUI: {}",
                            "Simulation Controller".green(),
                            src,
                            packet,
                            e
                        ),
                    }
                } else {
                    error!(
                        "[ {} ]: failed to find a extract source from packet: {:?}",
                        "Simulation Controller".red(),
                        packet
                    );
                }
            }
            DroneEvent::ControllerShortcut(packet) => {
                info!(
                    "[ {} ] Is a {}",
                    "Simulation Controller".yellow(),
                    packet.pack_type
                );

                // Get packet destination node
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
    #[allow(clippy::too_many_lines)]
    pub fn handle_drone_command(&mut self, drone: &NodeId, drone_command: DroneCommand) {
        // Get drone channel
        if let Some((command_channel, _)) = self.drones.get(drone) {
            match drone_command {
                DroneCommand::RemoveSender(node_id) => {
                    if let Some(vec) = self.neighbor.get_mut(drone) {
                        vec.retain(|x| *x != node_id);
                        match command_channel.send(DroneCommand::RemoveSender(node_id)) {
                            Ok(()) => {
                                self.send_re_init_flooding();
                                info!(
                                    "[ {} ]: sent a DroneCommand::RemoveSender({}) to [ Drone {} ]",
                                    "Simulation Controller".green(),
                                    node_id,
                                    drone
                                );
                            },
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
                            Ok(()) => {
                                self.send_re_init_flooding();
                                info!(
                                    "[ {} ]: sent a DroneCommand::AddSender({}, sender_channel) to [ Drone {} ]",
                                    "Simulation Controller".green(),
                                    node_id,
                                    drone
                                );
                            },
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
                        #[allow(dropping_references)]
                        drop(command_send);
                        #[allow(dropping_references)]
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
}
