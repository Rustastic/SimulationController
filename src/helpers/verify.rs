use colored::Colorize;
use log::info;
use wg_2024::network::NodeId;

use crate::SimulationController;

use super::error::SimulationControllerError;

impl SimulationController {
    pub(super) fn check_drone_existence(
        &self,
        node_id: &NodeId,
    ) -> Result<(), SimulationControllerError> {
        if self.drones.contains_key(node_id) {
            Ok(())
        } else {
            Err(SimulationControllerError::DroneNotFound(*node_id))
        }
    }

    pub(super) fn has_neighbors<'a>(
        &self,
        drone: &NodeId,
    ) -> Result<&Vec<NodeId>, SimulationControllerError> {
        if let Some(neighbor) = self.neighbor.get(drone) {
            Ok(neighbor)
        } else {
            Err(SimulationControllerError::HasNoNeighbor(*drone))
        }
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
            Err(SimulationControllerError::IsNeighbor(*neighbor, *drone))
        } else {
            info!(
                "[ {} ]: [ Drone: {} ] is not a neighbor of [ Drone: {} ]",
                "Simulation Controller".green(),
                drone,
                neighbor
            );
            Ok(())
        }
    } else {
        // If i am hoping it is a neighbor (not = false)
        if neighbor_vec.contains(&neighbor) {
            info!(
                "[ {} ]: [ Drone: {} ] is a neighbor of [ Drone: {} ]",
                "Simulation Controller".green(),
                drone,
                neighbor
            );
            Ok(())
        } else {
            Err(SimulationControllerError::NotNeighbor(*neighbor, *drone))
        }
    }
}