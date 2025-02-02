use colored::Colorize;
use log::info;
use wg_2024::network::NodeId;

use crate::{SimulationController, SimulationControllerError};

pub fn check_drone_existence(
    sim_ctrl: &SimulationController,
    node_id: &NodeId,
) -> Result<(), SimulationControllerError> {
    if sim_ctrl.drones.contains_key(node_id) {
        Ok(())
    } else {
        Err(SimulationControllerError::DroneNotFound(*node_id))
    }
}

pub fn is_a_neighbor(
    neighbor_vec: &Vec<NodeId>,
    neighbor: &NodeId,
    drone: &NodeId,
    not: bool,
) -> Result<(), SimulationControllerError> {
    if not {
        // If i am hoping it is not a neighbor (not = true)
        if neighbor_vec.contains(&neighbor) {
            info!("[ {} ] Yes, [ Drone: {} ] is not a neighbor of [ Drone: {} ]", "Simulation Controller".green(), drone, neighbor);
            Ok(())
        } else {
            Err(SimulationControllerError::IsNeighbor(*neighbor, *drone))            
        }
    } else {
        // If i am hoping it is a neighbor (not = false)
        if neighbor_vec.contains(&neighbor) {
            Err(SimulationControllerError::NotNeighbor(*neighbor, *drone))
        } else {
            info!("[ {} ] Yes, [ Drone: {} ] is a neighbor of [ Drone: {} ]", "Simulation Controller".green(), drone, neighbor);
            Ok(())
        }
    }
}

pub fn has_neighbors<'a>(
    sim_ctrl: &'a SimulationController,
    drone: &NodeId,
) -> Result<&'a Vec<NodeId>, SimulationControllerError> {
    if let Some(neighbor) = sim_ctrl.neighbor.get(drone) {
        Ok(neighbor)
    } else {
        Err(SimulationControllerError::HasNoNeighbor(*drone))
    }
}
