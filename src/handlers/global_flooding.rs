use messages::{
    client_commands::{ChatClientCommand, MediaClientCommand},
    server_commands::{CommunicationServerCommand, ContentServerCommand},
};

use crate::SimulationController;

impl SimulationController {
    pub fn global_flooding(&mut self) {
        for (chat_client, (_, _)) in self.cclients.clone() {
            self.handle_chat_client_command(&chat_client, ChatClientCommand::InitFlooding);
        }
        for (media_client, (_, _)) in self.mclients.clone() {
            self.handle_media_client_command(&media_client, MediaClientCommand::InitFlooding);
        }
        for (text_server, (_, _)) in self.text_servers.clone() {
            self.handle_text_server_command(&text_server, ContentServerCommand::InitFlooding);
        }
        for (media_server, (_, _)) in self.media_servers.clone() {
            self.handle_media_server_command(&media_server, ContentServerCommand::InitFlooding);
        }
        for (comm_server, (_, _)) in self.comm_servers.clone() {
            self.handle_communication_server_command(
                &comm_server,
                CommunicationServerCommand::InitFlooding,
            );
        }
    }
}
