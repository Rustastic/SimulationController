use crossbeam_channel::{unbounded, Receiver, Sender};
use rand::Rng;
use std::{collections::HashMap, io};
use toml::value;

use colored::Colorize;
use log::{error, info, warn};

use wg_2024::{
    config::Drone as ConfigDrone,
    controller::{DroneCommand, DroneEvent},
    drone::Drone,
    network::NodeId,
    packet::{Packet, PacketType},
};

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

    fn ask_action(&mut self) {
        println!("Select the action to execute:");
        println!("0 - Spawn");
        println!("1 - Crash");
        println!("2 - RemoveSender");
        println!("3 - AddSender");
        println!("4 - SetPackageDropRate");
        println!("5 - None");
        println!("\nChiose: ");

        let mut input = String::new();
        let _ = io::stdin().read_line(&mut input);
        let number: i32;
        match input.trim_end().parse() {
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
        input.clear();

        match number {
            0 => {
                println!("Please provide the necessary parameters for ne new drone");
                println!("Id: ");
                let _ = io::stdin().read_line(&mut input);
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
                input.clear();

                println!("Connected Drones Id: ");
                let mut connected_node_ids = Vec::<NodeId>::new();
                let mut add = true;
                while add {
                    println!("Connected Drones Id: ");
                    let _ = io::stdin().read_line(&mut input);
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
                    input.clear();

                    if !connected_node_ids.contains(&new_neighbor) {
                        connected_node_ids.push(new_neighbor);
                    } else {
                        error!(
                            "{} [ ERROR ]: The [ Drone: {} ] is already a neighbor",
                            "✗".red(),
                            new_neighbor
                        );
                    }

                    println!("Do you want to add another Drone to the neighbor list? 0->No 1->Yes");
                    let _ = io::stdin().read_line(&mut input);
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
                    input.clear();
                }

                println!("PDR: ");
                let _ = io::stdin().read_line(&mut input);
                let pdr: f32;
                match input.trim_end().parse::<f32>() {
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
                input.clear();

                let drone = ConfigDrone {
                    id,
                    connected_node_ids,
                    pdr,
                };

                let rand = rand::thread_rng().gen_range(0..10);

                let drone_factories = vec![
                    drone_factory::<rusty_drones::RustyDrone>(),
                    drone_factory::<LeDron_James::Drone>(),
                    //drone_factory::<dr_ones::Drone>(),
                    drone_factory::<skylink::SkyLinkDrone>(),
                    drone_factory::<rustbusters_drone::RustBustersDrone>(),
                    drone_factory::<rust_roveri::RustRoveri>(),
                    drone_factory::<rust_do_it::RustDoIt>(),
                    drone_factory::<wg_2024_rust::drone::RustDrone>(),
                    drone_factory::<null_pointer_drone::MyDrone>(),
                    drone_factory::<null_pointer_drone::MyDrone>(),
                    drone_factory::<null_pointer_drone::MyDrone>(),
                    //create_factory::<lockheedrustin_drone::LockheedRustin>(),
                ];

                let (command_send, command_recv) = unbounded::<DroneCommand>();
                let (packet_send, packet_recv) = unbounded::<Packet>();
                if let Some(factory) = drone_factories.get(rand) {
                    let new_drone = factory(
                        &drone,
                        &self.event_send,
                        &command_recv,
                        &packet_recv,
                        &packet_send,
                    );

                    // Fill drone Hashmap
                    self.drones.insert(id, (command_send, packet_send));
                } else {
                    panic!("No factory defined for [ Drone {} ]", drone.id);
                }
            }
            1 => {
                println!("Witch drone would you like to send the DroneCommand::Crash");

                for (node_id, _) in self.drones.iter() {
                    println!("- [ Drone {} ]", node_id)
                }

                println!("Chiose: ");
                let _ = io::stdin().read_line(&mut input);
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
                input.clear();

                if let Some(neighbor_ids) = self.neighbor.get_mut(&target).cloned() {
                    for neighbor in  neighbor_ids{
                        self.handle_command(&neighbor, DroneCommand::RemoveSender(target));
                    }
                }

                self.handle_command(&target, DroneCommand::Crash);
            }
            2 => {
                println!("Witch drone would you like to send the DroneCommand::RemoveSender");

                for (node_id, _) in self.drones.iter() {
                    println!("- [ Drone {} ]", node_id);
                }

                println!("Chiose: ");
                let _ = io::stdin().read_line(&mut input);
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
                input.clear();

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
                let _ = io::stdin().read_line(&mut input);
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
                input.clear();

                self.handle_command(&target, DroneCommand::RemoveSender(to_remove));
                self.handle_command(&to_remove, DroneCommand::RemoveSender(target));
            }
            3 => {
                println!("Witch drone would you like to send the DroneCommand::AddSender");

                for (node_id, _) in self.drones.iter() {
                    println!("- [ Drone {} ]", node_id);
                }

                println!("Chiose: ");
                let _ = io::stdin().read_line(&mut input);
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
                input.clear();

                println!("Which Drone you want to add?");
                if let Some(neighbor) = self.neighbor.get(&target.clone()) {
                    for (node_id, _) in self.drones.iter() {
                        let mut not_neighbor = true;
                        for neighbor_id in neighbor {
                            if node_id == neighbor_id || *node_id == target {
                                not_neighbor = false
                            }
                        }
                        if not_neighbor {
                            println!("- [ Drone {} ]", node_id);
                        }
                    }
                } else {
                    error!("{} [ ERROR ]: The selected drone does not exist or does not have any neighbor",
                        "✗".red(),
                    );
                }

                println!("Chiose: ");
                let _ = io::stdin().read_line(&mut input);
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
                input.clear();

                // get sender channel of to_add
                let (_, target_packet_send) = self.drones.get(&target).unwrap().clone();
                // get sender channel of target
                let (_, to_add_packet_send) = self.drones.get(&to_add).unwrap().clone();
                // send command
                self.handle_command(
                    &target,
                    DroneCommand::AddSender(to_add, to_add_packet_send.clone()),
                );

                self.handle_command(
                    &to_add,
                    DroneCommand::AddSender(target, target_packet_send.clone()),
                );

                // add new neighbor to target
                let target_neighbor = self.neighbor.get_mut(&target).unwrap();
                target_neighbor.push(to_add);

                // add new neighbor to to_add
                let to_add_neighbor = self.neighbor.get_mut(&to_add).unwrap();
                to_add_neighbor.push(target);
            }
            4 => {
                println!("Witch drone would you like to send the DroneCommand::SetPackageDropRate");

                for (node_id, _) in self.drones.iter() {
                    println!("- [ Drone {} ]", node_id);
                }

                println!("Chiose: ");
                let _ = io::stdin().read_line(&mut input);
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
                input.clear();

                println!("Insert the desired PDR: ");
                let _ = io::stdin().read_line(&mut input);

                match input.trim_end().parse::<f32>() {
                    Ok(value) if (0.0..=1.0).contains(&value) => {
                        let mut found: bool = false;
                        let drone_ids: Vec<NodeId> = self.drones.keys().cloned().collect();
                        for node_id in drone_ids {
                            if node_id == target {
                                found = true;
                                self.handle_command(
                                    &node_id,
                                    DroneCommand::SetPacketDropRate(value),
                                );
                            }
                        }

                        if !found {
                            error!(
                                "{} [ ERROR ]: There is no drones with the provided NodeIds",
                                "✗".red(),
                            );
                        }
                    }
                    Ok(_) => {
                        error!(
                            "{} [ ERROR ]: The PDR number is out of range. Please enter a number between 0 and 1.",
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
            5 => info!("{} None selected", "✓".green()),
            _ => error!("{} [ ERROR ]: Select a number between 0 and 5", "✗".red(),),
        }
    }

    pub fn run(&mut self) {
        loop {
            self.ask_action();

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

                    self.neighbor
                        .iter()
                        .position(|(x, _)| x == drone)
                        .map(|x| self.neighbor.remove(&(x as NodeId)));
                    self.drones
                        .iter()
                        .position(|(x, _)| x == drone)
                        .map(|x| {
                            self.neighbor.remove(&(x as NodeId));
                        });

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
