use crossbeam_channel::{select, Receiver, Sender};
use log::{error, info};
use std::{collections::HashMap, thread};

use colored::Colorize;

use wg_2024::{
    controller::{DroneCommand, DroneEvent},
    drone::Drone,
    network::NodeId,
    packet::Packet,
};

use messages::{
    client_commands::{ChatClientCommand, ChatClientEvent, MediaClientCommand, MediaClientEvent},
    server_commands::{CommunicationServerCommand, CommunicationServerEvent, ContentServerCommand, ContentServerEvent},
    gui_commands::{GUICommands, GUIEvents},
};

pub struct SimulationController {
    pub drones: HashMap<NodeId, (Sender<DroneCommand>, Sender<Packet>)>,
    drone_recv: Receiver<DroneEvent>,
    pub neighbor: HashMap<NodeId, Vec<NodeId>>,
    pub event_send: Sender<DroneEvent>,
    pub new_drones: Vec<Box<dyn Drone>>,

    pub gui_send: Sender<GUIEvents>,
    gui_recv: Receiver<GUICommands>,

    pub cclients: HashMap<NodeId, (Sender<ChatClientCommand>, Sender<Packet>)>,
    cclient_recv: Receiver<ChatClientEvent>,

    pub mclients: HashMap<NodeId, (Sender<MediaClientCommand>, Sender<Packet>)>,
    mclient_recv: Receiver<MediaClientEvent>,

    pub comm_servers: HashMap<NodeId, (Sender<CommunicationServerCommand>, Sender<Packet>)>,
    comm_server_recv: Receiver<CommunicationServerEvent>,

    pub text_servers: HashMap<NodeId, (Sender<ContentServerCommand>, Sender<Packet>)>,
    text_recv: Receiver<ContentServerEvent>,

    pub media_servers: HashMap<NodeId, (Sender<ContentServerCommand>, Sender<Packet>)>,
    media_recv: Receiver<ContentServerEvent>,
}

impl SimulationController {
    pub fn new(
        drones: HashMap<NodeId, (Sender<DroneCommand>, Sender<Packet>)>,
        drone_recv: Receiver<DroneEvent>,
        neighbor: HashMap<NodeId, Vec<NodeId>>,
        event_send: Sender<DroneEvent>,
        gui_send: Sender<GUIEvents>,
        gui_recv: Receiver<GUICommands>,
        cclients: HashMap<NodeId, (Sender<ChatClientCommand>, Sender<Packet>)>,
        cclient_recv: Receiver<ChatClientEvent>,
        mclients: HashMap<NodeId, (Sender<MediaClientCommand>, Sender<Packet>)>,
        mclient_recv: Receiver<MediaClientEvent>,
        comm_servers: HashMap<NodeId, (Sender<CommunicationServerCommand>, Sender<Packet>)>,
        comm_server_recv: Receiver<CommunicationServerEvent>,
        text_servers: HashMap<NodeId, (Sender<ContentServerCommand>, Sender<Packet>)>,
        text_recv: Receiver<ContentServerEvent>,
        media_servers: HashMap<NodeId, (Sender<ContentServerCommand>, Sender<Packet>)>,
        media_recv: Receiver<ContentServerEvent>
    ) -> Self {
        return Self {
            drones,
            drone_recv,
            neighbor,
            event_send,
            new_drones: Vec::new(),
            gui_send,
            gui_recv,
            cclients,
            cclient_recv,
            mclients,
            mclient_recv,
            comm_servers,
            comm_server_recv,
            text_servers,
            text_recv,
            media_servers,
            media_recv
        };
    }

    pub fn run(&mut self) {
        info!(
            "[ {} ] Starting Simulation Controller",
            "Simulation Controller".green()
        );

        thread::sleep(std::time::Duration::from_secs(2));

        // Init ChatClient
        for (chat_client, _) in self.cclients.clone().iter() {
            self.handle_cclient_command(chat_client, ChatClientCommand::InitFlooding);
            thread::sleep(std::time::Duration::from_secs(5));
            self.handle_cclient_command(chat_client, ChatClientCommand::StartChatClient);
            thread::sleep(std::time::Duration::from_secs(5));
        }

        // Start loop
        loop {
            select! {
                recv(self.drone_recv) -> drone_event => match drone_event {
                    Ok(drone_event) => {
                        info!("[ {} ]: DroneEvent received", "Simulation Controller".green());
                        self.handle_drone_event(drone_event);
                    }
                    Err(e) => {
                        error!("[ {} ]: DroneEvent receiver channel disconnected: {}", "Simulation Controller".red(), e);
                        break;
                    }
                },
                recv(self.cclient_recv) -> cclient_event => match cclient_event {
                    Ok(cclient_event) => {
                        info!("[ {} ]: ChatClientEvent received", "Simulation Controller".green());
                        self.handle_cclient_event(cclient_event);
                    }
                    Err(e) => {
                        error!("[ {} ]: ChatClientEvent receiver channel disconnected: {}", "Simulation Controller".red(), e);
                        break;
                    }
                },
                recv(self.mclient_recv) -> mclient_event => match mclient_event {
                    Ok(mclient_event) => {
                        info!("[ {} ]: MediaClientEvent received", "Simulation Controller".green());
                        self.handle_mclient_event(mclient_event);
                    }
                    Err(e) => {
                        error!("[ {} ]: MediaClientEvent receiver channel disconnected: {}", "Simulation Controller".red(), e);
                        break;
                    }
                },
                recv(self.comm_server_recv) -> comm_event => match comm_event {
                    Ok(comm_event) => {
                        info!("[ {} ]: CommunicationServer received", "Simulation Controller".green());
                        self.handle_commserver_event(comm_event);
                    }
                    Err(e) => {
                        error!("[ {} ]: MediaClientEvent receiver channel disconnected: {}", "Simulation Controller".red(), e);
                        break;
                    }
                },
                recv(self.text_recv) -> text_event => match text_event {
                    Ok(text_event) => {
                        info!("[ {} ]: TextContentServer received", "Simulation Controller".green());
                        self.handle_text_event(text_event);
                    }
                    Err(e) => {
                        error!("[ {} ]: MediaClientEvent receiver channel disconnected: {}", "Simulation Controller".red(), e);
                        break;
                    }
                },
                recv(self.media_recv) -> media_event => match media_event {
                    Ok(media_event) => {
                        info!("[ {} ]: MediaClientEvent received", "Simulation Controller".green());
                        self.handle_media_event(media_event);
                    }
                    Err(e) => {
                        error!("[ {} ]: MediaClientEvent receiver channel disconnected: {}", "Simulation Controller".red(), e);
                        break;
                    }
                },
                recv(self.gui_recv) -> gui_command => match gui_command {
                    Ok(gui_command) => {
                        info!("[ {} ]: GUICommand received", "Simulation Controller".green());
                        self.handle_gui_command(gui_command);
                    }
                    Err(e) => {
                        error!("[ {} ]: GUICommands receiver channel disconnected: {}", "Simulation Controller".red(), e);
                        break;
                    }
                },
            }

            //////////////////////////////////////////////////////////// REMOVE
            thread::sleep(std::time::Duration::from_secs_f32(0.01));
        }
    }

