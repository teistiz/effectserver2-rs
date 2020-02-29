//! Proxy for effectserver science.

use std::io;
use super::{LightCommand, EffectHost};

use crate::client::{LightParam, UdpClient};

/// The UDP proxy host passes commands to another effect server.
pub struct UdpProxy {
    client: UdpClient,
    cmds: Vec<LightParam>,
}

impl UdpProxy {
    pub fn new(addr: &str) -> io::Result<UdpProxy> {
        Ok(UdpProxy {
            client: UdpClient::new(addr)?,
            cmds: vec![],
        })
    }
}

impl EffectHost for UdpProxy {
    fn take_command(&mut self, cmd: &LightCommand) {
        match cmd {
            LightCommand::Rgb { id, red, green, blue, ..} => {
                self.cmds.push(LightParam::new(
                    *id as u8,
                    *red,
                    *green,
                    *blue,
                ))
            },
            LightCommand::Uv { id, intensity, .. } => {
                self.cmds.push(LightParam::new(
                    *id as u8,
                    // FIXME calculate UV rgb properly
                    *intensity,
                    *intensity,
                    *intensity,
                ))
            }
        }
    }
    fn flush(&mut self) -> io::Result<()> {
        self.client.set("esrs proxy", self.cmds.as_slice())?;
        self.cmds.clear();

        Ok(())
    }
}
