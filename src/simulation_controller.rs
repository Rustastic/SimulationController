use crossbeam_channel::{Receiver, Sender};
use std::{collections::HashMap, io};

use colored::Colorize;

use wg_2024::{
    controller::{DroneCommand, DroneEvent},
    network::NodeId,
    packet::{Packet, PacketType},
};

use gui::{GUICommands, GUIEvents};

//use chat_client::ChatClient;

use crate::{action, helpers::user_interaction};

#[derive(Clone)]
pub struct SimulationController {
    pub drones: HashMap<NodeId, (Sender<DroneCommand>, Sender<Packet>)>,
    receiver: Receiver<DroneEvent>,
    pub neighbor: HashMap<NodeId, Vec<NodeId>>,
    pub event_send: Sender<DroneEvent>,
    gui_send: Sender<GUIEvents>,
    gui_recv: Receiver<GUICommands>
}

impl SimulationController {
    pub fn new(
        drones: HashMap<NodeId, (Sender<DroneCommand>, Sender<Packet>)>,
        receiver: Receiver<DroneEvent>,
        neighbor: HashMap<NodeId, Vec<NodeId>>,
        event_send: Sender<DroneEvent>,
        gui_send: Sender<GUIEvents>,
        gui_recv: Receiver<GUICommands>
    ) -> Self {
        return Self {
            drones,
            receiver,
            neighbor,
            event_send,
            gui_send,
            gui_recv
        };
    }

    fn drone_action_handler(&mut self) {
        // UI menu
        println!("Select the action to execute:");
        println!("0 - Spawn");
        println!("1 - Crash");
        println!("2 - RemoveSender");
        println!("3 - AddSender");
        println!("4 - SetPackageDropRate");
        println!("5 - Print");
        println!("6 - None");
        println!("\nChiose: ");

        // Create input sting
        let mut input = String::new();
        // Get input from stdin
        let _ = io::stdin().read_line(&mut input);

        // Parse and verify input
        let number: i32;
        match user_interaction::parse_and_verify(&mut input) {
            Ok(node_id) => number = node_id,
            Err(e) => {
                println!("{}", e);
                return;
            }
        }

        // Handle chiose
        match number {
            0 => action::spawn(self),
            1 => action::crash(self),
            2 => action::remove_sender(self),
            3 => action::add_sender(self),
            4 => action::set_pdr(self),
            5 => action::print(self),
            6 => println!("None selected"),
            _ => eprintln!("[ ERROR ]: Select a number between 0 and 6"),
        }
    }

    fn client_action_handler(&mut self) {}

    fn ask_action(&mut self) {
        // UI menu
        println!("Would u like to perform an action on:");
        println!("0 - Drone");
        println!("1 - Client");
        println!("\nChiose: ");

        // Create input sting
        let mut input = String::new();
        // Get input from stdin
        let _ = io::stdin().read_line(&mut input);

        // Parse and verify input
        let category: i32;
        match input.trim_end().parse::<i32>() {
            Ok(value) => category = value,
            Err(e) => {
                eprintln!("[ ERROR ]: Please insert a valid value: {}", e);
                return;
            }
        }
        // Clear input string
        input.clear();

        // Handle chiose
        match category {
            0 => self.drone_action_handler(),
            1 => self.client_action_handler(),
            _ => eprintln!("[ ERROR ]: The number must be either 0 or 1"),
        }
    }

    pub fn run(&mut self) {
        // Start loop
        loop {
            // Check for action to perform
            //self.ask_action();

            // Check if any events are received
            match self.receiver.try_recv() {
                Ok(drone_event) => {
                    self.handle_event(drone_event);
                }
                Err(e) => match e {
                    crossbeam_channel::TryRecvError::Empty => continue,
                    crossbeam_channel::TryRecvError::Disconnected => eprintln!(
                        "[ {} ]: DroneEvent receiver channel disconnected: {}",
                        "Simulation Controller".red(),
                        e
                    ),
                },
            }

            // Check if any commands are received
            match self.gui_recv.try_recv() {
                Ok(gui_command) => {
                    self.handle_gui_command(gui_command);
                },
                Err(e) => match e {
                    crossbeam_channel::TryRecvError::Empty => continue,
                    crossbeam_channel::TryRecvError::Disconnected => eprintln!(
                        "[ {} ]: GUICommands receiver channel disconnected: {}",
                        "Simulation Controller".red(),
                        e
                    ),
                },
            }
        }
    }

