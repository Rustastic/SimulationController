use crossbeam_channel::{Receiver, Sender};
use std::collections::HashMap;

use colored::Colorize;
use log::{error, info, warn};

use wg_2024::{
    controller::{DroneCommand, DroneEvent},
    network::NodeId,
    packet::{Packet, PacketType},
};

#[derive(Clone)]
pub struct SimulationController {
    drones: HashMap<NodeId, (Sender<DroneCommand>, Sender<Packet>)>,
    receiver: Receiver<DroneEvent>,
    neighbor: HashMap<NodeId, Vec<NodeId>>,
}

impl SimulationController {
    pub fn new(
        drones: HashMap<NodeId, (Sender<DroneCommand>, Sender<Packet>)>,
        receiver: Receiver<DroneEvent>,
        neighbor: HashMap<NodeId, Vec<NodeId>>,
    ) -> Self {
        return Self {
            drones,
            receiver,
            neighbor,
        };
    }

    pub fn run(&mut self) {
        println!("\tStarting Simulation Controller...");
        match self.receiver.recv() {
            Ok(drone_event) => self.handle_event(drone_event),
            Err(_) => error!("{} Channel is closed", "✗".red()),
        }

        println!("\tStarting Simulation Controller GUI...");
        let options = eframe::NativeOptions::default();
        let _ = eframe::run_native(
            "Simulation Controller",
            options,
            Box::new(|_cc| {
                Ok(Box::<SimulationControllerInstance>::new(
                    SimulationControllerInstance::new(self.clone()),
                ))
            }),
        );
        println!("\tShutting Doewn Simulation Controller...");
    }

    fn spawn() {}

    fn handle_event(&self, drone_event: DroneEvent) {
        match drone_event {
            DroneEvent::PacketSent(packet) => {
                let src = packet
                    .routing_header
                    .hops
                    .get(packet.routing_header.hop_index)
                    .unwrap();

                let dest = packet
                    .routing_header
                    .hops
                    .get(packet.routing_header.hop_index + 1)
                    .unwrap();

                let pakcet_type = packet.pack_type;

                // GUI
            }
            DroneEvent::PacketDropped(packet) => {
                let drone = packet
                    .routing_header
                    .hops
                    .get(packet.routing_header.hop_index)
                    .unwrap();

                // GUI
            }
            DroneEvent::ControllerShortcut(packet) => {
                if let Some(dest) = packet.routing_header.hops.get(packet.routing_header.len()) {
                    if let Some((_, packet_channel)) = self.drones.get(dest) {
                        match packet.pack_type {
                            PacketType::MsgFragment(_) => error!(""),
                            _ => {
                                packet_channel.send(packet.clone()).unwrap();
                            }
                        }
                    }
                } else {
                    error!("");
                }
            }
        }
    }

    fn handle_command(&self, drone: &NodeId, drone_command: DroneCommand) {
        if let Some((command_channel, _)) = self.drones.get(drone) {
            match drone_command {
                DroneCommand::RemoveSender(node_id) => {
                    command_channel
                        .send(DroneCommand::RemoveSender(node_id))
                        .unwrap();
                }
                DroneCommand::AddSender(node_id, sender) => {
                    command_channel
                        .send(DroneCommand::AddSender(node_id, sender))
                        .unwrap();
                }
                DroneCommand::SetPacketDropRate(pdr) => {
                    command_channel
                        .send(DroneCommand::SetPacketDropRate(pdr))
                        .unwrap();
                }
                DroneCommand::Crash => {
                    let neighbors = self.neighbor.get(drone).unwrap();

                    for neighbor in neighbors {
                        let (neighbor_channel, _) = self.drones.get(neighbor).unwrap();

                        neighbor_channel
                            .send(DroneCommand::RemoveSender(*drone))
                            .unwrap();
                    }

                    command_channel.send(DroneCommand::Crash).unwrap();
                }
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

#[derive(Eq, Hash, PartialEq, Clone)]
struct DroneInstance {
    id: NodeId,           // Id of the Drone
    x: i32,               // X-coordinate for display
    y: i32,               // Y-coordinate for display
    selected: bool,       // Boolean to track if the drone is selected by the user
    color: egui::Color32, // Color used for visual representation of the drone
}

impl DroneInstance {
    fn new(id: NodeId) -> Self {
        Self {
            id,
            x: 0,
            y: 0,
            selected: false,
            color: egui::Color32::BLUE,
        }
    }
}

struct SimulationControllerInstance {
    nodes: Vec<DroneInstance>,
    edges: HashMap<DroneInstance, Vec<DroneInstance>>,
    edge_color: egui::Color32,
}

impl SimulationControllerInstance {
    fn new(simulation_controller: SimulationController) -> Self {
        let nodes: Vec<DroneInstance> = simulation_controller
            .drones
            .keys()
            .map(|id| DroneInstance::new(id.clone()))
            .collect();

        let mut edges = HashMap::<DroneInstance, Vec<DroneInstance>>::new();

        for (drone_id, neighbor) in simulation_controller.neighbor {
            let start = nodes.iter().find(|drone| drone.id == drone_id).unwrap();
            for dest in neighbor {
                let end = nodes.iter().find(|drone| drone.id == dest).unwrap();
                if edges.contains_key(start) {
                    let vec = edges.get_mut(start).unwrap();
                    vec.push(end.clone());
                } else {
                    let mut vec: Vec<DroneInstance> = Vec::new();
                    vec.push(end.clone());
                    edges.insert(start.clone(), vec);
                }
            }
        }

        Self {
            nodes,
            edges,
            edge_color: egui::Color32::GRAY,
        }
    }
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

            // Drawing the nodes (drones) and handling user interaction for selection
            for pos in self.nodes.iter_mut() {
                let screen_pos = egui::pos2(pos.x as f32, pos.y as f32);
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

                // Drawing edges (connections) between drones
                let start = pos;
                let vec = self.edges.get(&start.clone()).unwrap();
                for end in vec {
                    if end.id > start.id {
                        painter.line_segment(
                            [
                                egui::pos2(start.x as f32, start.y as f32),
                                egui::pos2(end.x as f32, end.y as f32),
                            ],
                            egui::Stroke::new(2.0, self.edge_color),
                        );
                    }
                }
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
                            ui.label(format!("Neighbors: {:?}", 99));
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
