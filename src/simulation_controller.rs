use crossbeam_channel::{unbounded, Receiver, Sender};
use rand::Rng;
use std::{collections::HashMap, io};

use colored::Colorize;
use log::{error, info};

use wg_2024::{
    config::Drone as ConfigDrone,
    controller::{DroneCommand, DroneEvent},
    drone::Drone,
    network::NodeId,
    packet::{Packet, PacketType},
};

//use chat_client::ChatClient;

fn drone_factory<T>() -> Box<
    dyn Fn(
        &ConfigDrone,
        &Sender<DroneEvent>,
        &Receiver<DroneCommand>,
        &Receiver<Packet>,
        &Sender<Packet>,
    ) -> Box<dyn Drone>,
>
where
    T: Drone + 'static,
{
    Box::new(
        |drone, event_send, command_recv, packet_recv, packet_send| {
            // Create packet send hashmap
            let mut packet_send_hashmap = HashMap::<NodeId, Sender<Packet>>::new();
            // Fill hashmap with only neighbor
            for neighbor in &drone.connected_node_ids {
                packet_send_hashmap.insert(*neighbor, packet_send.clone());
            }

            // Get drone's command receiver channel
            return Box::new(T::new(
                drone.id,
                event_send.clone(),
                command_recv.clone(),
                packet_recv.clone(),
                packet_send_hashmap,
                drone.pdr,
            ));
        },
    )
}

#[derive(Clone)]
pub struct SimulationController {
    drones: HashMap<NodeId, (Sender<DroneCommand>, Sender<Packet>)>,
    receiver: Receiver<DroneEvent>,
    neighbor: HashMap<NodeId, Vec<NodeId>>,
    event_send: Sender<DroneEvent>,
}

impl SimulationController {
    pub fn new(
        drones: HashMap<NodeId, (Sender<DroneCommand>, Sender<Packet>)>,
        receiver: Receiver<DroneEvent>,
        neighbor: HashMap<NodeId, Vec<NodeId>>,
        event_send: Sender<DroneEvent>,
    ) -> Self {
        return Self {
            drones,
            receiver,
            neighbor,
            event_send,
        };
    }

    fn print(&self) {
        for (drone, _) in self.drones.iter() {
            println!("\n[ Drone: {} ]", drone);
            
            println!("Neighbors:");
            let neighbor_ids = self.neighbor.get(drone).unwrap();
            for neighbor in neighbor_ids {
                println!("\t[ Drone: {} ]", neighbor)
            }
        }
    }

