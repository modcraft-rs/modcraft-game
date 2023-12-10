use bevy::prelude::*;
use bevy_quinnet::server::QuinnetServerPlugin;

use crate::server::server::*;

/// Used so that the client can know when to
/// attempt to connect.
#[derive(Debug, Clone, Eq, PartialEq, Hash, Default, States)]
pub enum InternalServerState {
    #[default]
    Off,
    Launching,
    Running,
}

/// These are just schedule labels the mimic the built in
/// schedule labels. These will do more in the future.
#[derive(Debug, Hash, PartialEq, Eq, Clone, SystemSet)]
enum ServerSystems {
    Startup,
    FixedUpdate,
    OnExit,
}

/// A system that is run when during the `OnEnter(InternalServerState::Launching)` schedule, after
/// the ServerSystems::Startup set.
fn set_internal_server_ready(
    mut next_internal_server_state: ResMut<NextState<InternalServerState>>,
) {
    next_internal_server_state.set(InternalServerState::Running);
}

/// Clears the users resource when finished
fn clear_users(mut users: ResMut<Users>) {
    (*users).names.clear();
}

pub struct InternalServerPlugin;
impl Plugin for InternalServerPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(QuinnetServerPlugin::default())
            .add_state::<InternalServerState>()
            .init_resource::<Users>()
            .add_systems(
                OnEnter(InternalServerState::Launching),
                (start_listening,).in_set(ServerSystems::Startup),
            )
            .add_systems(
                OnEnter(InternalServerState::Launching),
                set_internal_server_ready.after(ServerSystems::Startup),
            )
            .add_systems(
                FixedUpdate,
                (handle_client_messages, handle_server_events)
                    .run_if(in_state(InternalServerState::Running))
                    .in_set(ServerSystems::FixedUpdate),
            )
            .add_systems(
                OnExit(InternalServerState::Running),
                (on_server_exit.before(clear_users), clear_users).in_set(ServerSystems::OnExit),
            );
    }
}
