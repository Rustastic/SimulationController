use colored::Colorize;
use crossbeam_channel::{unbounded, Receiver, Sender};
use log::{error, info};
use rand::Rng;
use std::collections::HashMap;

use wg_2024::{
    config::Drone as ConfigDrone,
    controller::{DroneCommand, DroneEvent},
    drone::Drone,
    network::NodeId,
    packet::Packet,
};

use messages::{
    client_commands::{ChatClientCommand, MediaClientCommand},
    server_commands::{CommunicationServerCommand, ContentServerCommand},
};

use crate::{helpers::error::Error, SimulationController};

type DroneFactoryFn = dyn Fn(
    &mut SimulationController,
    &ConfigDrone,
    &Receiver<DroneCommand>,
    &Receiver<Packet>,
) -> Box<dyn Drone>;

impl SimulationController {
    fn drone_factory<T>() -> Box<DroneFactoryFn>
    where
        T: Drone + 'static,
    {
        Box::new(|sim_ctrl, drone, command_recv, packet_recv| {
            // Create packet send hashmap
            let mut packet_send_hashmap = HashMap::<NodeId, Sender<Packet>>::new();
            // Fill hashmap with only
            let mut packet_send;
            for neighbor in &drone.connected_node_ids {
                if let Some((_, chan)) = sim_ctrl.drones.get(neighbor) {
                    packet_send = chan.clone();
                } else if let Some((_, chan)) = sim_ctrl.cclients.get(neighbor) {
                    packet_send = chan.clone();
                } else if let Some((_, chan)) = sim_ctrl.mclients.get(neighbor) {
                    packet_send = chan.clone();
                } else if let Some((_, chan)) = sim_ctrl.comm_servers.get(neighbor) {
                    packet_send = chan.clone();
                } else if let Some((_, chan)) = sim_ctrl.text_servers.get(neighbor) {
                    packet_send = chan.clone();
                } else {
                    let (_, chan) = sim_ctrl.media_servers.get(neighbor).unwrap();
                    packet_send = chan.clone();
                }
                packet_send_hashmap.insert(*neighbor, packet_send);
            }

            // Get drone's command receiver channel
            Box::new(T::new(
                drone.id,
                sim_ctrl.event_send.clone(),
                command_recv.clone(),
                packet_recv.clone(),
                packet_send_hashmap,
                drone.pdr,
            ))
        })
    }

    #[allow(clippy::missing_errors_doc)]
    pub fn spawn(
        &mut self,
        id: NodeId,
        connected_node_ids: Vec<NodeId>,
        pdr: f32,
    ) -> Result<(), Error> {
        // Create new drone
        let drone = ConfigDrone {
            id,
            connected_node_ids,
            pdr,
        };

        // Generate random number to pick a random factory
        let rand = rand::rng().random_range(0..10);
        let drone_factories = [
            Self::drone_factory::<rustbusters_drone::RustBustersDrone>(),
            Self::drone_factory::<rustbusters_drone::RustBustersDrone>(),
            Self::drone_factory::<rustbusters_drone::RustBustersDrone>(),
            Self::drone_factory::<rustbusters_drone::RustBustersDrone>(),
            Self::drone_factory::<rustbusters_drone::RustBustersDrone>(),
            Self::drone_factory::<rustbusters_drone::RustBustersDrone>(),
            Self::drone_factory::<rustbusters_drone::RustBustersDrone>(),
            Self::drone_factory::<rustbusters_drone::RustBustersDrone>(),
            Self::drone_factory::<rustbusters_drone::RustBustersDrone>(),
            Self::drone_factory::<rustbusters_drone::RustBustersDrone>(),
            
            /*Self::drone_factory::<rusty_drones::RustyDrone>(),
            Self::drone_factory::<LeDron_James::Drone>(),
            Self::drone_factory::<dr_ones::Drone>(),
            //Self::drone_factory::<skylink::SkyLinkDrone>(),
            Self::drone_factory::<rustbusters_drone::RustBustersDrone>(),
            Self::drone_factory::<rustbusters_drone::RustBustersDrone>(),
            Self::drone_factory::<rustbusters_drone::RustBustersDrone>(),
            Self::drone_factory::<rust_roveri::RustRoveri>(),
            Self::drone_factory::<rust_do_it::RustDoIt>(),
            Self::drone_factory::<wg_2024_rust::drone::RustDrone>(),
            //Self::drone_factory::<null_pointer_drone::MyDrone>(),
            Self::drone_factory::<lockheedrustin_drone::LockheedRustin>(),*/
        ];

        // create necessary channels
        let (command_send, command_recv) = unbounded::<DroneCommand>();
        let (packet_send, packet_recv) = unbounded::<Packet>();

        // Crate drone
        if let Some(factory) = drone_factories.get(rand) {
            let new_drone = factory(
                self,
                &drone,
                &command_recv,
                &packet_recv,
            );

            // Add drone to drone list
            self.drones.insert(id, (command_send, packet_send));

            // Add drone to neighbor list
            self.neighbor.insert(drone.id, Vec::new());

            self.new_drones.push(new_drone);

            info!(
                "[ {} ] Successfully create Drone({}, {:?}, {})",
                "Simulation Controller".green(),
                id,
                drone.connected_node_ids.clone(),
                pdr
            );

            Ok(())
        } else {
            Err(Error::MissingFactory(id))
        }
    }