    fn spawn(&mut self) {
        // Get ID of the new drone
        // UI menu
        println!("Please provide the necessary parameters for the new drone");
        println!("Id: ");
        
        // Create input sting
        let mut input = String::new();
        // Get input from stdin
        let _ = io::stdin().read_line(&mut input);

        // Parse and verify input
        let id: NodeId;
        match input.trim_end().parse::<NodeId>() {
            Ok(value) => id = value,
            Err(e) => {
                error!(
                    "{} [ ERROR ]: Please insert a valid NodeId: {}",
                    "✗".red(),
                    e
                );
                return;
            }
        }
        // Clear input String 
        input.clear();

        // Check if drone with this id already exist
        if self.drones.contains_key(&id) {
            error!(
                "{} [ ERROR ]: A drone with the NodeId: {} already exists",
                "✗".red(),
                id
            );
            return;
        }

        // Get all the neighbors
        // Vec containing the neighbor drones
        let mut connected_node_ids = Vec::<NodeId>::new();

        // loop to get all the neighbors
        let mut add = true;
        while add {
            // UI menu
            println!("Connected Drones Id: ");
            // Get input from stdin
            let _ = io::stdin().read_line(&mut input);

            // Parse and verify input
            let new_neighbor: NodeId;
            match input.trim_end().parse::<NodeId>() {
                Ok(value) => new_neighbor = value,
                Err(e) => {
                    error!(
                        "{} [ ERROR ]: Please insert a valid NodeId: {}",
                        "✗".red(),
                        e
                    );
                    return;
                }
            }
            // Clear input string
            input.clear();

            // Add drone to the neighbor vec
            if !connected_node_ids.contains(&new_neighbor) {
                connected_node_ids.push(new_neighbor);
            } else {
                error!(
                    "{} [ ERROR ]: The [ Drone: {} ] is already a neighbor",
                    "✗".red(),
                    new_neighbor
                );
            }

            // Ask if user want to add another drone
            // UI menu
            println!("Do you want to add another Drone to the neighbor list?\n 0->No 1->Yes");
            // Get input from stdin
            let _ = io::stdin().read_line(&mut input);

            // Parse and verify input
            match input.trim_end().parse::<u8>() {
                Ok(value) => {
                    if value != 0 {
                        add = true;
                    } else {
                        add = false;
                    }
                }
                Err(e) => {
                    error!(
                        "{} [ ERROR ]: Please insert a valid NodeId: {}",
                        "✗".red(),
                        e
                    );
                    return;
                }
            }
            //clear input string
            input.clear();
        }

        // Get drone pdr
        // UI menu
        println!("PDR: ");
        // Get input from stdin
        let _ = io::stdin().read_line(&mut input);

        // Parse and verify input
        let pdr: f32;
        match input.trim_end().parse::<f32>() {
            // Check if number is in the right range
            Ok(value) if (0.0..=1.0).contains(&value) => pdr = value,
            Ok(_) => {
                error!(
                    "{} [ ERROR ]: The PDR number is out of range. Please enter a number between 0 and 1.",
                    "✗".red(),
                );
                return;
            }
            Err(e) => {
                error!(
                    "{} [ ERROR ]: Please insert a valid f32 value: {}",
                    "✗".red(),
                    e
                );
                return;
            }
        }
        // Clear input String
        input.clear();

        // Create new drone
        let drone = ConfigDrone {
            id,
            connected_node_ids,
            pdr,
        };

        // Add drone to neighbor list
        self.neighbor.insert(drone.id, drone.connected_node_ids.clone());

        // Generate random number to pick a random factory
        let rand = rand::rng().random_range(0..10);
        let drone_factories = vec![
            drone_factory::<rusty_drones::RustyDrone>(),
            drone_factory::<LeDron_James::Drone>(),
            drone_factory::<dr_ones::Drone>(),
            drone_factory::<skylink::SkyLinkDrone>(),
            drone_factory::<rustbusters_drone::RustBustersDrone>(),
            drone_factory::<rust_roveri::RustRoveri>(),
            drone_factory::<rust_do_it::RustDoIt>(),
            drone_factory::<wg_2024_rust::drone::RustDrone>(),
            drone_factory::<null_pointer_drone::MyDrone>(),
            drone_factory::<lockheedrustin_drone::LockheedRustin>(),
        ];

        // create necessary channels
        let (command_send, command_recv) = unbounded::<DroneCommand>();
        let (packet_send, packet_recv) = unbounded::<Packet>();
        // Crate drone
        if let Some(factory) = drone_factories.get(rand) {
            factory(
                &drone,
                &self.event_send,
                &command_recv,
                &packet_recv,
                &packet_send,
            );

            // Add drone to drone list
            self.drones.insert(id, (command_send, packet_send));
        } else {
            panic!("No factory defined for [ Drone {} ]", drone.id);
        }
    }

