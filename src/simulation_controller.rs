use std::collections::HashMap;
use crossbeam_channel::{Receiver, Sender};

use log::{error, info, warn};
use colored::Colorize;

use wg_2024::{
    controller::{DroneCommand, DroneEvent}, network::NodeId, packet::{Packet, PacketType}
};

pub struct SimulationController {
    drones: HashMap<NodeId, (Sender<DroneCommand>, Sender<Packet>)>,
    receiver: Receiver<DroneEvent>
}

impl SimulationController {
    pub fn new(drones: HashMap<NodeId, (Sender<DroneCommand>, Sender<Packet>)>, recv: Receiver<DroneEvent>) -> Self {
        return Self {
            drones,
            receiver: recv
        };
    }

    pub fn run(&mut self) {
        match self.receiver.recv() {
            Ok(drone_event) => self.handle_event(drone_event),
            Err(_) => error!("{} Channel is closed", "✗".red()),
        }

        
    }

    fn spawn() {
        
    }

    fn handle_event(&self, drone_event: DroneEvent) {
        match drone_event {
            DroneEvent::PacketSent(packet) => {
                let src = packet.routing_header.hops.get(packet.routing_header.hop_index).unwrap();
                let dest = packet.routing_header.hops.get(packet.routing_header.hop_index + 1).unwrap();
                let pakcet_type = packet.pack_type;

                // GUI
            },
            DroneEvent::PacketDropped(packet) => {
                let drone = packet.routing_header.hops.get(packet.routing_header.hop_index).unwrap();

                // GUI
            },
            DroneEvent::ControllerShortcut(packet) => {
                if let Some(dest) = packet.routing_header.hops.get(packet.routing_header.len()) {
                    if let Some((_, packet_channel)) = self.drones.get(dest) {
                        match packet.pack_type {
                            PacketType::MsgFragment(_) => error!(""),
                            _ => {
                                packet_channel.send(packet.clone()).unwrap();
                            },
                        }
                    }
                } else {
                    error!("");
                }
            },
        }
    }

    fn handle_command(&self, drone: &NodeId, drone_command: DroneCommand) {
        if let Some((command_channel, _)) = self.drones.get(drone) {
            match drone_command {
                DroneCommand::RemoveSender(node_id) => {
                    command_channel.send(DroneCommand::RemoveSender(node_id)).unwrap();
                },
                DroneCommand::AddSender(node_id, sender) => {
                    command_channel.send(DroneCommand::AddSender(node_id, sender)).unwrap();
                },
                DroneCommand::SetPacketDropRate(pdr) => {
                    command_channel.send(DroneCommand::SetPacketDropRate(pdr)).unwrap();
                },
                DroneCommand::Crash => {
                    command_channel.send(DroneCommand::Crash).unwrap();
                },
            }
        } else {
            error!("");
        }
    }
}

//-----
// GUI
//-----
use eframe::egui;

#[derive(Clone)]
struct DroneInstance {
    id: NodeId,             // Id of the Drone
    x: f32,                 // X-coordinate for display
    y: f32,                 // Y-coordinate for display
    selected: bool,         // Boolean to track if the drone is selected by the user
    color: egui::Color32,   // Color used for visual representation of the drone
}

struct SimulationControllerInstance {
    nodes: Vec<DroneInstance>,
    edges: Vec<(usize, usize)>,
    edge_color: egui::Color32
}

// Implementation for updating the simulation UI in the eframe application (the main loop)
impl eframe::App for SimulationControllerInstance {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Creating the main window for the UI
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Simulation Controller");

            // Allocating space for drawing and preparing the painter for rendering
            let (_response, painter) =
                ui.allocate_painter(egui::Vec2::new(400.0, 400.0), egui::Sense::hover());

            // Drawing edges (connections) between drones
            for &(start_idx, end_idx) in &self.edges {
                let start = self.nodes[start_idx].clone();
                let end = self.nodes[end_idx].clone();
                painter.line_segment(
                    [egui::pos2(start.x, start.y), egui::pos2(end.x, end.y)],
                    egui::Stroke::new(2.0, self.edge_color),
                );
            }

            // Drawing the nodes (drones) and handling user interaction for selection
            for pos in self.nodes.iter_mut() {
                let screen_pos = egui::pos2(pos.x, pos.y);
                let radius = 10.0;

                // Allocating space for each drone's graphical representation
                let response = ui.allocate_rect(
                    egui::Rect::from_center_size(screen_pos, egui::Vec2::splat(radius * 2.0)),
                    egui::Sense::click(),
                );

                // Detecting if the drone is clicked and updating its selected status
                if response.clicked() {
                    pos.selected = true;
                }

                // Drawing the drone as a filled circle
                painter.circle_filled(screen_pos, radius, pos.color);
            }

            // Displaying a pop-up with detailed information when a drone is selected
            for instance in self.nodes.iter_mut() {
                if instance.selected {
                    egui::Window::new(format!("Node {}", instance.id))
                        .fixed_size([100.0, 100.0]) // Window size
                        .resizable(false) // disable resizable
                        .collapsible(true) // activate collapsable
                        .show(ctx, |ui| {
                            // Displaying information about the selected drone.
                            ui.label(format!("Id: {}", instance.id));
                            ui.label(format!(
                                "Neighbors: {:?}",
                                99
                            ));
                            ui.label(format!("PDR: {}", 99));
                            ui.add_space(10.0);

                            // Buttons to change the color of the selected drone
                            ui.horizontal_centered(|ui| {
                                if ui.button("Red").clicked() {
                                    instance.color = egui::Color32::RED;
                                }
                                if ui.button("Green").clicked() {
                                    instance.color = egui::Color32::GREEN;
                                }
                                if ui.button("Blue").clicked() {
                                    instance.color = egui::Color32::BLUE;
                                }
                            });
                            ui.add_space(10.0);

                            // Button to close the window
                            if ui.button("Close").clicked() {
                                instance.selected = false;
                            }
                        });
                }
            }
        });
    }
}