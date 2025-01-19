use std::collections::HashMap;
use crossbeam_channel::{Receiver, Sender};

use log::{error, info, warn};
use colored::Colorize;

use wg_2024::{
    controller::{DroneCommand, DroneEvent}, drone::Drone, network::NodeId, packet::{Packet, PacketType}
};

struct SimulationController {
    drones: HashMap<NodeId, (Sender<DroneCommand>, Sender<Packet>)>,
    receiver: Receiver<DroneEvent>
}

impl SimulationController {
    fn new(drones: HashMap<NodeId, (Sender<DroneCommand>, Sender<Packet>)>, recv: Receiver<DroneEvent>) -> Self {
        return Self {
            drones,
            receiver: recv
        };
    }

    fn run(&mut self) {
        match self.receiver.recv() {
            Ok(drone_event) => self.handle_event(drone_event),
            Err(_) => error!("{} Channel is closed", "✗".red()),
        }

        
    }

    fn spawn() {
        
    }

    fn handle_event(&self, drone_event: DroneEvent) {
        match drone_event {
            DroneEvent::PacketSent(packet) => {
                let src = packet.routing_header.hops.get(packet.routing_header.hop_index).unwrap();
                let dest = packet.routing_header.hops.get(packet.routing_header.hop_index + 1).unwrap();
                let pakcet_type = packet.pack_type;

                // GUI
            },
            DroneEvent::PacketDropped(packet) => {
                let drone = packet.routing_header.hops.get(packet.routing_header.hop_index).unwrap();

                // GUI
            },
            DroneEvent::ControllerShortcut(packet) => {
                if let Some(dest) = packet.routing_header.hops.get(packet.routing_header.len()) {
                    if let Some((_, packet_channel)) = self.drones.get(dest) {
                        match packet.pack_type {
                            PacketType::MsgFragment(_) => error!(""),
                            _ => {
                                packet_channel.send(packet.clone()).unwrap();
                            },
                        }
                    }
                } else {
                    error!("");
                }
            },
        }
    }

    fn handle_command(&self, drone: &NodeId, drone_command: DroneCommand) {
        if let Some((command_channel, _)) = self.drones.get(drone) {
            match drone_command {
                DroneCommand::RemoveSender(node_id) => {
                    command_channel.send(DroneCommand::RemoveSender(node_id)).unwrap();
                },
                DroneCommand::AddSender(node_id, sender) => {
                    command_channel.send(DroneCommand::AddSender(node_id, sender)).unwrap();
                },
                DroneCommand::SetPacketDropRate(pdr) => {
                    command_channel.send(DroneCommand::SetPacketDropRate(pdr)).unwrap();
                },
                DroneCommand::Crash => {
                    command_channel.send(DroneCommand::Crash).unwrap();
                },
            }
        } else {
            error!("");
        }
    }
}