    pub fn handle_event(&self, drone_event: DroneEvent) {
        match drone_event {
            DroneEvent::PacketSent(packet) => {
                let gui_packet = packet.clone();

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

                let packet_type = packet.clone().pack_type;

                // GUI
                match self.gui_send.send(GUIEvents::PacketSent(*src, *dest, gui_packet)) {
                    Ok(()) => println!(
                        "[ Simulation Controller ]: sent a GUIEvent: PacketSent({}, {}) sent to GUI",
                        src,
                        dest
                    ),
                    Err(e) => eprintln!(
                        "[ {} ]: failed to send GUIEvent: PacketSent({}, {}) sent to GUI: {}",
                        "Simulation Controller".red(),
                        src,
                        dest,
                        e
                    ),
                }

                println!(
                    "[ Drone: {} ]: Sent a Packet: {} to Drone {}",
                    src,
                    packet_type,
                    dest
                );
            },
            DroneEvent::PacketDropped(packet) => {
                let gui_packet = packet.clone();

                let drone = packet
                    .routing_header
                    .hops
                    .get(packet.routing_header.hop_index)
                    .unwrap();

                let session_id = packet.session_id;

                // GUI
                match self.gui_send.send(GUIEvents::PacketDropped(*drone, gui_packet)) {
                    Ok(()) => println!(
                        "[ Simulation Controller ]: sent a GUIEvent: PacketDropped({}) sent to GUI",
                        drone
                    ),
                    Err(e) => eprintln!(
                        "[ {} ]: failed to send GUIEvent: PacketDropped({}) sent to GUI: {}",
                        "Simulation Controller".red(),
                        drone,
                        e
                    ),
                }

                println!(
                    "[ Drone: {} ]: Dropped the packet with session_id: {}",
                    drone,
                    session_id
                );
            },
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
                            PacketType::MsgFragment(_) => {
                                panic!("Impossible how the hell did u do this")
                            }
                            _ => {
                                packet_channel.send(packet.clone()).unwrap();
                            }
                        }
                    } else {
                        eprintln!(
                            "[ {} ]: failed to find a Sender<Packet> channel for the [ Drone {} ]",
                            "Simulation Controller".red(),
                            dest
                        );
                    }
                } else {
                    eprintln!(
                        "[ {} ]: failed to find a Drone to send the DroneEvent: ControllerShortcut",
                        "Simulation Controller".red()
                    );
                }
            }
        }
    }

    pub fn handle_command(&mut self, drone: &NodeId, drone_command: DroneCommand) {
        if let Some((command_channel, _)) = self.drones.get(drone) {
            match drone_command {
                DroneCommand::RemoveSender(node_id) => {
                    if let Some(vec) = self.neighbor.get_mut(drone) {
                        vec.retain(|x| *x != node_id);
                        match command_channel.send(DroneCommand::RemoveSender(node_id)) {
                            Ok(()) => println!(
                                "[ Simulation Controller ]: sent a DroneCommand: RemoveSender({}) to [ Drone {} ]",
                                node_id,
                                drone
                            ),
                            Err(e) => eprintln!(
                                "[ {} ]: failed to send a DroneCommand: RemoveSender({}) to the [ Drone {} ]: {}",
                                "Simulation Controller".red(),
                                node_id,
                                drone,
                                e
                            ),
                        }
                    } else {
                        eprintln!(
                            "[ {} ]: the [ Drone {} ] does not have any neighbor",
                            "Simulation Controller".red(),
                            drone
                        );
                    }
                }
                DroneCommand::AddSender(node_id, sender) => {
                    if let Some(vec) = self.neighbor.get_mut(drone) {
                        vec.push(node_id);
                        match command_channel.send(DroneCommand::AddSender(node_id, sender)) {
                            Ok(()) => println!(
                                "[ Simulation Controller ]: sent a DroneCommand: AddSender({}, sender_channel) to [ Drone {} ]",
                                node_id,
                                drone
                            ),
                            Err(e) => eprintln!(
                                "[ {} ]: failed to send a DroneCommand: AddSender({}, sender_channel) to the [ Drone {} ]: {}",
                                "Simulation Controller".red(),
                                node_id,
                                drone,
                                e
                            ),
                        }
                    } else {
                        eprintln!(
                            "[ {} ]: the [ Drone {} ] does not have any neighbor",
                            "Simulation Controller".red(),
                            drone
                        );
                    }
                }
                DroneCommand::SetPacketDropRate(pdr) => {
                    match command_channel.send(DroneCommand::SetPacketDropRate(pdr)) {
                        Ok(()) => println!(
                            "[ Simulation Controller ]: sent a DroneCommand: SetPacketDropRate({}) to [ Drone {} ]",
                            pdr,
                            drone
                        ),
                        Err(e) => eprintln!(
                            "[ {} ]: failed to send a DroneCommand: SetPacketDropRate({}) to the [ Drone {} ]: {}",
                            "Simulation Controller".red(),
                            pdr,
                            drone,
                            e
                        ),
                    }
                }
                DroneCommand::Crash => {
                    if let Some((command_send, packet_send)) = self.drones.get(drone) {
                        let _ = drop(command_send);
                        let _ = drop(packet_send);
                    }

                    let drone_entry = self.drones.remove(drone);

                    self.neighbor.remove(drone);

                    if let Some((command_channel, _)) = drone_entry {
                        match command_channel.send(DroneCommand::Crash) {
                            Ok(()) => println!(
                                "[ Simulation Controller ]: sent a DroneCommand: Crash() to [ Drone {} ]",
                                drone
                            ),
                            Err(e) => eprintln!(
                                "[ {} ]: failed to send a DroneCommand: Crash() to the [ Drone {} ]: {}",
                                "Simulation Controller".red(),
                                drone,
                                e
                            ),
                        }
                    } else {
                        eprintln!(
                            "[ {} ]: the [ Drone {} ] was not found in the drones map",
                            "Simulation Controller".red(),
                            drone
                        );
                    }
                }
            }
        } else {
            eprintln!(
                "[ {} ]: failed to find a Sender<DroneCommand> channel for the [ Drone {} ]",
                "Simulation Controller".red(),
                drone
            );
        }
    }

    fn handle_gui_command(&mut self, command: GUICommands) {
        match command {
            GUICommands::Spawn => return,
            GUICommands::Crash(drone) => self.handle_command(&drone, DroneCommand::Crash),
            GUICommands::RemoveSender(drone, neighbor) => self.handle_command(&drone, DroneCommand::RemoveSender(neighbor)),
            GUICommands::AddSender(drone, neighbor) => return,
            GUICommands::SetPDR(drone, pdr) => self.handle_command(&drone, DroneCommand::SetPacketDropRate(pdr)),
        }
    }
}
