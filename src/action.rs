use crossbeam_channel::{unbounded, Receiver, Sender};
use rand::Rng;
use std::collections::HashMap;

use colored::Colorize;
use log::error;

use wg_2024::{
    config::Drone as ConfigDrone,
    controller::{DroneCommand, DroneEvent},
    drone::Drone,
    network::NodeId,
    packet::Packet,
};

use crate::{verify, SimulationController};

fn drone_factory<T>() -> Box<
    dyn Fn(
        &mut SimulationController,
        &ConfigDrone,
        &Sender<DroneEvent>,
        &Receiver<DroneCommand>,
        &Receiver<Packet>,
    ) -> Box<dyn Drone>,
>
where
    T: Drone + 'static,
{
    Box::new(
        |sim_ctrl, drone, event_send, command_recv, packet_recv| {
            // Create packet send hashmap
            let mut packet_send_hashmap = HashMap::<NodeId, Sender<Packet>>::new();
            // Fill hashmap with only neighbor
            for neighbor in &drone.connected_node_ids {
                let (_, neighbor_send_channel) = sim_ctrl.drones.get(neighbor).unwrap();
                packet_send_hashmap.insert(*neighbor, neighbor_send_channel.clone());
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


pub fn spawn(
    sim_ctrl: &mut SimulationController,
    id: NodeId,
    connected_node_ids: Vec<NodeId>,
    pdr: f32,
) {
    // Check if drone with this id already exist
    if sim_ctrl.drones.contains_key(&id) {
        error!("[ ERROR ]: A drone with the NodeId: {} already exists", id);
        return;
    }

    // Create new drone
    let drone = ConfigDrone {
        id,
        connected_node_ids,
        pdr,
    };

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

    // Add drone to neighbor list
    sim_ctrl
        .neighbor
        .insert(drone.id, drone.connected_node_ids.clone());

    // add to neighbor list of neighbor
    for neighbor_id in drone.connected_node_ids.clone() {
        sim_ctrl.neighbor.get_mut(&neighbor_id).unwrap().push(drone.id);
    }

    // Add drone to drone list
    sim_ctrl.drones.insert(id, (command_send, packet_send));

    // Crate drone
    if let Some(factory) = drone_factories.get(rand) {
        let new_drone = factory(
            sim_ctrl,
            &drone,
            &sim_ctrl.event_send.clone(),
            &command_recv.clone(),
            &packet_recv.clone(),
        );

        sim_ctrl.new_drones.push(new_drone);
    } else {
        panic!(
            "[ {} ]: No factory defined for [ Drone {} ]",
            "Simulation Controller".red(),
            drone.id
        );
    }
}

pub fn crash(sim_ctrl: &mut SimulationController, target: NodeId) {
    if let Err(e) = verify::check_drone_existence(&sim_ctrl, &target) {
        panic!("{}", e);
    }

    // If the drone has any neighbors
    if let Some(neighbor_ids) = sim_ctrl.neighbor.get_mut(&target).cloned() {
        for neighbor in neighbor_ids {
            // Send command to neighbors
            sim_ctrl.handle_command(&neighbor, DroneCommand::RemoveSender(target));
        }
    }
}

pub fn remove_sender(sim_ctrl: &mut SimulationController, drone: &NodeId, to_remove: &NodeId) {
    // Check if it exists a neighbor with this id
    match verify::has_neighbors(sim_ctrl, &drone) {
        Ok(neighbor) => match verify::is_a_neighbor(neighbor, to_remove, drone, false) {
            Ok(()) => sim_ctrl.handle_command(&to_remove, DroneCommand::RemoveSender(*drone)),
            Err(e) => {
                panic!("{}", e);
            }
        },
        Err(e) => {
            panic!("{}", e);
        }
    }
}

pub fn add_sender(sim_ctrl: &mut SimulationController, drone: &NodeId, to_add: &NodeId) {
    // Check if it exists a neighbor with this id
    match verify::has_neighbors(sim_ctrl, &drone) {
        Ok(neighbor) => match verify::is_a_neighbor(neighbor, &to_add, &drone, true) {
            Ok(()) => {
                let (_, drone_packet_send) = sim_ctrl.drones.get(&drone).unwrap().clone();

                sim_ctrl.handle_command(
                    &to_add,
                    DroneCommand::AddSender(*drone, drone_packet_send.clone()),
                );
            }
            Err(e) => {
                panic!("{}", e);
            }
        },
        Err(e) => {
            panic!("{}", e);
        }
    }
}
/*
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
}*/
