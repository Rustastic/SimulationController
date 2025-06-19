use messages::gui_commands::GUIEvents;
use wg_2024::{controller::DroneEvent, packet::PacketType};

use colored::Colorize;
use log::error;

use crate::SimulationController;

impl SimulationController {
    #[allow(clippy::too_many_lines)]
    pub fn handle_drone_event(&mut self, drone_event: DroneEvent) {
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
                        let _ =
                            self.gui_send
                                .send(GUIEvents::PacketSent(*src, *dest, packet.clone()));

                        if let PacketType::Nack(nack) = packet.pack_type {
                            match nack.nack_type {
                                wg_2024::packet::NackType::Dropped => {}
                                _ => {
                                    self.global_flooding();
                                }
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
                    let _ = self
                        .gui_send
                        .send(GUIEvents::PacketDropped(*src, packet.clone()));
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
                    if let Some((_, chan)) = self.drones.get(dest) {
                        packet_channel = chan.clone();
                    } else if let Some((_, chan)) = self.cclients.get(dest) {
                        packet_channel = chan.clone();
                    } else if let Some((_, chan)) = self.mclients.get(dest) {
                        packet_channel = chan.clone();
                    } else if let Some((_, chan)) = self.comm_servers.get(dest) {
                        packet_channel = chan.clone();
                    } else if let Some((_, chan)) = self.text_servers.get(dest) {
                        packet_channel = chan.clone();
                    } else if let Some((_, chan)) = self.media_servers.get(dest) {
                        packet_channel = chan.clone();
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
                            error!("[ {} ] MsgFragment received in controller logic — this should not happen.", "Simulation Controller".red());
                        }
                        _ => {
                            let _ = packet_channel.send(packet.clone());
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
