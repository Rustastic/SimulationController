use wg_2024::{network::NodeId, packet::PacketType};
use colored::Colorize;
use log::{error, info};

use messages::server_commands::{CommunicationServerCommand, CommunicationServerEvent};

use crate::SimulationController;

impl SimulationController {
    // Handle Server Command
    pub fn handle_commserver_event(&mut self, event: CommunicationServerEvent) {
        info!(
            "[ {} ] Is a {:?}",
            "Simulation Controller".yellow(),
            event
        );
        match event {
            CommunicationServerEvent::ServerStarted => {
                info!(
                    "[ {} ]: CommunicationServer started successfully",
                    "Simulation Controller".green(),
                )
            }
            CommunicationServerEvent::ServerStopped => {
                info!(
                    "[ {} ]: CommunicationServer stopped successfully",
                    "Simulation Controller".green(),
                )
            }
            CommunicationServerEvent::ClientRegistered(client) => {
                info!(
                    "[ {} ]: CommunicationServer registered [ Client {} ]",
                    "Simulation Controller".green(),
                    client,
                )
            }
            CommunicationServerEvent::ClientDeregistered(client) => {
                info!(
                    "[ {} ]: CommunicationServer deregistered [ Client {} ]",
                    "Simulation Controller".green(),
                    client,
                )
            }
            CommunicationServerEvent::MessageForwarded(dest, msg) => {
                info!(
                    "[ {} ]: CommunicationServer forwarded the message {:?} to [ Client {} ]",
                    "Simulation Controller".green(),
                    msg,
                    dest
                )
            }
            CommunicationServerEvent::MessageReceived(src, msg) => info!(
                "[ {} ]: CommunicationServer received the message {:?} from [ Client {} ]",
                "Simulation Controller".green(),
                msg,
                src
            ),
            CommunicationServerEvent::UnreachableClient(client) => {
                error!(
                    "[ {} ]: received an error message: [ Client {} ] is unreachable",
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
                if let Some(dest) = packet
                    .routing_header
                    .hops
                    .get(0)
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
            },
            CommunicationServerEvent::DestinationIsDrone(drone) => {
                error!(
                    "[ {} ]: received an error message: The selected destination is a drone [ Drone {} ]",
                    "Simulation Controller".red(),
                    drone
                );
            },
            CommunicationServerEvent::ErrorPacketCache(session_id, fragment_index) => {
                error!(
                    "[ {} ]: received an error message: Error in the packet cache [ session_id : {}, fragment_index: {} ]",
                    "Simulation Controller".red(),
                    session_id,
                    fragment_index
                );
            },
        }
    }

    // Handle Server Command
    pub fn handle_commserver_command(&mut self, comm_server: &NodeId, command: CommunicationServerCommand) {
        match command {
            CommunicationServerCommand::InitFlooding => {
                if let Some((server, _)) = self.comm_servers.get(comm_server) {
                    match server.send(CommunicationServerCommand::InitFlooding) {
                        Ok(()) => info!(
                            "[ {} ]: sent a CommunicationServerCommand::InitFlooding to [ CommunicationServer {} ]",
                            "Simulation Controller".green(),
                            comm_server
                        ),
                        Err(e) => error!(
                            "[ {} ]: failed to send a CommunicationServerCommand::InitFlooding to the [ CommunicationServer {} ]: {}",
                            "Simulation Controller".red(),
                            comm_server,
                            e
                        ),
                    }
                } else {
                    error!(
                        "[ {} ]: failed to find a Sender<CommunicationServerCommand> channel for the [ CommunicationServer {} ]",
                        "Simulation Controller".red(),
                        comm_server
                    );
                }
            }
            CommunicationServerCommand::AddSender(node_id, sender) => {
                if let Some((server, _)) = self.comm_servers.get(comm_server) {
                    if let Some(vec) = self.neighbor.get_mut(comm_server) {
                        vec.push(node_id);
                        match server.send(CommunicationServerCommand::AddSender(node_id, sender)) {
                            Ok(()) => info!(
                                "[ {} ]: sent a CommunicationServerCommand::AddSender({}, sender_channel) to [ CommunicationServer {} ]",
                                "Simulation Controller".green(),
                                node_id,
                                comm_server
                            ),
                            Err(e) => error!(
                                "[ {} ]: failed to send a CommunicationServerCommand::AddSender({}, sender_channel) to the [ CommunicationServer {} ]: {}",
                                "Simulation Controller".red(),
                                node_id,
                                comm_server,
                                e
                            ),
                        }
                    } else {
                        error!(
                            "[ {} ]: the [ CommunicationServer {} ] does not have any neighbor",
                            "Simulation Controller".red(),
                            comm_server
                        );
                    }
                } else {
                    error!(
                        "[ {} ]: failed to find a Sender<CommunicationServerCommand> channel for the [ CommunicationServer {} ]",
                        "Simulation Controller".red(),
                        comm_server
                    );
                }
            }
            CommunicationServerCommand::RemoveSender(node_id) => {
                if let Some((server, _)) = self.comm_servers.get(comm_server) {
                    if let Some(vec) = self.neighbor.get_mut(comm_server) {
                        if vec.len() > 2 {
                            vec.retain(|x| *x != node_id);
                            match server.send(CommunicationServerCommand::RemoveSender(node_id)) {
                                Ok(()) => {
                                    info!(
                                    "[ {} ]: sent a CommunicationServerCommand::RemoveSender({}) to [ CommunicationServer {} ]",
                                    "Simulation Controller".green(),
                                    node_id,
                                    comm_server
                                );},
                                Err(e) => error!(
                                    "[ {} ]: failed to send a CommunicationServerCommand::RemoveSender({}) to the [ CommunicationServer {} ]: {}",
                                    "Simulation Controller".red(),
                                    node_id,
                                    comm_server,
                                    e
                                ),
                            }
                        } else {
                            error!(
                                "[ {} ]: the [ CommunicationServer {} ] must be connected to at least two nodes",
                                "Simulation Controller".red(),
                                comm_server
                            );
                        }
                    } else {
                        error!(
                            "[ {} ]: the [ CommunicationServer {} ] does not have any neighbor",
                            "Simulation Controller".red(),
                            comm_server
                        );
                    }
                } else {
                    error!(
                        "[ {} ]: failed to find a Sender<CommunicationServerCommand> channel for the [ CommunicationServer {} ]",
                        "Simulation Controller".red(),
                        comm_server
                    );
                }
            }
        }
    }
    
}