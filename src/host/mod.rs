//! Host devices receive commands and produce physical effects.

use std::io;

pub mod proxy;
pub mod enttec;
pub use self::enttec::Enttec;

pub use self::proxy::UdpProxy;

/// Light hosts accept RGB or other commands and pass them to an Enttec-like device.
pub trait LightHost {
    /// Accept a single light command.
    fn take_command(&mut self, cmd: &LightCommand);
    /// Write the current buffer to the device.
    ///
    /// TODO: Should this do double buffering to allow rollback in case of protocol fails?
    fn flush(&mut self) -> io::Result<()>;
}

/// Command to set a single light to a given color.
#[derive(Debug)]
pub struct LightCommand {
    /// Logical light id. Possibly useful for logging or specific implementations.
    pub id: usize,
    /// Address that may mean something to a specific implementation.
    pub address: usize,
    pub red: u8,
    pub green: u8,
    pub blue: u8,
}

