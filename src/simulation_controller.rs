use crossbeam_channel::{select, Receiver, Sender};
use log::info;
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
    gui_commands::{GUICommands, GUIEvents},
    server_commands::{
        CommunicationServerCommand, CommunicationServerEvent, ContentServerCommand,
        ContentServerEvent,
    },
};

pub struct SimulationController {
    pub drones: HashMap<NodeId, (Sender<DroneCommand>, Sender<Packet>)>,
    pub drone_recv: Receiver<DroneEvent>,
    pub neighbor: HashMap<NodeId, Vec<NodeId>>,
    pub event_send: Sender<DroneEvent>,
    pub new_drones: Vec<Box<dyn Drone>>,

    pub gui_send: Sender<GUIEvents>,
    pub gui_recv: Receiver<GUICommands>,

    pub cclients: HashMap<NodeId, (Sender<ChatClientCommand>, Sender<Packet>)>,
    pub cclient_recv: Receiver<ChatClientEvent>,

    pub mclients: HashMap<NodeId, (Sender<MediaClientCommand>, Sender<Packet>)>,
    pub mclient_recv: Receiver<MediaClientEvent>,

    pub comm_servers: HashMap<NodeId, (Sender<CommunicationServerCommand>, Sender<Packet>)>,
    pub comm_server_recv: Receiver<CommunicationServerEvent>,

    pub text_servers: HashMap<NodeId, (Sender<ContentServerCommand>, Sender<Packet>)>,
    pub text_recv: Receiver<ContentServerEvent>,

    pub media_servers: HashMap<NodeId, (Sender<ContentServerCommand>, Sender<Packet>)>,
    pub media_recv: Receiver<ContentServerEvent>,
}

impl SimulationController {
    #[allow(clippy::too_many_arguments)]
    #[must_use]
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
        media_recv: Receiver<ContentServerEvent>,
    ) -> Self {
        Self {
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
            media_recv,
        }
    }

    pub fn run(&mut self) {
        info!(
            "[ {} ] Starting Simulation Controller",
            "Simulation Controller".green()
        );

        thread::sleep(std::time::Duration::from_secs_f32(0.5));

        // Init ChatClient
        for chat_client in self.cclients.clone().keys() {
            self.handle_chat_client_command(chat_client, ChatClientCommand::InitFlooding);
            self.handle_chat_client_command(chat_client, ChatClientCommand::StartChatClient);
        }

        // Start loop
        loop {
            /*self.handle_drone_event();
            self.handle_chat_client_event();
            self.handle_media_client_event();
            self.handle_communication_server_event();
            self.handle_media_server_event();
            self.handle_text_server_event();
            self.handle_gui_command();*/

            select! {
                recv(self.drone_recv) -> drone_event => match drone_event {
                    Ok(drone_event) => {
                        self.handle_drone_event(drone_event);
                    }
                    Err(_) => {
                        break;
                    }
                },
                recv(self.cclient_recv) -> cclient_event => match cclient_event {
                    Ok(cclient_event) => {
                        self.handle_chat_client_events(cclient_event);
                    }
                    Err(_) => {
                        break;
                    }
                },
                recv(self.mclient_recv) -> mclient_event => match mclient_event {
                    Ok(mclient_event) => {
                        self.handle_media_client_event(mclient_event);
                    }
                    Err(_) => {
                        break;
                    }
                },
                recv(self.comm_server_recv) -> comm_event => match comm_event {
                    Ok(comm_event) => {
                        self.handle_communication_server_event(comm_event);
                    }
                    Err(_) => {
                        break;
                    }
                },
                recv(self.text_recv) -> text_event => match text_event {
                    Ok(text_event) => {
                        self.handle_text_server_event(text_event);
                    }
                    Err(_) => {
                        break;
                    }
                },
                recv(self.media_recv) -> media_event => match media_event {
                    Ok(media_event) => {
                        self.handle_media_server_event(media_event);
                    }
                    Err(_) => {
                        break;
                    }
                },
                recv(self.gui_recv) -> gui_command => match gui_command {
                    Ok(gui_command) => {
                        self.handle_gui_command(gui_command);
                    }
                    Err(_) => {
                        break;
                    }
                },
            }
        }
    }
}
