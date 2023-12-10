use bevy::prelude::App;

mod protocol;
mod server;

#[cfg(not(feature = "dedicated-server"))]
mod client;

fn main() {
    let mut app = App::new();

    #[cfg(feature = "dedicated-server")]
    app.add_plugins(server::DedicatedServerPlugin);

    #[cfg(not(feature = "dedicated-server"))]
    app.add_plugins(client::ClientPlugin);

    app.run();
}