    fn crash(&mut self) {
        // Get drone to crash
        // UI menu
        println!("Witch drone would you like to send the DroneCommand::Crash");
        for (node_id, _) in self.drones.iter() {
            println!("- [ Drone {} ]", node_id)
        }
        println!("Chiose: ");

        // Create input sting
        let mut input = String::new();
        // Get input from stdin
        let _ = io::stdin().read_line(&mut input);

        // Parse and verify input
        let target: NodeId;
        match input.trim_end().parse::<NodeId>() {
            Ok(value) => target = value,
            Err(e) => {
                error!(
                    "{} [ ERROR ]: Please insert a valid NodeId: {}",
                    "✗".red(),
                    e
                );
                return;
            }
        }
        // UI menu
        input.clear();

        // Check if it exists a drone with this id
        if !self.drones.contains_key(&target) {
            error!(
                "{} [ ERROR ]: There is not a drone with the NodeId: {}",
                "✗".red(),
                target
            );
            return;
        }

        // If the drone has any neighbors
        if let Some(neighbor_ids) = self.neighbor.get_mut(&target).cloned() {
            for neighbor in  neighbor_ids{
                // Send command to neighbors
                self.handle_command(&neighbor, DroneCommand::RemoveSender(target));
            }
        }

        // Send command
        self.handle_command(&target, DroneCommand::Crash);
    }

    fn remove_sender(&mut self) {
        // Get drone to which remove a sender
        // UI menu
        println!("Witch drone would you like to send the DroneCommand::RemoveSender");
        for (node_id, _) in self.drones.iter() {
            println!("- [ Drone {} ]", node_id);
        }
        println!("Chiose: ");

        // Create input sting
        let mut input = String::new();
        // Get input from stdin
        let _ = io::stdin().read_line(&mut input);

        // Parse and verify input
        let target: NodeId;
        match input.trim_end().parse::<NodeId>() {
            Ok(value) => target = value,
            Err(e) => {
                error!(
                    "{} [ ERROR ]: Please insert a valid NodeId: {}",
                    "✗".red(),
                    e
                );
                return;
            }
        }
        // Clear input string
        input.clear();

        // Check if it exists a drone with this id
        if !self.drones.contains_key(&target) {
            error!(
                "{} [ ERROR ]: There is not a drone with the NodeId: {}",
                "✗".red(),
                target
            );
            return;
        }

        // Get the neighbor to remove
        // UI menu
        println!("Which of his neighbor would u like to remove?");
        if let Some(neighbor) = self.neighbor.get(&target.clone()) {
            for node_id in neighbor {
                println!("- [ Drone {} ]", node_id)
            }
        } else {
            error!("{} [ ERROR ]: The selected drone does not exist or does not have any neighbor",
                "✗".red(),
            );
        }
        println!("Chiose: ");

        // Get input from stdin
        let _ = io::stdin().read_line(&mut input);

        // Parse and verify input
        let to_remove: NodeId;
        match input.trim_end().parse::<NodeId>() {
            Ok(value) => to_remove = value,
            Err(e) => {
                error!(
                    "{} [ ERROR ]: Please insert a valid NodeId: {}",
                    "✗".red(),
                    e
                );
                return;
            }
        }
        // Clear input String
        input.clear();

        // Check if it exists a neighbor with this id
        if let Some(neighbor) = self.neighbor.get(&target) {
            if !neighbor.contains(&to_remove) {
                error!(
                    "{} [ ERROR ]: The [ Drone: {} ] is not a neighbor of [ Drone: {} ]",
                    "✗".red(),
                    target,
                    to_remove
                );
            }  
        } else {
            error!("{} [ ERROR ]: The selected [ Drone: {} ] does not exist or does not have any neighbor",
                "✗".red(),
                target,
            );
        }

        // Send command
        self.handle_command(&target, DroneCommand::RemoveSender(to_remove));
        self.handle_command(&to_remove, DroneCommand::RemoveSender(target));
    }

