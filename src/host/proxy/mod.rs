//! Proxy for effectserver science.

use std::io;
use super::{LightCommand, LightHost};

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

impl LightHost for UdpProxy {
    fn take_command(&mut self, cmd: &LightCommand) {
        self.cmds.push(LightParam::new(
            cmd.id as u8,
            cmd.red,
            cmd.green,
            cmd.blue,
        ))
    }
    fn flush(&mut self) -> io::Result<()> {
        self.client.set("esrs proxy", self.cmds.as_slice())?;
        self.cmds.clear();

        Ok(())
    }
}
