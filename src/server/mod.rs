pub mod server;

#[cfg(not(feature = "dedicated-server"))]
mod internal_server;

pub use server::DedicatedServerPlugin;

#[cfg(not(feature = "dedicated-server"))]
pub use internal_server::{InternalServerPlugin, InternalServerState};
