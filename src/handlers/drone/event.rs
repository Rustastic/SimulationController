use crossbeam_channel::TryRecvError;
use messages::gui_commands::GUIEvents;
use wg_2024::{controller::DroneEvent, packet::PacketType};

use colored::Colorize;
use log::{error, info};

use crate::SimulationController;

impl SimulationController {
    pub fn handle_drone_event(&mut self) {
        match self.drone_recv.try_recv() {
            Ok(event) => self.process_drone_event(event),
            Err(TryRecvError::Empty) => (),
            Err(TryRecvError::Disconnected) => {
                error!(
                    "[ {} ]: DroneEvent receiver channel disconnected",
                    "Simulation Controller".red()
                );
            }
        }
    }

    #[allow(clippy::too_many_lines)]
    fn process_drone_event(&self, drone_event: DroneEvent) {
        match drone_event {
            DroneEvent::PacketSent(packet) => {
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
                        match self
                            .gui_send
                            .send(GUIEvents::PacketSent(*src, *dest, packet.clone()))
                        {
                            Ok(()) => {
                                info!(
                                    "[ {} ]: successfully sent a GUIEvents::PacketSent({}, {}, {:?}) from the Simulation Controller to the GUI",
                                    "Simulation Controller".green(),
                                    src,
                                    dest,
                                    packet
                                );
                            }
                            Err(e) => {
                                error!(
                                    "[ {} ]: failed to sent a GUIEvents::PacketSent({}, {}, {:?}) from the Simulation Controller to the GUI: {}",
                                    "Simulation Controller".green(),
                                    src,
                                    dest,
                                    packet,
                                    e
                                );
                            }
                        }
                    }
                }
            }

            DroneEvent::PacketDropped(packet) => {
                if let Some(src) = packet
                    .routing_header
                    .hops
                    .get(packet.routing_header.hop_index)
                {
                    match self
                        .gui_send
                        .send(GUIEvents::PacketDropped(*src, packet.clone()))
                    {
                        Ok(()) => {
                            info!(
                                "[ {} ]: successfully sent a GUIEvents::PacketDropped({}, {:?}) from the Simulation Controller to the GUI",
                                "Simulation Controller".green(),
                                src,
                                packet
                            );
                        }
                        Err(e) => {
                            error!(
                                "[ {} ]: failed to sent a GUIEvents::PacketDropped({}, {:?}) from the Simulation Controller to the GUI: {}",
                                "Simulation Controller".green(),
                                src,
                                packet,
                                e
                            );
                        }
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
                // Get destination of the packet
                if let Some(dest) = packet.routing_header.hops.last() {
                    // Get destination's channel
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
}