    // Handle GUI Commands
    fn handle_gui_command(&mut self, command: GUICommands) {
        match command {
            GUICommands::Spawn(id, connected_node_ids, pdr) => {
                match self.spawn(id, connected_node_ids, pdr) {
                    Ok(()) => return,
                    Err(e) => {
                        error!("{}", e);
                    }
                }
            }
            GUICommands::Crash(drone) => {
                match self.crash(drone) {
                    Ok(()) => self.handle_drone_command(&drone, DroneCommand::Crash),
                    Err(e) => error!("{}", e),
                }
            },
            GUICommands::RemoveSender(node_id, to_remove) => {
                match self.remove_sender(&node_id, &to_remove) {
                    Ok(()) => {
                        if self.drones.contains_key(&node_id) {
                            self.handle_drone_command(
                                &node_id,
                                DroneCommand::RemoveSender(to_remove),
                            )
                        } else if self.cclients.contains_key(&node_id) {
                            self.handle_cclient_command(
                                &node_id,
                                ChatClientCommand::RemoveSender(to_remove),
                            )
                        } else if self.mclients.contains_key(&node_id) {
                            self.handle_mclient_command(
                                &node_id,
                                MediaClientCommand::RemoveSender(to_remove),
                            )
                        } else if self.comm_servers.contains_key(&node_id) {
                            self.handle_commserver_command(
                                &node_id,
                                CommunicationServerCommand::RemoveSender(to_remove),
                            )
                        }
                    }
                    Err(e) => error!("{}", e),
                }
            }
            GUICommands::AddSender(node_id, to_add) => {
                match self.add_sender(&node_id, &to_add) {
                    Ok(()) => {
                        let sender;
                        if self.drones.contains_key(&to_add) {
                            (_, sender) = self.drones.get(&to_add).unwrap().clone();
                        } else if self.cclients.contains_key(&to_add) {
                            (_, sender) = self.cclients.get(&to_add).unwrap().clone();
                        } else if self.mclients.contains_key(&to_add) {
                            (_, sender) = self.mclients.get(&to_add).unwrap().clone();
                        } else {
                            (_, sender) = self.comm_servers.get(&to_add).unwrap().clone();
                        }

                        if self.drones.contains_key(&node_id) {
                            self.handle_drone_command(
                                &node_id,
                                DroneCommand::AddSender(to_add, sender),
                            );
                        } else if self.cclients.contains_key(&node_id) {
                            self.handle_cclient_command(
                                &node_id,
                                ChatClientCommand::AddSender(to_add, sender),
                            );
                        } else if self.mclients.contains_key(&node_id) {
                            self.handle_mclient_command(
                                &node_id,
                                MediaClientCommand::AddSender(to_add, sender),
                            );
                        } else if self.comm_servers.contains_key(&node_id) {
                            self.handle_commserver_command(
                                &node_id,
                                CommunicationServerCommand::AddSender(to_add, sender),
                            );
                        }
                    }
                    Err(e) => error!("{}", e),
                }
            }
            GUICommands::SetPDR(drone, pdr) => {
                if pdr >= 0.0 && pdr <= 1.0 {
                    self.handle_drone_command(&drone, DroneCommand::SetPacketDropRate(pdr))
                } else {
                    error!("[ ERROR ]: The PDR number is out of range. Please enter a number between 0.00 and 1.00")      
                }
            },

            GUICommands::SendMessageTo(src, dest, msg) => {
                self.handle_cclient_command(&src, ChatClientCommand::SendMessageTo(dest, msg))
            },
            GUICommands::RegisterTo(client, server) => {
                self.handle_cclient_command(&client, ChatClientCommand::RegisterTo(server))
            },
            GUICommands::GetClientList(client) => {
                self.handle_cclient_command(&client, ChatClientCommand::LogNetwork);
                //self.handle_cclient_command(&client, ChatClientCommand::GetClientList)
            }
            GUICommands::LogOut(client, _) => {
                self.handle_cclient_command(&client, ChatClientCommand::LogOut)
            },
            GUICommands::AskForFileList(client, server) => {
                self.handle_mclient_command(&client, MediaClientCommand::AskFilesList(server));
            },
            GUICommands::GetFile(client, server, title) => {
                self.handle_mclient_command(&client, MediaClientCommand::AskForFile(server, title));
            }
        }
    }
}