    #[allow(clippy::missing_errors_doc)]
    pub fn crash(&mut self, node_id: NodeId) -> Result<(), Error> {
        // If the drone has any neighbors
        if let Some(neighbor_ids) = self.neighbor.get(&node_id).cloned() {
            for neighbor in neighbor_ids {
                if self.drones.contains_key(&neighbor) {
                    self.handle_drone_command(&neighbor, DroneCommand::RemoveSender(node_id));
                } else if self.cclients.contains_key(&neighbor) {
                    self.handle_chat_client_command(
                        &neighbor,
                        ChatClientCommand::RemoveSender(node_id),
                    );
                } else if self.mclients.contains_key(&neighbor) {
                    self.handle_media_client_command(
                        &neighbor,
                        MediaClientCommand::RemoveSender(node_id),
                    );
                } else if self.comm_servers.contains_key(&neighbor) {
                    self.handle_communication_server_command(
                        &neighbor,
                        CommunicationServerCommand::RemoveSender(node_id),
                    );
                } else if self.text_servers.contains_key(&neighbor) {
                    self.handle_text_server_command(
                        &neighbor,
                        ContentServerCommand::RemoveSender(node_id),
                    );
                } else {
                    self.handle_media_server_command(
                        &neighbor,
                        ContentServerCommand::RemoveSender(node_id),
                    );
                }
            }
        }

        Ok(())
    }

    #[allow(clippy::missing_errors_doc)]
    pub fn remove_sender(&mut self, node_id: NodeId, to_remove: NodeId) -> Result<(), Error> {
        // Check if it exists a neighbor with this id
        match self.has_neighbors(node_id) {
            Ok(neighbor) => {
                match super::verify::is_a_neighbor(neighbor, to_remove, node_id, false) {
                    Ok(()) => {
                        if self.drones.contains_key(&to_remove) {
                            self.handle_drone_command(
                                &to_remove,
                                DroneCommand::RemoveSender(node_id),
                            );
                            return Ok(());
                        } else if self.cclients.contains_key(&to_remove) {
                            self.handle_chat_client_command(
                                &to_remove,
                                ChatClientCommand::RemoveSender(node_id),
                            );
                            return Ok(());
                        } else if self.mclients.contains_key(&to_remove) {
                            self.handle_media_client_command(
                                &to_remove,
                                MediaClientCommand::RemoveSender(node_id),
                            );
                            return Ok(());
                        } else if self.comm_servers.contains_key(&to_remove) {
                            self.handle_communication_server_command(
                                &to_remove,
                                CommunicationServerCommand::RemoveSender(node_id),
                            );
                            return Ok(());
                        } else if self.text_servers.contains_key(&to_remove) {
                            self.handle_text_server_command(
                                &to_remove,
                                ContentServerCommand::RemoveSender(node_id),
                            );
                            return Ok(());
                        } else if self.media_servers.contains_key(&to_remove) {
                            self.handle_media_server_command(
                                &to_remove,
                                ContentServerCommand::RemoveSender(node_id),
                            );
                            return Ok(());
                        } else {
                            log::error!(
                                "[ {} ]: failed to send AddSender to [ Node {} ]",
                                "Simulation Controller".red(),
                                to_remove
                            );
                        }
                        Err(Error::ClientOnClient)
                    }
                    Err(e) => Err(e),
                }
            }
            Err(e) => Err(e),
        }
    }

    #[allow(clippy::missing_errors_doc)]
    pub fn add_sender(&mut self, node_id: NodeId, to_add: NodeId) -> Result<(), Error> {
        // Check if it exists a neighbor with this id
        match self.has_neighbors(node_id) {
            Ok(neighbor) => match super::verify::is_a_neighbor(neighbor, to_add, node_id, true) {
                Ok(()) => {
                    let packet_send;
                    if let Some((_, chan)) = self.drones.get(&node_id) {
                        packet_send = chan.clone();
                    } else if let Some((_, chan)) = self.cclients.get(&node_id) {
                        packet_send = chan.clone();
                    } else if let Some((_, chan)) = self.mclients.get(&node_id) {
                        packet_send = chan.clone();
                    } else if let Some((_, chan)) = self.comm_servers.get(&node_id) {
                        packet_send = chan.clone();
                    } else if let Some((_, chan)) = self.text_servers.get(&node_id) {
                        packet_send = chan.clone();
                    } else if let Some((_, chan)) = self.media_servers.get(&node_id) {
                        packet_send = chan.clone();
                    } else {
                        return Err(Error::NoSender(node_id));
                    }

                    if self.drones.contains_key(&to_add) {
                        self.handle_drone_command(
                            &to_add,
                            DroneCommand::AddSender(node_id, packet_send.clone()),
                        );

                        return Ok(());
                    } else if self.mclients.contains_key(&to_add) {
                        self.handle_media_client_command(
                            &to_add,
                            MediaClientCommand::AddSender(node_id, packet_send.clone()),
                        );

                        return Ok(());
                    } else if self.cclients.contains_key(&to_add) {
                        self.handle_chat_client_command(
                            &to_add,
                            ChatClientCommand::AddSender(node_id, packet_send.clone()),
                        );

                        return Ok(());
                    } else if self.comm_servers.contains_key(&to_add) {
                        self.handle_communication_server_command(
                            &to_add,
                            CommunicationServerCommand::AddSender(node_id, packet_send.clone()),
                        );

                        return Ok(());
                    } else if self.text_servers.contains_key(&to_add) {
                        self.handle_text_server_command(
                            &to_add,
                            ContentServerCommand::AddSender(node_id, packet_send.clone()),
                        );

                        return Ok(());
                    } else if self.media_servers.contains_key(&to_add) {
                        self.handle_media_server_command(
                            &to_add,
                            ContentServerCommand::AddSender(node_id, packet_send.clone()),
                        );

                        return Ok(());
                    } else {
                        log::error!(
                            "[ {} ]: failed to send AddSender to [ Node {} ]",
                            "Simulation Controller".red(),
                            to_add
                        );
                    }

                    Err(Error::ClientOnClient)
                }
                Err(e) => Err(e),
            },
            Err(e) => Err(e),
        }
    }
}
