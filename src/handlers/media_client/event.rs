use colored::Colorize;
use log::{error, info};

use wg_2024::packet::PacketType;

use messages::{client_commands::MediaClientEvent, gui_commands::GUIEvents};

use crate::SimulationController;

impl SimulationController {
    #[allow(clippy::too_many_lines)]
    pub fn handle_media_client_event(&mut self, event: MediaClientEvent) {
        match event {
            MediaClientEvent::ReceveidFloodResponse => {
                info!(
                    "[ {} ]: The media client retrieved a FloodResponse",
                    "Simulation Controller".green()
                );
            }

            MediaClientEvent::RemovedSender(drone) => {
                info!(
                    "[ {} ]: The media client removed the neighbor [ Drone {} ]",
                    "Simulation Controller".green(),
                    drone
                );
            }

            MediaClientEvent::AddedSender(drone) => {
                info!(
                    "[ {} ]: The media client added the neighbor [ Drone {} ]",
                    "Simulation Controller".green(),
                    drone
                );
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

            MediaClientEvent::ReceveidFileList(server, dest, items) => {
                info!(
                    "[ {} ]: received the file list of [ TextServer {} ]",
                    "Simulation Controller".green(),
                    server,
                );

                match self.gui_send.send(GUIEvents::FileList(server, dest, items.clone())) {
                    Ok(()) => info!(
                        "[ {} ]: successfully sent a GUIEvents::FileList({}, {} {:?}) from the Simulation Controller to the GUI",
                        "Simulation Controller".green(),
                        server,
                        dest,
                        items
                    ),
                    Err(e) => error!(
                        "[ {} ]: failed to sent a GUIEvents::FileList({}, {}, {:?}) from the Simulation Controller to the GUI: {}",
                        "Simulation Controller".green(),
                        server,
                        dest,
                        items,
                        e
                    ),
                }
            }

            MediaClientEvent::ReceveidFile(node_id, _, _) => {
                info!(
                    "[ {} ]: [ MediaClient {} ] received a file",
                    "Simulation Controller".green(),
                    node_id,
                );
            }

            MediaClientEvent::ControllerShortcut(packet) => {
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
                            panic!("Impossible how the hell did u do this");
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
            MediaClientEvent::ServerList(_items) => {
                ()
            },
            MediaClientEvent::ReceveidServerType(_node_id, _server_type) => {
                ()
            }
        }
    }
}
