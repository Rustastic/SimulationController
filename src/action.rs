use crossbeam_channel::{unbounded, Receiver, Sender};
use rand::Rng;
use std::{collections::HashMap, io};

use wg_2024::{
    config::Drone as ConfigDrone,
    controller::{DroneCommand, DroneEvent},
    drone::Drone,
    network::NodeId,
    packet::Packet,
};

use crate::{user_interaction, verify, SimulationController};

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

pub fn print(sim_ctrl: &SimulationController) {
    for (drone, _) in sim_ctrl.drones.iter() {
        println!("\n[ Drone: {} ]", drone);

        println!("Neighbors:");
        let neighbor_ids = sim_ctrl.neighbor.get(drone).unwrap();
        for neighbor in neighbor_ids {
            println!("\t[ Drone: {} ]", neighbor)
        }
    }
}

pub fn spawn(sim_ctrl: &mut SimulationController) {
    // Get ID of the new drone
    // UI menu
    println!("Please provide the necessary parameters for the new drone\n");
    println!("Id: Choose a value between 0 and 255 that will become the Id of the new drone");
    println!("Be careful that id doesn't match the one of an existing drone");
    let prompt = "See the following list of existing drone".to_string();
    user_interaction::print_drones(sim_ctrl, prompt);

    // Create input sting
    let mut input = String::new();
    // Get input from stdin
    let _ = io::stdin().read_line(&mut input);

    // Parse and verify input
    let id: NodeId;
    match user_interaction::parse_and_verify(&mut input) {
        Ok(node_id) => id = node_id,
        Err(e) => {
            println!("{}", e);
            return;
        }
    }

    // Check if drone with this id already exist
    if sim_ctrl.drones.contains_key(&id) {
        eprintln!("[ ERROR ]: A drone with the NodeId: {} already exists", id);
        return;
    }

    // Get all the neighbors
    // Vec containing the neighbor drones
    let mut connected_node_ids = Vec::<NodeId>::new();

    let prompt =
        "Neighbors: Choose the drones that will be the neighbors of the new drone".to_string();
    user_interaction::print_drones(sim_ctrl, prompt);

    // loop to get all the neighbors
    let mut add = true;
    while add {
        // UI menu
        println!("Connected Drones Id: ");
        // Get input from stdin
        let _ = io::stdin().read_line(&mut input);

        // Parse and verify input
        let new_neighbor: NodeId;
        match user_interaction::parse_and_verify(&mut input) {
            Ok(node_id) => new_neighbor = node_id,
            Err(e) => {
                println!("{}", e);
                return;
            }
        }

        // Add drone to the neighbor vec
        if !connected_node_ids.contains(&new_neighbor) {
            connected_node_ids.push(new_neighbor);
        } else {
            eprintln!(
                "[ ERROR ]: The [ Drone: {} ] is already a neighbor",
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
                    println!("Write the number corresponding to the chosen option");
                    add = true;
                } else {
                    add = false;
                }
            }
            Err(e) => {
                eprintln!("[ ERROR ]: Please insert a valid value: {}", e);
                return;
            }
        }
        //clear input string
        input.clear();
    }

    // Get drone pdr
    // UI menu
    println!("Packet Drop Rate: Choose a value between 0.00 and 1.00 that will become the PDR of the new drone");
    // Get input from stdin
    let _ = io::stdin().read_line(&mut input);

    // Parse and verify input
    let pdr: f32;
    match user_interaction::pdr_parse_and_verify(&mut input) {
        Ok(value) => pdr = value,
        Err(e) => {
            println!("{}", e);
            return;
        }
    }

    // Create new drone
    let drone = ConfigDrone {
        id,
        connected_node_ids,
        pdr,
    };

    // Add drone to neighbor list
    sim_ctrl
        .neighbor
        .insert(drone.id, drone.connected_node_ids.clone());

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
            &sim_ctrl.event_send,
            &command_recv,
            &packet_recv,
            &packet_send,
        );

        // Add drone to drone list
        sim_ctrl.drones.insert(id, (command_send, packet_send));
    } else {
        panic!("No factory defined for [ Drone {} ]", drone.id);
    }
}

pub fn crash(sim_ctrl: &mut SimulationController) {
    // Get drone to crash
    // UI menu
    let prompt = "Witch drone would you like to send the DroneCommand::Crash".to_string();
    user_interaction::print_drones(&sim_ctrl, prompt);

    // Create input sting
    let mut input = String::new();
    // Get input from stdin
    let _ = io::stdin().read_line(&mut input);

    // Parse and verify input
    let target: NodeId;
    match user_interaction::parse_and_verify(&mut input) {
        Ok(node_id) => target = node_id,
        Err(e) => {
            println!("{}", e);
            return;
        }
    }

    if let Err(e) = verify::check_drone_existence(&sim_ctrl, &target) {
        println!("{}", e);
        return;
    }

    // If the drone has any neighbors
    if let Some(neighbor_ids) = sim_ctrl.neighbor.get_mut(&target).cloned() {
        for neighbor in neighbor_ids {
            // Send command to neighbors
            sim_ctrl.handle_command(&neighbor, DroneCommand::RemoveSender(target));
        }
    }

    // Send command
    sim_ctrl.handle_command(&target, DroneCommand::Crash);
}

