use wg_2024::{controller::DroneCommand, network::NodeId};

use colored::Colorize;
use log::{error, info};

use crate::SimulationController;

impl SimulationController {
    #[allow(clippy::too_many_lines)]
    pub fn handle_drone_command(&mut self, drone: &NodeId, drone_command: DroneCommand) {
        // Get drone's sender channel
        if let Some((command_channel, _)) = self.drones.get(drone) {
            match drone_command {
                DroneCommand::RemoveSender(node_id) => {
                    // Get neighbors
                    if let Some(vec) = self.neighbor.get_mut(drone) {
                        // remove node from neighbors
                        vec.retain(|x| *x != node_id);
                        // Send command and handle Result
                        match command_channel.send(DroneCommand::RemoveSender(node_id)) {
                            Ok(()) => {
                                // Send command to GUI
                                self.gui_send.send(GUIEvents::RemoveSender(drone, node_id));

                                // Launch global flooding
                                self.global_flooding();

                                info!(
                                    "[ {} ]: sent a DroneCommand::RemoveSender({}) to [ Drone {} ]",
                                    "Simulation Controller".green(),
                                    node_id,
                                    drone
                                );
                            }
                            Err(e) => {
                                error!(
                                    "[ {} ]: failed to send a DroneCommand::RemoveSender({}) to the [ Drone {} ]: {}",
                                    "Simulation Controller".red(),
                                    node_id,
                                    drone,
                                    e
                                );
                            }
                        }
                    } else {
                        error!(
                            "[ {} ]: the [ Drone {} ] does not have any neighbor",
                            "Simulation Controller".red(),
                            drone
                        );
                    }
                }
                DroneCommand::AddSender(node_id, sender) => {
                    // Get neighbors
                    if let Some(vec) = self.neighbor.get_mut(drone) {
                        // Add node to neighbors
                        vec.push(node_id);
                        // Send command and handle Result
                        match command_channel.send(DroneCommand::AddSender(node_id, sender)) {
                            Ok(()) => {
                                // Send command to GUI
                                self.gui_send.send(GUIEvents::AddSender(drone, node_id));

                                // Launch global flooding
                                self.global_flooding();

                                info!(
                                    "[ {} ]: sent a DroneCommand::AddSender({}, sender_channel) to [ Drone {} ]",
                                    "Simulation Controller".green(),
                                    node_id,
                                    drone
                                );
                            }
                            Err(e) => {
                                error!(
                                    "[ {} ]: failed to send a DroneCommand::AddSender({}, sender_channel) to the [ Drone {} ]: {}",
                                    "Simulation Controller".red(),
                                    node_id,
                                    drone,
                                    e
                                );
                            }
                        }
                    } else {
                        error!(
                            "[ {} ]: the [ Drone {} ] does not have any neighbor",
                            "Simulation Controller".red(),
                            drone
                        );
                    }
                }
                DroneCommand::SetPacketDropRate(pdr) => {
                    // Send command and handle Result
                    match command_channel.send(DroneCommand::SetPacketDropRate(pdr)) {
                        Ok(()) => {
                            info!(
                                "[ {} ]: sent a DroneCommand::SetPacketDropRate({}) to [ Drone {} ]",
                                "Simulation Controller".green(),
                                pdr,
                                drone
                            );
                        }
                        Err(e) => {
                            error!(
                                "[ {} ]: failed to send a DroneCommand::SetPacketDropRate({}) to the [ Drone {} ]: {}",
                                "Simulation Controller".red(),
                                pdr,
                                drone,
                                e
                            );
                        }
                    }
                }
                DroneCommand::Crash => {
                    if let Some((command_send, packet_send)) = self.drones.get(drone) {
                        #[allow(dropping_references)]
                        drop(command_send);
                        #[allow(dropping_references)]
                        drop(packet_send);
                    }

                    let drone_entry = self.drones.remove(drone);

                    self.neighbor.remove(drone);

                    if let Some((command_channel, _)) = drone_entry {
                        match command_channel.send(DroneCommand::Crash) {
                            Ok(()) => info!(
                                "[ {} ]: sent a DroneCommand::Crash() to [ Drone {} ]",
                                "Simulation Controller".green(),
                                drone
                            ),
                            Err(e) => error!(
                                "[ {} ]: failed to send a DroneCommand::Crash() to the [ Drone {} ]: {}",
                                "Simulation Controller".red(),
                                drone,
                                e
                            ),
                        }
                    } else {
                        error!(
                            "[ {} ]: the [ Drone {} ] was not found in the drones map",
                            "Simulation Controller".red(),
                            drone
                        );
                    }
                }
            }
        } else {
            error!(
                "[ {} ]: failed to find a Sender<DroneCommand> channel for the [ Drone {} ]",
                "Simulation Controller".red(),
                drone
            );
        }
    }
}