    fn add_sender(&mut self) {
        // Get drone to which add a sender
        // UI menu
        println!("Witch drone would you like to send the DroneCommand::AddSender");
        for (node_id, _) in self.drones.iter() {
            println!("- [ Drone {} ]", node_id);
        }
        println!("Chiose: ");

        // Create input sting
        let mut input = String::new();
        // Get input from stdin
        let _ = io::stdin().read_line(&mut input);

        // Parse and verify input
        let target: NodeId;
        match input.trim_end().parse::<NodeId>() {
            Ok(value) => target = value,
            Err(e) => {
                error!(
                    "{} [ ERROR ]: Please insert a valid NodeId: {}",
                    "✗".red(),
                    e
                );
                return;
            }
        }
        // Clear input string
        input.clear();

        // Check if it exists a drone with this id
        if !self.drones.contains_key(&target) {
            error!(
                "{} [ ERROR ]: There is not a drone with the NodeId: {}",
                "✗".red(),
                target
            );
            return;
        }

        // Get drone to add
        // UI menu
        println!("Which Drone you want to add?");

        // Get the drone neighbors
        if let Some(neighbor) = self.neighbor.get(&target.clone()) {
            // for all drones
            for (node_id, _) in self.drones.iter() {
                let mut not_neighbor = true;
                // for all the drone's neighbor
                for neighbor_id in neighbor {
                    // if the drone is not the neighbor or the drone itself
                    if node_id == neighbor_id || *node_id == target {
                        not_neighbor = false
                    }
                }
                if not_neighbor {
                    // UI menu
                    println!("- [ Drone {} ]", node_id);
                }
            }
        } else {
            error!("{} [ ERROR ]: The selected drone does not exist or does not have any neighbor",
                "✗".red(),
            );
        }

        // UI menu 
        println!("Chiose: ");
        // Get input from stdin
        let _ = io::stdin().read_line(&mut input);

        // Parse and verify input
        let to_add: NodeId;
        match input.trim_end().parse::<NodeId>() {
            Ok(value) => to_add = value,
            Err(e) => {
                error!(
                    "{} [ ERROR ]: Please insert a valid NodeId: {}",
                    "✗".red(),
                    e
                );
                return;
            }
        }
        // Clear input string
        input.clear();

        // Check if it exists a neighbor with this id
        if let Some(neighbor) = self.neighbor.get(&target) {
            if neighbor.contains(&to_add) {
                error!(
                    "{} [ ERROR ]: The [ Drone: {} ] is already a neighbor of [ Drone: {} ]",
                    "✗".red(),
                    target,
                    to_add
                );
            }  
        } else {
            error!("{} [ ERROR ]: The selected [ Drone: {} ] does not exist or does not have any neighbor",
                "✗".red(),
                target,
            );
        }

        // Add drone
        // Get sender channel of the target drone
        let (_, to_add_packet_send) = self.drones.get(&to_add).unwrap().clone();
        // Get sender channel of the drone to add 
        let (_, target_packet_send) = self.drones.get(&target).unwrap().clone();
        
        // Send command
        self.handle_command(
            &target,
            DroneCommand::AddSender(to_add, to_add_packet_send.clone()),
        );
        self.handle_command(
            &to_add,
            DroneCommand::AddSender(target, target_packet_send.clone()),
        );
    }

