//! Host devices receive commands and produce physical effects.

use std::io;

pub mod proxy;
pub mod enttec;
pub use self::enttec::Enttec;

pub use self::proxy::UdpProxy;

/// Light hosts accept RGB or other commands and pass them to an Enttec-like device.
pub trait EffectHost {
    /// Accept a single light command.
    fn take_command(&mut self, cmd: &LightCommand);
    /// Write the current buffer to the device.
    ///
    /// TODO: Should this do double buffering to allow rollback in case of protocol fails?
    fn flush(&mut self) -> io::Result<()>;
}

pub enum LightCommand {
    Rgb { id: usize, address: usize, red: u8, green: u8, blue: u8 },
    Uv { id: usize, address: usize, intensity: u8 },
}

