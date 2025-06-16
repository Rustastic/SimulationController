use crossbeam_channel::TryRecvError;
use wg_2024::packet::PacketType;

use colored::Colorize;
use log::{error, info};

use messages::server_commands::CommunicationServerEvent;

use crate::SimulationController;

impl SimulationController {
    #[allow(clippy::too_many_lines)]
    pub fn handle_communication_server_event(&mut self, event: CommunicationServerEvent) {
        match event {
            CommunicationServerEvent::ServerStarted => {
                info!(
                    "[ {} ]: CommunicationServer started successfully",
                    "Simulation Controller".green(),
                );
            }

            CommunicationServerEvent::ServerStopped => {
                info!(
                    "[ {} ]: CommunicationServer stopped successfully",
                    "Simulation Controller".green(),
                );
            }

            CommunicationServerEvent::ClientRegistered(client) => {
                info!(
                    "[ {} ]: CommunicationServer registered the [ Client {} ]",
                    "Simulation Controller".green(),
                    client,
                );
            }

            CommunicationServerEvent::ClientDeregistered(client) => {
                info!(
                    "[ {} ]: [ Client {} ]  deregistered from CommunicationServer",
                    "Simulation Controller".green(),
                    client,
                );
            }

            CommunicationServerEvent::MessageForwarded(dest, msg) => {
                info!(
                    "[ {} ]: CommunicationServer forwarded the message {:?} to [ Client {} ]",
                    "Simulation Controller".green(),
                    msg,
                    dest
                );
            }

            CommunicationServerEvent::MessageReceived(src, msg) => {
                info!(
                    "[ {} ]: CommunicationServer received the message {:?} from [ Client {} ]",
                    "Simulation Controller".green(),
                    msg,
                    src
                );
            }

            CommunicationServerEvent::UnreachableClient(client) => {
                error!(
                    "[ {} ]: received an error message: [ Client {} ] is unreachable",
                    "Simulation Controller".red(),
                    client,
                );
            }

            CommunicationServerEvent::UnreachableNode(client) => {
                error!(
                    "[ {} ]: received an error message: [ Node {} ] is unreachable",
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
                // Get destination of the packet
                if let Some(dest) = packet.routing_header.hops.last() {
                    // Get destination's packet channel
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

            CommunicationServerEvent::DestinationIsDrone(drone) => {
                error!(
                    "[ {} ]: received an error message: The selected destination is a drone [ Drone {} ]",
                    "Simulation Controller".red(),
                    drone
                );
            }

            CommunicationServerEvent::ErrorPacketCache(session_id, fragment_index) => {
                error!(
                    "[ {} ]: received an error message: Error in the packet cache [ session_id : {}, fragment_index: {} ]",
                    "Simulation Controller".red(),
                    session_id,
                    fragment_index
                );
            }
        }
    }
}
