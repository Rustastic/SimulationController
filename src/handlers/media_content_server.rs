use wg_2024::{network::NodeId, packet::PacketType};

use colored::Colorize;
use log::{error, info};

use messages::server_commands::{ContentServerCommand, ContentServerEvent};

use crate::SimulationController;

impl SimulationController {

    pub fn handle_media_event(&mut self, event: ContentServerEvent) {
        info!(
            "[ {} ] Is a {}",
            "Simulation Controller".yellow(),
            packet.pack_type
        );
        match event {
            ContentServerEvent::ServerStarted => {
                info!(
                    "[ {} ]: MediaContentServer started successfully",
                    "Simulation Controller".green(),
                )
            }
            ContentServerEvent::ServerStopped => {
                info!(
                    "[ {} ]: MediaContentServer stopped successfully",
                    "Simulation Controller".green(),
                )
            }
            ContentServerEvent::MessageForwarded(dest, msg) => {
                info!(
                    "[ {} ]: MediaContentServer forwarded the message {:?} to [ Client {} ]",
                    "Simulation Controller".green(),
                    msg,
                    dest
                )
            }
            ContentServerEvent::MessageReceived(src, msg) => info!(
                "[ {} ]: MediaContentServer received the message {:?} from [ Client {} ]",
                "Simulation Controller".green(),
                msg,
                src
            ),
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
            },
            ContentServerEvent::ErrorPacketCache(session_id, fragment_index) => {
                error!(
                    "[ {} ]: received an error message: Error in the packet cache [ session_id : {}, fragment_index: {} ]",
                    "Simulation Controller".red(),
                    session_id,
                    fragment_index
                );
            },
            ContentServerEvent::ControllerShortcut(packet) => {
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
            ContentServerEvent::UnreachableClient(_) => {
                error!("NOPE -> Not Implemented");
            }
        }
    }

    pub fn handle_media_command(&mut self, media_server: &NodeId, command: ContentServerCommand) {
        match command {
            ContentServerCommand::InitFlooding => {
                if let Some((server, _)) = self.media_servers.get(media_server) {
                    match server.send(ContentServerCommand::InitFlooding) {
                        Ok(()) => info!(
                            "[ {} ]: sent a ContentServerCommand::InitFlooding to [ MediaContentServer {} ]",
                            "Simulation Controller".green(),
                            media_server
                        ),
                        Err(e) => error!(
                            "[ {} ]: failed to send a ContentServerCommand::InitFlooding to the [ MediaContentServer {} ]: {}",
                            "Simulation Controller".red(),
                            media_server,
                            e
                        ),
                    }
                } else {
                    error!(
                        "[ {} ]: failed to find a Sender<ContentServerCommand> channel for the [ MediaContentServer {} ]",
                        "Simulation Controller".red(),
                        media_server
                    );
                }
            }
            ContentServerCommand::AddSender(node_id, sender) => {
                if let Some((server, _)) = self.media_servers.get(media_server) {
                    if let Some(vec) = self.neighbor.get_mut(media_server) {
                        vec.push(node_id);
                        match server.send(ContentServerCommand::AddSender(node_id, sender)) {
                            Ok(()) => info!(
                                "[ {} ]: sent a ContentServerCommand::AddSender({}, sender_channel) to [ MediaContentServer {} ]",
                                "Simulation Controller".green(),
                                node_id,
                                media_server
                            ),
                            Err(e) => error!(
                                "[ {} ]: failed to send a ContentServerCommand::AddSender({}, sender_channel) to the [ MediaContentServer {} ]: {}",
                                "Simulation Controller".red(),
                                node_id,
                                media_server,
                                e
                            ),
                        }
                    } else {
                        error!(
                            "[ {} ]: the [ MediaContentServer {} ] does not have any neighbor",
                            "Simulation Controller".red(),
                            media_server
                        );
                    }
                } else {
                    error!(
                        "[ {} ]: failed to find a Sender<ContentServerCommand> channel for the [ MediaContentServer {} ]",
                        "Simulation Controller".red(),
                        media_server
                    );
                }
            }
            ContentServerCommand::RemoveSender(node_id) => {
                if let Some((server, _)) = self.media_servers.get(media_server) {
                    if let Some(vec) = self.neighbor.get_mut(media_server) {
                        if vec.len() > 2 {
                            vec.retain(|x| *x != node_id);
                            match server.send(ContentServerCommand::RemoveSender(node_id)) {
                                Ok(()) => {
                                    info!(
                                    "[ {} ]: sent a ContentServerCommand::RemoveSender({}) to [ MediaContentServer {} ]",
                                    "Simulation Controller".green(),
                                    node_id,
                                    media_server
                                );},
                                Err(e) => error!(
                                    "[ {} ]: failed to send a ContentServerCommand::RemoveSender({}) to the [ MediaContentServer {} ]: {}",
                                    "Simulation Controller".red(),
                                    node_id,
                                    media_server,
                                    e
                                ),
                            }
                        } else {
                            error!(
                                "[ {} ]: the [ MediaContentServer {} ] must be connected to at least two nodes",
                                "Simulation Controller".red(),
                                media_server
                            );
                        }
                    } else {
                        error!(
                            "[ {} ]: the [ MediaContentServer {} ] does not have any neighbor",
                            "Simulation Controller".red(),
                            media_server
                        );
                    }
                } else {
                    error!(
                        "[ {} ]: failed to find a Sender<ContentServerCommand> channel for the [ MediaContentServer {} ]",
                        "Simulation Controller".red(),
                        media_server
                    );
                }
            }
        }
    }
}