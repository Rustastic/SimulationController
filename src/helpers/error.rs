use thiserror::Error;

use wg_2024::network::NodeId;

#[derive(Debug, Error)]
pub enum Error {
    #[error("A drone with the NodeId: {0} does not exist")]
    DroneNotFound(NodeId),
    #[error("The [ Drone: {0} ] is NOT a neighbor of [ Drone: {1} ]")]
    NotNeighbor(NodeId, NodeId),
    #[error("The [ Drone: {0} ] is a neighbor of [ Drone: {1} ]")]
    IsNeighbor(NodeId, NodeId),
    #[error("The [ Drone: {0} ] doesn't have any neighbor")]
    HasNoNeighbor(NodeId),
    #[error("Can't connect two clients together")]
    ClientOnClient,
    #[error("A Client must be connected to at least 1 nodes")]
    ClientRemove,
    #[error("A Server must be connected to at least 2 nodes")]
    ServerRemove,
    #[error("A Drone must be connected to at least 1 nodes")]
    DroneRemove,
    #[error("A Client can be connected to maximum of 2 nodes")]
    ClientAdd,
    #[error("No factory defined for [ Drone {0} ]")]
    MissingFactory(NodeId),
    #[error("failed to find a Sender<Packet> channel for the [ Node {0} ]")]
    NoSender(NodeId),
}
