use colored::Colorize;
use log::info;
use wg_2024::{network::NodeId, packet::NodeType};

use crate::SimulationController;

use super::error::Error;

impl SimulationController {
    #[allow(clippy::missing_errors_doc)]
    pub fn check_drone_existence(&self, drone_id: NodeId) -> Result<(), Error> {
        if self.drones.contains_key(&drone_id) {
            Ok(())
        } else {
            Err(Error::DroneNotFound(drone_id))
        }
    }

    #[allow(clippy::missing_errors_doc)]
    pub fn has_neighbors(&self, node_id: NodeId) -> Result<&Vec<NodeId>, Error> {
        if let Some(neighbor) = self.neighbor.get(&node_id) {
            Ok(neighbor)
        } else {
            Err(Error::HasNoNeighbor(node_id))
        }
    }

    #[allow(clippy::missing_errors_doc)]
    #[must_use]
    pub fn get_node_type(&self, node_id: NodeId) -> NodeType {
        if self.drones.contains_key(&node_id) {
            NodeType::Drone
        } else if self.cclients.contains_key(&node_id) || self.mclients.contains_key(&node_id) {
            NodeType::Client
        } else {
            NodeType::Server
        }
    }

    #[allow(clippy::missing_errors_doc)]
    pub fn check_remove(&self, node_id: NodeId) -> Result<(), Error> {
        let node_type = self.get_node_type(node_id);
        match node_type {
            NodeType::Client => match self.has_neighbors(node_id) {
                Ok(vec) => {
                    if vec.len() == 2 {
                        Ok(())
                    } else {
                        Err(Error::ClientRemove)
                    }
                }
                Err(e) => Err(e),
            },
            NodeType::Drone => match self.has_neighbors(node_id) {
                Ok(vec) => {
                    if vec.len() > 1 {
                        Ok(())
                    } else {
                        log::error!("drone {} -> neighbors {:?}", node_id, vec);
                        Err(Error::DroneRemove)
                    }
                }
                Err(e) => Err(e),
            },
            NodeType::Server => match self.has_neighbors(node_id) {
                Ok(vec) => {
                    if vec.len() > 2 {
                        Ok(())
                    } else {
                        Err(Error::ServerRemove)
                    }
                }
                Err(e) => Err(e),
            },
        }
    }

    #[allow(clippy::missing_errors_doc)]
    pub fn check_add(&self, node_id: NodeId) -> Result<(), Error> {
        let node_type = self.get_node_type(node_id);

        match node_type {
            NodeType::Client => match self.has_neighbors(node_id) {
                Ok(vec) => {
                    if vec.len() == 1 {
                        Ok(())
                    } else {
                        Err(Error::ClientAdd)
                    }
                }
                Err(e) => Err(e),
            },
            NodeType::Drone | NodeType::Server => Ok(()),
        }
    }
}

#[allow(clippy::missing_errors_doc)]
pub fn is_a_neighbor(
    neighbor_vec: &[NodeId],
    neighbor: NodeId,
    drone: NodeId,
    not: bool,
) -> Result<(), Error> {
    if not {
        // If i am hoping it is not a neighbor (not = true)
        if neighbor_vec.contains(&neighbor) {
            Err(Error::IsNeighbor(neighbor, drone))
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
            Err(Error::NotNeighbor(neighbor, drone))
        }
    }
}