    fn set_pdr(&mut self) {
        // Get drone to which change the pdr
        // UI menu
        println!("Witch drone would you like to send the DroneCommand::SetPackageDropRate");
        for (node_id, _) in self.drones.iter() {
            println!("- [ Drone {} ]", node_id);
        }
        println!("Chiose: ");

        // Create input sting
        let mut input = String::new();
        // Get input from stdin
        let _ = io::stdin().read_line(&mut input);

        // Parse and verify input
        let target: NodeId;
        match input.trim_end().parse::<NodeId>() {
            Ok(value) => target = value,
            Err(e) => {
                error!(
                    "{} [ ERROR ]: Please insert a valid NodeId: {}",
                    "✗".red(),
                    e
                );
                return;
            }
        }
        // Clear input string
        input.clear();

        // Check if it exists a drone with this id
        if !self.drones.contains_key(&target) {
            error!(
                "{} [ ERROR ]: There is not a drone with the NodeId: {}",
                "✗".red(),
                target
            );
            return;
        }

        // Get the new PDR
        // UI menu
        println!("Insert the desired PDR: ");
        // Get input from stdin
        let _ = io::stdin().read_line(&mut input);

        // Parse and verify input
        match input.trim_end().parse::<f32>() {
            // Check if number is in the right range
            Ok(value) if (0.0..=1.0).contains(&value) => {
                self.handle_command(&target, DroneCommand::SetPacketDropRate(value));
            }
            Ok(_) => {
                error!(
                    "{} [ ERROR ]: The PDR number is out of range. Please enter a float number between 0.00 and 1.00",
                    "✗".red(),
                );
            }
            Err(_) => {
                error!(
                    "{} [ ERROR ]: That's not a valid number. Please try again.",
                    "✗".red(),
                );
            }
        }
        input.clear();
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
        match input.trim_end().parse::<i32>() {
            Ok(value) => number = value,
            Err(e) => {
                error!(
                    "{} [ ERROR ]: Please insert a valid value: {}",
                    "✗".red(),
                    e
                );
                return;
            }
        }
        // Clear input string
        input.clear();

        // Handle chiose
        match number {
            0 => self.spawn(),
            1 => self.crash(),
            2 => self.remove_sender(),
            3 => self.add_sender(),
            4 => self.set_pdr(),
            5 => self.print(),
            6 => info!("{} None selected", "✓".green()),
            _ => error!("{} [ ERROR ]: Select a number between 0 and 6", "✗".red()),
        }
    }

    fn client_action_handler(&mut self) {

    }

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
                error!(
                    "{} [ ERROR ]: Please insert a valid value: {}",
                    "✗".red(),
                    e
                );
                return;
            }
        }
        // Clear input string
        input.clear();
        
        // Handle chiose
        match category {
            0 => self.drone_action_handler(),
            1 => self.client_action_handler(),
            _ => error!("{} [ ERROR ]: The number must be either 0 or 1", "✗".red())
        }
    }

    pub fn run(&mut self) {
        // Start loop
        loop {
            // Check for action to perform
            self.ask_action();

            // Check if any events are received
            match self.receiver.try_recv() {
                Ok(drone_event) => {
                    self.handle_event(drone_event);
                }
                Err(e) => match e {
                    crossbeam_channel::TryRecvError::Empty => continue,
                    crossbeam_channel::TryRecvError::Disconnected => error!(
                        "{} [ Simulation Controller ]: DroneEvent receiver channel disconnected: {}",
                        "✗".red(),
                        e
                    )
                },
            }
        }
    }

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
                            PacketType::MsgFragment(_) => {
                                panic!("Impossible how the hell did u do this")
                            }
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

    fn handle_command(&mut self, drone: &NodeId, drone_command: DroneCommand) {
        if let Some((command_channel, _)) = self.drones.get(drone) {
            match drone_command {
                DroneCommand::RemoveSender(node_id) => {
                    if let Some(vec) = self.neighbor.get_mut(drone) {
                        vec.retain(|x| *x != node_id);
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
                    } else {
                        error!(
                            "{} [ Simulation Controller ]: the [ Drone {} ] does not have any neighbor",
                            "✗".red(),
                            drone
                        );
                    }
                }
                DroneCommand::AddSender(node_id, sender) => {
                    if let Some(vec) = self.neighbor.get_mut(drone) {
                        vec.push(node_id);
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
                    } else {
                        error!(
                            "{} [ Simulation Controller ]: the [ Drone {} ] does not have any neighbor",
                            "✗".red(),
                            drone
                        );
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
                    if let Some((command_send, packet_send)) = self.drones.get(drone) {
                        let _ = drop(command_send);
                        let _ = drop(packet_send);
                    }                    

                    let drone_entry = self.drones.remove(drone);
                
                    self.neighbor.remove(drone);
                
                    if let Some((command_channel, _)) = drone_entry {
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
                        error!(
                            "{} [ Simulation Controller ]: the [ Drone {} ] was not found in the drones map",
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
