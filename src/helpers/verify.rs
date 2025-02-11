use colored::Colorize;
use log::info;
use wg_2024::network::NodeId;

use crate::SimulationController;

use super::types::SimCtrlError;

impl SimulationController {
    pub(super) fn check_drone_existence(&self, node_id: NodeId) -> Result<(), SimCtrlError> {
        if self.drones.contains_key(&node_id) {
            Ok(())
        } else {
            Err(SimCtrlError::DroneNotFound(node_id))
        }
    }

    pub(super) fn has_neighbors(&self, drone: NodeId) -> Result<&Vec<NodeId>, SimCtrlError> {
        if let Some(neighbor) = self.neighbor.get(&drone) {
            Ok(neighbor)
        } else {
            Err(SimCtrlError::HasNoNeighbor(drone))
        }
    }
}

pub fn is_a_neighbor(
    neighbor_vec: &[NodeId],
    neighbor: NodeId,
    drone: NodeId,
    not: bool,
) -> Result<(), SimCtrlError> {
    if not {
        // If i am hoping it is not a neighbor (not = true)
        if neighbor_vec.contains(&neighbor) {
            Err(SimCtrlError::IsNeighbor(neighbor, drone))
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
            Err(SimCtrlError::NotNeighbor(neighbor, drone))
        }
    }
}
