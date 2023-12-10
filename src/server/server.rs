use std::collections::HashMap;

use bevy::{app::AppExit, prelude::*};
use bevy_quinnet::{
    server::{
        certificate::CertificateRetrievalMode, ConnectionLostEvent, Endpoint, QuinnetServerPlugin,
        Server, ServerConfiguration,
    },
    shared::{channel::ChannelId, ClientId},
};

use crate::protocol::{ClientMessage, ServerMessage};

#[derive(Resource, Debug, Clone, Default)]
pub(in crate::server) struct Users {
    pub(crate) names: HashMap<ClientId, String>,
}

/// Receives and handles network messages from clients
pub(in crate::server) fn handle_client_messages(mut server: ResMut<Server>, mut users: ResMut<Users>) {
    let endpoint = server.endpoint_mut();
    for client_id in endpoint.clients() {
        while let Some(message) = endpoint.try_receive_message_from::<ClientMessage>(client_id) {
            match message {
                ClientMessage::Join { name } => {
                    if users.names.contains_key(&client_id) {
                        warn!(
                            "Received a Join from an already connected client: {}",
                            client_id
                        );
                    } else {
                        info!("{} connected", name);
                        users.names.insert(client_id, name.clone());
                        endpoint
                            .send_message(
                                client_id,
                                ServerMessage::InitClient {
                                    client_id,
                                    usernames: users.names.clone(),
                                },
                            )
                            .expect("Failed to send init client message to new client");
                        endpoint
                            .send_group_message(
                                users.names.keys().into_iter(),
                                ServerMessage::ClientConnected {
                                    client_id,
                                    username: name,
                                },
                            )
                            .expect("Failed to send client connected message to clients");
                    }
                }
                ClientMessage::Disconnect {} => {
                    // add something to disconnect clients if host quits
                    endpoint.disconnect_client(client_id).unwrap();
                    handle_disconnect(endpoint, &mut users, client_id);
                }
                ClientMessage::ChatMessage { message } => {
                    info!(
                        "Chat message | {:?}: {}",
                        users.names.get(&client_id),
                        message
                    );
                    endpoint
                        .send_group_message_on(
                            users.names.keys().into_iter(),
                            ChannelId::UnorderedReliable,
                            ServerMessage::ChatMessage { client_id, message },
                        )
                        .expect("Failed to send group message with chat");
                }
            }
        }
    }
}

/// Handles quinnet server events, right now just connection lost events
pub(in crate::server) fn handle_server_events(
    mut connection_lost_events: EventReader<ConnectionLostEvent>,
    mut server: ResMut<Server>,
    mut users: ResMut<Users>,
) {
    for client in connection_lost_events.read() {
        handle_disconnect(server.endpoint_mut(), &mut users, client.id);
    }
}

/// Removes a user from the users list and sends a message to everyone that they left.
/// DOES NOT DISCONNECT THE USER. That is assumed to have already happened.
pub(in crate::server) fn handle_disconnect(endpoint: &mut Endpoint, users: &mut ResMut<Users>, client_id: ClientId) {
    if let Some(username) = users.names.remove(&client_id) {
        endpoint
            .send_group_message(
                users.names.keys().into_iter(),
                ServerMessage::ClientDisconnected { client_id },
            )
            .expect("Failed to send user disconnected group message");
        info!("{} disconnected", username);
    } else {
        warn!(
            "Received a Disconnect from an unknown or disconnected client: {}",
            client_id
        );
    }
}

/// Starts server endpoint
pub(in crate::server) fn start_listening(mut server: ResMut<Server>) {
    server
        .start_endpoint(
            ServerConfiguration::from_string("0.0.0.0:6006").unwrap(),
            CertificateRetrievalMode::GenerateSelfSigned {
                server_hostname: "127.0.0.1".to_string(),
            },
        )
        .expect("Server failed to start endpoint");
}

/// Catches app exit events and calls `on_server_exit` which stops the endpoint etc.
pub(in crate::server) fn handle_app_exit(
    app_exit_events: EventReader<AppExit>,
    server: ResMut<Server>,
    users: Res<Users>,
) {
    if !app_exit_events.is_empty() {
        on_server_exit(server, users);
    }
}

/// Sends a group message of ServerStopping and stops endpoint.
pub(in crate::server) fn on_server_exit(mut server: ResMut<Server>, users: Res<Users>) {
    info!("Server exiting!");

    let endpoint = server.endpoint();
    endpoint
        .send_group_message(
            users.names.keys().into_iter(),
            ServerMessage::ServerStopping {},
        )
        .expect("Server failed to send group message that it is stopping");
    server
        .stop_endpoint()
        .expect("Server failed to stop its endpoint");
}

pub struct DedicatedServerPlugin;
impl Plugin for DedicatedServerPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((MinimalPlugins, QuinnetServerPlugin::default()))
            .init_resource::<Users>()
            .add_systems(Startup, (start_listening,))
            .add_systems(FixedUpdate, (handle_client_messages, handle_server_events))
            .add_systems(PostUpdate, (handle_app_exit,));
    }
}
