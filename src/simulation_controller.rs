use crossbeam_channel::{Receiver, Sender};
use std::collections::HashMap;

use colored::Colorize;
use log::{error, info, warn};

use wg_2024::{
    controller::{DroneCommand, DroneEvent},
    network::NodeId,
    packet::{Packet, PacketType},
};

#[derive(Clone)]
pub struct SimulationController {
    drones: HashMap<NodeId, (Sender<DroneCommand>, Sender<Packet>)>,
    receiver: Receiver<DroneEvent>,
    neighbor: HashMap<NodeId, Vec<NodeId>>,
}

impl SimulationController {
    pub fn new(
        drones: HashMap<NodeId, (Sender<DroneCommand>, Sender<Packet>)>,
        receiver: Receiver<DroneEvent>,
        neighbor: HashMap<NodeId, Vec<NodeId>>,
    ) -> Self {
        return Self {
            drones,
            receiver,
            neighbor,
        };
    }

    pub fn run(&mut self) {
        loop {
            match self.receiver.recv() {
                Ok(drone_event) => self.handle_event(drone_event),
                Err(_) => error!("{} Channel is closed", "✗".red()),
            }
        }
    }

    fn spawn() {}

    fn handle_event(&self, drone_event: DroneEvent) {
        match drone_event {
            DroneEvent::PacketSent(packet) => {
                let src = packet
                    .routing_header
                    .hops
                    .get(packet.routing_header.hop_index)
                    .unwrap();

                let dest = packet
                    .routing_header
                    .hops
                    .get(packet.routing_header.hop_index + 1)
                    .unwrap();

                let pakcet_type = packet.pack_type;

                // GUI
            }
            DroneEvent::PacketDropped(packet) => {
                let drone = packet
                    .routing_header
                    .hops
                    .get(packet.routing_header.hop_index)
                    .unwrap();

                // GUI
            }
            DroneEvent::ControllerShortcut(packet) => {
                // Get packet destination node
                if let Some(dest) = packet
                    .routing_header
                    .hops
                    .get(packet.routing_header.len() - 1)
                {
                    // Get destination node channel
                    if let Some((_, packet_channel)) = self.drones.get(dest) {
                        // Send Packet t destination
                        match packet.pack_type {
                            PacketType::MsgFragment(_) => error!(""),
                            _ => {
                                packet_channel.send(packet.clone()).unwrap();
                            }
                        }
                    } else {
                        error!(
                            "{} [ Simulation Controller ]: failed to find a Sender<Packet> channel for the [ Drone {} ]",
                            "✗".red(),
                            dest
                        );
                    }
                } else {
                    error!(
                        "{} [ Simulation Controller ]: failed to find a Drone to send the DroneEvent: ControllerShortcut",
                        "✗".red()
                    );
                }
            }
        }
    }

    fn handle_command(&self, drone: &NodeId, drone_command: DroneCommand) {
        if let Some((command_channel, _)) = self.drones.get(drone) {
            match drone_command {
                DroneCommand::RemoveSender(node_id) => {
                    match command_channel.send(DroneCommand::RemoveSender(node_id)) {
                        Ok(()) => info!(
                            "{} [ Simulation Controller ]: sent a DroneCommand: RemoveSender({}) sent to [ Drone {} ]",
                            "✓".green(),
                            node_id,
                            drone
                        ),
                        Err(e) => error!(
                            "{} [ Simulation Controller ]: failed to send a DroneCommand: RemoveSender({}) to the [ Drone {} ]: {}",
                            "✗".red(),
                            node_id,
                            drone,
                            e
                        ),
                    }
                }
                DroneCommand::AddSender(node_id, sender) => {
                    match command_channel.send(DroneCommand::AddSender(node_id, sender)) {
                        Ok(()) => info!(
                            "{} [ Simulation Controller ]: sent a DroneCommand: AddSender({}, sender_channel) sent to [ Drone {} ]",
                            "✓".green(),
                            node_id,
                            drone
                        ),
                        Err(e) => error!(
                            "{} [ Simulation Controller ]: failed to send a DroneCommand: AddSender({}, sender_channel) to the [ Drone {} ]: {}",
                            "✗".red(),
                            node_id,
                            drone,
                            e
                        ),
                    }
                }
                DroneCommand::SetPacketDropRate(pdr) => {
                    match command_channel.send(DroneCommand::SetPacketDropRate(pdr)) {
                        Ok(()) => info!(
                            "{} [ Simulation Controller ]: sent a DroneCommand: SetPacketDropRate({}) sent to [ Drone {} ]",
                            "✓".green(),
                            pdr,
                            drone
                        ),
                        Err(e) => error!(
                            "{} [ Simulation Controller ]: failed to send a DroneCommand: SetPacketDropRate({}) to the [ Drone {} ]: {}",
                            "✗".red(),
                            pdr,
                            drone,
                            e
                        ),
                    }
                }
                DroneCommand::Crash => {
                    if let Some(neighbors) = self.neighbor.get(drone) {
                        for neighbor in neighbors {
                            self.handle_command(neighbor, DroneCommand::RemoveSender(*drone));
                        }

                        match command_channel.send(DroneCommand::Crash) {
                            Ok(()) => info!(
                                "{} [ Simulation Controller ]: sent a DroneCommand: Crash() sent to [ Drone {} ]",
                                "✓".green(),
                                drone
                            ),
                            Err(e) => error!(
                                "{} [ Simulation Controller ]: failed to send a DroneCommand: Crash() to the [ Drone {} ]: {}",
                                "✗".red(),
                                drone,
                                e
                            ),
                        }
                    } else {
                        error!("{} [ Simulation Controller ]: failed to send a DroneCommand: Crash() to the [ Drone {} ]",
                            "✗".red(),
                            drone
                        );
                    }
                }
            }
        } else {
            error!("
                {} [ Simulation Controller ]: failed to find a Sender<DroneCommand> channel for the [ Drone {} ]",
                "✗".red(),
                drone
            );
        }
    }
}
