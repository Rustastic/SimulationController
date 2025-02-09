use crossbeam_channel::{unbounded, Receiver, Sender};
use messages::{client_commands::{ChatClientCommand, MediaClientCommand}, server_commands::CommunicationServerCommand};
use rand::Rng;
use std::collections::HashMap;

use colored::Colorize;

use wg_2024::{
    config::Drone as ConfigDrone,
    controller::{DroneCommand, DroneEvent},
    drone::Drone,
    network::NodeId,
    packet::Packet,
};

use crate::{error::SimulationControllerError, verify, SimulationController};

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
    Box::new(|sim_ctrl, drone, event_send, command_recv, packet_recv| {
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
    })
}

pub fn spawn(
    sim_ctrl: &mut SimulationController,
    id: NodeId,
    connected_node_ids: Vec<NodeId>,
    pdr: f32,
) -> Result<(), SimulationControllerError> {
    // Check if drone with this id already exist
    if sim_ctrl.drones.contains_key(&id) {
        return Err(SimulationControllerError::DroneAlreadyExist(id));
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

    // Add drone to drone list
    sim_ctrl.drones.insert(id, (command_send, packet_send));

    // Add drone to neighbor list
    sim_ctrl
        .neighbor
        .insert(drone.id, drone.connected_node_ids.clone());

    // add to neighbor list of neighbor
    for neighbor_id in drone.connected_node_ids.clone() {
        sim_ctrl
            .neighbor
            .get_mut(&neighbor_id)
            .unwrap()
            .push(drone.id);
    }

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

    Ok(())
}

pub fn crash(
    sim_ctrl: &mut SimulationController,
    node_id: NodeId,
) -> Result<(), SimulationControllerError> {
    if let Err(e) = verify::check_drone_existence(&sim_ctrl, &node_id) {
        return Err(e);
    }

    // If the drone has any neighbors
    if let Some(neighbor_ids) = sim_ctrl.neighbor.get(&node_id).cloned() {
        for neighbor in neighbor_ids {
            if sim_ctrl.drones.contains_key(&neighbor) {
                sim_ctrl.handle_drone_command(&neighbor, DroneCommand::RemoveSender(node_id));
            } else if sim_ctrl.cclients.contains_key(&neighbor) {
                sim_ctrl
                    .handle_cclient_command(&neighbor, ChatClientCommand::RemoveSender(node_id));
            } else if sim_ctrl.mclients.contains_key(&neighbor) {
                sim_ctrl
                    .handle_mclient_command(&neighbor, MediaClientCommand::RemoveSender(node_id));
            } else {
                sim_ctrl
                    .handle_commserver_command(&neighbor, CommunicationServerCommand::RemoveSender(node_id));
            }
        }
    }

    Ok(())
}

pub fn remove_sender(
    sim_ctrl: &mut SimulationController,
    node_id: &NodeId,
    to_remove: &NodeId,
) -> Result<(), SimulationControllerError> {
    // Check if it exists a neighbor with this id
    match verify::has_neighbors(sim_ctrl, &node_id) {
        Ok(neighbor) => match verify::is_a_neighbor(neighbor, to_remove, node_id, false) {
            Ok(()) => {
                if sim_ctrl.drones.contains_key(&to_remove) {
                    sim_ctrl.handle_drone_command(
                        &to_remove,
                        DroneCommand::RemoveSender(*node_id),
                    );
                    return Ok(());
                } else if sim_ctrl.cclients.contains_key(&to_remove) {
                    sim_ctrl.handle_cclient_command(
                        &to_remove,
                        ChatClientCommand::RemoveSender(*node_id),
                    );
                    return Ok(());
                } else if sim_ctrl.mclients.contains_key(&to_remove) {
                    sim_ctrl.handle_mclient_command(
                        &to_remove,
                        MediaClientCommand::RemoveSender(*node_id),
                    );
                    return Ok(());
                } else if sim_ctrl.comm_servers.contains_key(&to_remove) {
                    println!("\nremoving as to_remove\n");

                    sim_ctrl.handle_commserver_command(
                        &to_remove,
                        CommunicationServerCommand::RemoveSender(*node_id),
                    );
                    return Ok(());
                }
                Err(SimulationControllerError::ClientOnClient)
            }
            Err(e) => Err(e),
        },
        Err(e) => Err(e),
    }
}

pub fn add_sender(
    sim_ctrl: &mut SimulationController,
    node_id: &NodeId,
    to_add: &NodeId,
) -> Result<(), SimulationControllerError> {
    // Check if it exists a neighbor with this id
    match verify::has_neighbors(sim_ctrl, &node_id) {
        Ok(neighbor) => match verify::is_a_neighbor(neighbor, &to_add, &node_id, true) {
            Ok(()) => {
                let packet_send;
                if sim_ctrl.drones.contains_key(node_id) {
                    (_, packet_send) = sim_ctrl.drones.get(&node_id).unwrap().clone()
                } else if sim_ctrl.comm_servers.contains_key(node_id) {
                    (_, packet_send) = sim_ctrl.comm_servers.get(&node_id).unwrap().clone();
                } else if sim_ctrl.mclients.contains_key(node_id){
                    (_, packet_send) = sim_ctrl.mclients.get(&node_id).unwrap().clone();
                } else /*if sim_ctrl.cclients.contains_key(node_id)*/ {
                    (_, packet_send) = sim_ctrl.cclients.get(&node_id).unwrap().clone();
                }

                if sim_ctrl.drones.contains_key(to_add) {
                    sim_ctrl.handle_drone_command(
                        &to_add,
                        DroneCommand::AddSender(*node_id, packet_send.clone()),
                    );

                    return Ok(());
                } else if sim_ctrl.comm_servers.contains_key(to_add) {
                    println!("\nadding as to_add\n");
                    sim_ctrl.handle_commserver_command(
                        &to_add,
                        CommunicationServerCommand::AddSender(*node_id, packet_send.clone()),
                    );

                    return Ok(());
                } else if sim_ctrl.cclients.contains_key(to_add) {
                    sim_ctrl.handle_cclient_command(
                        &to_add,
                        ChatClientCommand::AddSender(*node_id, packet_send.clone()),
                    );

                    return Ok(());
                } else if sim_ctrl.mclients.contains_key(to_add) {
                    sim_ctrl.handle_mclient_command(
                        &to_add,
                        MediaClientCommand::AddSender(*node_id, packet_send.clone())
                    );

                    return Ok(());
                }

                Err(SimulationControllerError::ClientOnClient)
            }
            Err(e) => Err(e),
        },
        Err(e) => Err(e),
    }
}

pub fn send_message(
    sim_ctrl: &mut SimulationController,
    src: &NodeId,
    dest: &NodeId,
) -> Result<(), SimulationControllerError> {
    // verify nodes connected
    Ok(())
}

pub fn register(
    sim_ctrl: &mut SimulationController,
    client: &NodeId,
    server: &NodeId,
) -> Result<(), SimulationControllerError> {
    // verify it is really a server
    Ok(())
}

pub fn logout(
    sim_ctrl: &mut SimulationController,
    client: &NodeId,
    server: &NodeId,
) -> Result<(), SimulationControllerError> {
    // verify it is a server and it is the one it is connected to
    Ok(())
}
