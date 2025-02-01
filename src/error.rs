use std::num::ParseFloatError;

use thiserror::Error;

use wg_2024::network::NodeId;

#[derive(Debug, Error)]
pub enum SimulationControllerError {
    #[error("[ ERROR ]: A drone with the NodeId: {0} does not exist")]
    DroneNotFound(NodeId),
    #[error("[ ERROR ]: Please insert a valid NodeId")]
    InvalidNodeId,
    #[error("[ ERROR ]: The [ Drone: {0} ] is NOT a neighbor of [ Drone: {1} ]")]
    NotNeighbor(NodeId, NodeId),
    #[error("[ ERROR ]: The [ Drone: {0} ] is a neighbor of [ Drone: {1} ]")]
    IsNeighbor(NodeId, NodeId),
    #[error("[ ERROR ]: The [ Drone: {0} ] doesn't have any neighbor")]
    HasNoNeighbor(NodeId),
    #[error(
        "[ ERROR ]: The PDR number is out of range. Please enter a number between 0.00 and 1.00"
    )]
    PacketDropRateOutOfRange,
    #[error("[ ERROR ]: Please insert a valid f32 value: {0}")]
    InvalidPacketDropRate(ParseFloatError),
}
