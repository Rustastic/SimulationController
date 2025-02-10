use colored::Colorize;
use log::{error, info};

use wg_2024::{network::NodeId, packet::PacketType};

use messages::{client_commands::{MediaClientCommand, MediaClientEvent}, gui_commands::GUIEvents};

use crate::SimulationController;

impl SimulationController {

    // Handle MediaClient Event
    pub fn handle_mclient_event(&mut self, event: MediaClientEvent) {
        info!(
            "[ {} ] Is a {:?}",
            "Simulation Controller".yellow(),
            event
        );
        match event {
            MediaClientEvent::ReceveidFloodResponse => {
                info!(
                    "[ {} ]: The media client retrieved a FloodResponse",
                    "Simulation Controller".green()
                )
            }
            MediaClientEvent::RemovedSender(drone) => {
                info!(
                    "[ {} ]: The media client removed the neighbor [ Drone {} ]",
                    "Simulation Controller".green(),
                    drone
                )
            }
            MediaClientEvent::AddedSender(drone) => {
                info!(
                    "[ {} ]: The media client added the neighbor [ Drone {} ]",
                    "Simulation Controller".green(),
                    drone
                )
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
            MediaClientEvent::ReceveidFileList(server, items) => {
                info!(
                    "[ {} ]: received the file list of [ TextServer {} ]",
                    "Simulation Controller".green(),
                    server,
                );
                match self.gui_send.send(GUIEvents::FileList(server, items.clone())) {
                    Ok(()) => info!(
                        "[ {} ]: successfully sent a GUIEvents::FileList({}, {:?}) from the Simulation Controller to the GUI",
                        "Simulation Controller".green(),
                        server,
                        items
                    ),
                    Err(e) => error!(
                        "[ {} ]: failed to sent a GUIEvents::FileList({}, {:?}) from the Simulation Controller to the GUI: {}",
                        "Simulation Controller".green(),
                        server,
                        items,
                        e
                    ),
                }
            },
            MediaClientEvent::ReceveidFile(node_id, _, file_response) => {
                info!(
                    "[ {} ]: received a file from [ MediaClient {} ]",
                    "Simulation Controller".green(),
                    node_id,
                );
                match self.gui_send.send(GUIEvents::MessageReceived(node_id, file_response.clone())) {
                    Ok(()) => info!(
                        "[ {} ]: successfully sent a GUIEvents::MessageReceived({}, {:?}) from the Simulation Controller to the GUI",
                        "Simulation Controller".green(),
                        node_id,
                        file_response
                    ),
                    Err(e) => error!(
                        "[ {} ]: failed to sent a GUIEvents::MessageReceived({}, {:?}) from the Simulation Controller to the GUI: {}",
                        "Simulation Controller".green(),
                        node_id,
                        file_response,
                        e
                    ),
                }
            }
            MediaClientEvent::ControllerShortcut(packet) => {
                if let Some(dest) = packet
                    .routing_header
                    .hops
                    .get(packet.routing_header.hops.len() - 1)
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
                        "[ {} ]: failed to find a MediaClient to send the MediaClientCommand::ControllerShortcut",
                        "Simulation Controller".red()
                    );
                }
            }
            //MediaClientEvent::ServerList(items) => todo!(),
            //MediaClientEvent::ReceveidServerType(_, server_type) => todo!(),
            _ => {
                error!("NOPE -> Not Implemented");
            }            
        }
    }

    // Handle MediaClient Command
    pub fn handle_mclient_command(&mut self, media_client: &NodeId, command: MediaClientCommand) {
        match command {
            MediaClientCommand::InitFlooding => {
                if let Some((client, _)) = self.mclients.get(media_client) {
                    match client.send(MediaClientCommand::InitFlooding) {
                        Ok(()) => info!(
                            "[ {} ]: sent a MediaClientCommand::InitFlo0ding to [ Client {} ]",
                            "Simulation Controller".green(),
                            media_client
                        ),
                        Err(e) => error!(
                            "[ {} ]: failed to send a MediaClientCommand::InitFlooding to the [ Client {} ]: {}",
                            "Simulation Controller".red(),
                            media_client,
                            e
                        ),
                    }
                } else {
                    error!(
                        "[ {} ]: failed to find a Sender<MediaClientCommand> channel for the [ Client {} ]",
                        "Simulation Controller".red(),
                        media_client
                    );
                }
            }
            MediaClientCommand::RemoveSender(drone) => {
                if let Some(neighbors) = self.neighbor.get_mut(media_client) {
                    // Max 2 neighbor, Min 1 neighbor
                    if neighbors.len() == 2 {
                        neighbors.retain(|x| *x != drone);
                        if let Some((client, _)) = self.mclients.get(media_client) {
                            match client.send(MediaClientCommand::RemoveSender(drone)) {
                                Ok(()) => info!(
                                    "[ {} ]: sent a MediaClientCommand::RemoveSender({}) to [ Client {} ]",
                                    "Simulation Controller".green(),
                                    drone,
                                    media_client
                                ),
                                Err(e) => error!(
                                    "[ {} ]: failed to send a MediaClientCommand::RemoveSender({}) to the [ Client {} ]: {}",
                                    "Simulation Controller".red(),
                                    drone,
                                    media_client,
                                    e
                                ),
                            }
                        } else {
                            error!(
                                "[ {} ]: failed to find a Sender<MediaClientCommand> channel for the [ Client {} ]",
                                "Simulation Controller".red(),
                                media_client
                            );
                        }
                    } else {
                        error!(
                            "[ {} ]: failed to send a MediaClientCommand::RemoveSender({}) to the [ Client {} ]: {}",
                            "Simulation Controller".red(),
                            drone,
                            media_client,
                            "Each client must remain connected to at least one drone"
                        );
                    }
                } else {
                    error!(
                        "[ {} ]: the [ Drone {} ] does not have any neighbor",
                        "Simulation Controller".red(),
                        drone
                    );
                }
            }
            MediaClientCommand::AddSender(drone, sender) => {
                // cant connect to a client
                if !self.mclients.contains_key(&drone) {
                    if let Some(neighbors) = self.neighbor.get_mut(media_client) {
                        // Max 2 neighbor, Min 1 neighbor
                        if neighbors.len() == 1 {
                            neighbors.push(drone);
                            if let Some((client, _)) = self.mclients.get(media_client) {
                                match client.send(MediaClientCommand::AddSender(drone, sender.clone())) {
                                    Ok(()) => info!(
                                        "[ {} ]: sent a MediaClientCommand::AddSender({}, {:?}) to [ Client {} ]",
                                        "Simulation Controller".green(),
                                        drone,
                                        sender,
                                        media_client
                                    ),
                                    Err(e) => error!(
                                        "[ {} ]: failed to send a MediaClientCommand::AddSender({}, {:?}) to the [ Client {} ]: {}",
                                        "Simulation Controller".red(),
                                        drone,
                                        sender,
                                        media_client,
                                        e
                                    ),
                                }
                            } else {
                                error!(
                                    "[ {} ]: failed to find a Sender<MediaClientCommand> channel for the [ Client {} ]",
                                    "Simulation Controller".red(),
                                    media_client
                                );
                            }
                        } else {
                            error!(
                                "[ {} ]: failed to send a MediaClientCommand::AddSender({}, {:?}) to the [ Client {} ]: {}",
                                "Simulation Controller".red(),
                                drone,
                                sender,
                                media_client,
                                "Each client must be connected to at most two drones"
                            );
                        }
                    } else {
                        error!(
                            "[ {} ]: the [ Drone {} ] does not have any neighbor",
                            "Simulation Controller".red(),
                            drone
                        );
                    }
                } else {
                    error!(
                        "[ {} ]: The selected NodeId: {} correspond to a Client not a Drone",
                        "Simulation Controller".red(),
                        media_client
                    );
                }
            }
            MediaClientCommand::AskFilesList(server) => {
                if let Some((client, _)) = self.mclients.get(media_client) {
                    match client.send(MediaClientCommand::AskFilesList(server)) {
                        Ok(()) => info!(
                            "[ {} ]: sent a MediaClientCommand::AskFilesList({}) to [ Client {} ]",
                            "Simulation Controller".green(),
                            server,
                            media_client
                        ),
                        Err(e) => error!(
                            "[ {} ]: failed to send a MediaClientCommand::AskFilesList({}) to the [ Client {} ]: {}",
                            "Simulation Controller".red(),
                            server,
                            media_client,
                            e
                        ),
                    }
                } else {
                    error!(
                        "[ {} ]: failed to find a Sender<MediaClientCommand> channel for the [ Client {} ]",
                        "Simulation Controller".red(),
                        media_client
                    );
                }
            }
            MediaClientCommand::AskForFile(server, title) => {
                if let Some((client, _)) = self.mclients.get(media_client) {
                    match client.send(MediaClientCommand::AskForFile(server, title.clone())) {
                        Ok(()) => info!(
                            "[ {} ]: sent a MediaClientCommand::AskForFile({}, {}) to [ Client {} ]",
                            "Simulation Controller".green(),
                            server,
                            title,
                            media_client
                        ),
                        Err(e) => error!(
                            "[ {} ]: failed to send a MediaClientCommand::AskForFile({}, {}) to the [ Client {} ]: {}",
                            "Simulation Controller".red(),
                            server,
                            title,
                            media_client,
                            e
                        ),
                    }
                } else {
                    error!(
                        "[ {} ]: failed to find a Sender<MediaClientCommand> channel for the [ Client {} ]",
                        "Simulation Controller".red(),
                        media_client
                    );
                }
            }
            //MediaClientCommand::GetServerList => todo!(),
            //MediaClientCommand::AskServerType(_) => todo!(),
            _ => {
                error!("NOPE -> Not Implemented");
            }
        }
    }
}
