use crossbeam_channel::TryRecvError;
use wg_2024::packet::PacketType;

use colored::Colorize;
use log::{error, info};

use messages::server_commands::ContentServerEvent;

use crate::SimulationController;

impl SimulationController {
    pub fn handle_text_server_event(&mut self) {
        match self.media_recv.try_recv() {
            Ok(event) => self.process_text_server_event(event),
            Err(TryRecvError::Empty) => (),
            Err(TryRecvError::Disconnected) => {
                error!(
                    "[ {} ]: MediaServerEvent receiver channel disconnected",
                    "Simulation Controller".red()
                );
            }
        }
    }

    #[allow(clippy::too_many_lines)]
    fn process_text_server_event(&mut self, event: ContentServerEvent) {
        info!("[ {} ] Is a {:?}", "Simulation Controller".yellow(), event);
        match event {
            ContentServerEvent::ServerStarted => {
                info!(
                    "[ {} ]: TextContentServer started successfully",
                    "Simulation Controller".green(),
                );
            }

            ContentServerEvent::ServerStopped => {
                info!(
                    "[ {} ]: TextContentServer stopped successfully",
                    "Simulation Controller".green(),
                );
            }

            ContentServerEvent::MessageForwarded(dest, msg) => {
                info!(
                    "[ {} ]: TextContentServer forwarded the message {:?} to [ Client {} ]",
                    "Simulation Controller".green(),
                    msg,
                    dest
                );
            }

            ContentServerEvent::MessageReceived(src, msg) => {
                info!(
                    "[ {} ]: TextContentServer received the message {:?} from [ Client {} ]",
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

            ContentServerEvent::UnreachableNode(node) => {
                error!(
                    "[ {} ]: received an error message: [ Node {} ] is unreachable",
                    "Simulation Controller".red(),
                    node
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
                            panic!("Impossible how the hell did u do this");
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