pub fn remove_sender(sim_ctrl: &mut SimulationController) {
    // Get drone to which remove a sender
    // UI menu
    let prompt = "Witch drone would you like to send the DroneCommand::RemoveSender".to_string();
    user_interaction::print_drones(sim_ctrl, prompt);

    // Create input sting
    let mut input = String::new();
    // Get input from stdin
    let _ = io::stdin().read_line(&mut input);

    // Parse and verify input
    let target: NodeId;
    match user_interaction::parse_and_verify(&mut input) {
        Ok(node_id) => target = node_id,
        Err(e) => {
            println!("{}", e);
            return;
        }
    };

    // Check if it exists a drone with this id
    if let Err(e) = verify::check_drone_existence(&sim_ctrl, &target) {
        println!("{}", e);
        return;
    }

    // Get the neighbor to remove
    // UI menu
    let prompt = "Which of his neighbor would u like to remove?".to_string();
    user_interaction::print_neighbor(sim_ctrl, prompt, &target);

    // Get input from stdin
    let _ = io::stdin().read_line(&mut input);

    // Parse and verify input
    let to_remove: NodeId;
    match user_interaction::parse_and_verify(&mut input) {
        Ok(node_id) => to_remove = node_id,
        Err(e) => {
            println!("{}", e);
            return;
        }
    }

    // Check if it exists a neighbor with this id
    match verify::has_neighbors(sim_ctrl, &target) {
        Ok(neighbor) => match verify::is_a_neighbor(neighbor, &to_remove, &target, false) {
            Ok(()) => (),
            Err(e) => {
                println!("{}", e);
                return;
            }
        },
        Err(e) => {
            println!("{}", e);
            return;
        }
    }

    // Send command
    sim_ctrl.handle_command(&target, DroneCommand::RemoveSender(to_remove));
    sim_ctrl.handle_command(&to_remove, DroneCommand::RemoveSender(target));
}

pub fn add_sender(sim_ctrl: &mut SimulationController) {
    // Get drone to which add a sender
    // UI menu
    let prompt = "Witch drone would you like to send the DroneCommand::AddSender".to_string();
    user_interaction::print_drones(sim_ctrl, prompt);

    // Create input sting
    let mut input = String::new();
    // Get input from stdin
    let _ = io::stdin().read_line(&mut input);

    // Parse and verify input
    let target: NodeId;
    match user_interaction::parse_and_verify(&mut input) {
        Ok(node_id) => target = node_id,
        Err(e) => {
            println!("{}", e);
            return;
        }
    }

    if let Err(e) = verify::check_drone_existence(&sim_ctrl, &target) {
        println!("{}", e);
        return;
    }

    // Get drone to add
    // UI menu
    println!("Which Drone you want to add?");
    // Get the drone neighbors
    match verify::has_neighbors(sim_ctrl, &target) {
        Ok(neighbor) => {
            // for all drones
            for (node_id, _) in sim_ctrl.drones.iter() {
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
        }
        Err(e) => {
            println!("{}", e);
            return;
        }
    }

    // UI menu
    println!("Write the number corresponding to the chosen option");
    // Get input from stdin
    let _ = io::stdin().read_line(&mut input);

    // Parse and verify input
    let to_add: NodeId;
    match user_interaction::parse_and_verify(&mut input) {
        Ok(node_id) => to_add = node_id,
        Err(e) => {
            println!("{}", e);
            return;
        }
    }

    // Check if it exists a neighbor with this id
    match verify::has_neighbors(sim_ctrl, &target) {
        Ok(neighbor) => match verify::is_a_neighbor(neighbor, &to_add, &target, true) {
            Ok(()) => (),
            Err(e) => {
                println!("{}", e);
                return;
            }
        },
        Err(e) => {
            println!("{}", e);
            return;
        }
    }

    // Add drone
    // Get sender channel of the target drone
    let (_, to_add_packet_send) = sim_ctrl.drones.get(&to_add).unwrap().clone();
    // Get sender channel of the drone to add
    let (_, target_packet_send) = sim_ctrl.drones.get(&target).unwrap().clone();

    // Send command
    sim_ctrl.handle_command(
        &target,
        DroneCommand::AddSender(to_add, to_add_packet_send.clone()),
    );
    sim_ctrl.handle_command(
        &to_add,
        DroneCommand::AddSender(target, target_packet_send.clone()),
    );
}

pub fn set_pdr(sim_ctrl: &mut SimulationController) {
    // Get drone to which change the pdr
    // UI menu
    let prompt =
        "Witch drone would you like to send the DroneCommand::SetPackageDropRate".to_string();
    user_interaction::print_drones(sim_ctrl, prompt);

    // Create input sting
    let mut input = String::new();
    // Get input from stdin
    let _ = io::stdin().read_line(&mut input);

    // Parse and verify input
    let target: NodeId;
    match user_interaction::parse_and_verify(&mut input) {
        Ok(node_id) => target = node_id,
        Err(e) => {
            println!("{}", e);
            return;
        }
    }

    if let Err(e) = verify::check_drone_existence(&sim_ctrl, &target) {
        println!("{}", e);
        return;
    }

    // Get the new PDR
    // UI menu
    println!("Insert the desired PDR: ");
    // Get input from stdin
    let _ = io::stdin().read_line(&mut input);

    // Parse and verify input
    match user_interaction::pdr_parse_and_verify(&mut input) {
        Ok(value) => sim_ctrl.handle_command(&target, DroneCommand::SetPacketDropRate(value)),
        Err(e) => {
            println!("{}", e);
            return;
        }
    }
}
