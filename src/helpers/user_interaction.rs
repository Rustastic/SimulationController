use std::str::FromStr;

use wg_2024::network::NodeId;

use crate::{error::SimulationControllerError, verify, SimulationController};

pub fn parse_and_verify<T>(input: &mut String) -> Result<T, SimulationControllerError>
where
    T: FromStr,
{
    let to_parse = input.clone();
    input.clear();
    match to_parse.trim_end().parse::<T>() {
        Ok(value) => return Ok(value),
        Err(_) => Err(SimulationControllerError::InvalidNodeId),
    }
}

pub fn pdr_parse_and_verify(input: &mut String) -> Result<f32, SimulationControllerError> {
    let to_parse = input.clone();
    input.clear();
    match to_parse.trim_end().parse::<f32>() {
        Ok(value) if (0.0..=1.0).contains(&value) => Ok(value),
        Ok(_) => Err(SimulationControllerError::PacketDropRateOutOfRange),
        Err(e) => Err(SimulationControllerError::InvalidPacketDropRate(e)),
    }
}

pub fn print_drones(sim_ctrl: &SimulationController, prompt: String) {
    println!("{}", prompt);
    for (node_id, _) in sim_ctrl.drones.iter() {
        println!("- [ Drone {} ]", node_id)
    }
    println!("Write the number corresponding to the chosen option");
}

pub fn print_neighbor(sim_ctrl: &SimulationController, prompt: String, drone: &NodeId) {
    match verify::has_neighbors(sim_ctrl, drone) {
        Ok(neighbors) => {
            println!("{}", prompt);
            for mode_id in neighbors {
                println!("- [ Drone {} ]", mode_id)
            }
        }
        Err(e) => {
            println!("{}", e);
            return;
        }
    }

    println!("Write the number corresponding to the chosen option");
}
