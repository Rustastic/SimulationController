use wg_2024::packet::PacketType;

use colored::Colorize;
use log::{error, info};

use messages::server_commands::ContentServerEvent;

use crate::SimulationController;

impl SimulationController {
    #[allow(clippy::too_many_lines)]
    pub fn handle_media_server_event(&mut self, event: ContentServerEvent) {
        match event {
            ContentServerEvent::ServerStarted => {
                info!(
                    "[ {} ]: MediaContentServer started successfully",
                    "Simulation Controller".green(),
                );
            }

            ContentServerEvent::ServerStopped => {
                info!(
                    "[ {} ]: MediaContentServer stopped successfully",
                    "Simulation Controller".green(),
                );
            }

            ContentServerEvent::MessageForwarded(dest, msg) => {
                info!(
                    "[ {} ]: MediaContentServer forwarded the message {:?} to [ Client {} ]",
                    "Simulation Controller".green(),
                    msg,
                    dest
                );
            }

            ContentServerEvent::MessageReceived(src, msg) => {
                info!(
                    "[ {} ]: MediaContentServer received the message {:?} from [ Client {} ]",
                    "Simulation Controller".green(),
                    msg,
                    src
                );
            }

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
            }

            ContentServerEvent::ErrorPacketCache(session_id, fragment_index) => {
                error!(
                    "[ {} ]: received an error message: Error in the packet cache [ session_id : {}, fragment_index: {} ]",
                    "Simulation Controller".red(),
                    session_id,
                    fragment_index
                );
            }

            ContentServerEvent::ControllerShortcut(packet) => {
                if let Some(dest) = packet.routing_header.hops.last() {
                    // Get destination node channel
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
                            "[ {} ]: failed to find a Sender<Packet> channel for the [ CommunicationServer {} ]",
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
                        "[ {} ]: failed to find a CommunicationServer to send the CommunicationServerCommand::ControllerShortcut",
                        "Simulation Controller".red()
                    );
                }
            }

            ContentServerEvent::UnreachableNode(client) => {
                error!(
                    "[ {} ]: received an error message: [ Node {} ] is unreachable",
                    "Simulation Controller".red(),
                    client,
                );
            }

            ContentServerEvent::UnreachableClient(client) => {
                error!(
                    "[ {} ]: received an error message: [ Client {} ] is unreachable",
                    "Simulation Controller".red(),
                    client,
                );
            }
        }
    }
}
