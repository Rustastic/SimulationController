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
    if neighbor_vec.contains(&neighbor) {
        if not {
            Err(SimulationControllerError::NotNeighbor(*neighbor, *drone))
        } else {
            Ok(())
        }
    } else {
        if not {
            Ok(())
        } else {
            Err(SimulationControllerError::AlreadyNeighbor(
                *neighbor, *drone,
            ))
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
