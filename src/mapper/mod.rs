use std::collections::HashMap;

use crate::config::{self, Root};
use crate::host::{self, LightHost, LightCommand};

mod parser;
use parser::{Command, CommandParser, ParserError};

/// A single light's state in the mapper.
struct Light {
    name: String,
    host_index: usize,
    address: usize,
    red: u8,
    green: u8,
    blue: u8,
}

/// Mappers read requests and issue them to host devices.
pub struct Mapper {
    lights: HashMap<u8, Light>,
    light_hosts: Vec<Box<LightHost>>,

    parser: CommandParser,
}

pub type MapperResult<T> = Result<T, MapperError>;

/// Various runtime errors for the Mapper.
#[derive(Debug)]
pub enum MapperError {
    /// Unknown command tag (is this caught by the parser?)
    UnknownTag(u8),
    /// Unknown logical address.
    UnknownAddr(u8),
    /// The parser couldn't understand the message.
    ParserError(ParserError),
    /// Some sort of I/O error occurred.
    IoError(std::io::Error),
}

impl From<ParserError> for MapperError {
    fn from(err: ParserError) -> MapperError {
        MapperError::ParserError(err)
    }
}

impl Mapper {
    /// Try to set up a mapper and its host devices from a configuration.
    pub fn from_config(config: &Root) -> MapperResult<Mapper> {
        let mut lights: HashMap<u8, Light> = HashMap::new();
        let mut light_hosts: Vec<Box<LightHost>> = vec![];

        // Helper for assigning lights to hosts.
        let mut light_hosts_lookup: HashMap<String, usize> = HashMap::new();

        // Read host device information.
        for (id, host) in &config.hosts {
            light_hosts_lookup.insert(id.clone(), light_hosts.len());
            let host_device: Box<LightHost> = match host {
                config::Host::Enttec { path } => Box::new(
                    host::Enttec::new(path.as_ref()).expect("Unable to initialize Enttec device!"),
                ),
                config::Host::Proxy { addr } => Box::new(
                    host::UdpProxy::new(addr).expect("Unable to initialize Proxy device!")
                ),
            };
            light_hosts.push(host_device);
        }

        // Set up lights and their host device mapping.
        for (id, light) in &config.mapping.lights {
            match light {
                config::Light::Rgb {
                    host,
                    address,
                    name,
                } => {
                    let host_index = 0;

                    lights.insert(
                        *id,
                        Light {
                            name: name.clone().unwrap_or_else(|| format!("{}-{}", host, id)),
                            host_index,
                            address: *address as usize,
                            red: 0,
                            green: 0,
                            blue: 0,
                        },
                    );
                }
            }
        }

        Ok(Mapper {
            lights,
            light_hosts,
            parser: CommandParser::new(),
        })
    }

    /**
     * Read a message from a buffer and issue some commands.
     */
    pub fn take_msg(&mut self, buf: &[u8]) -> MapperResult<()> {
        let mut reader = std::io::BufReader::new(buf);
        self.parser.read_from(&mut reader)?;

        let mut last_nick: Option<String> = None;

        for cmd in &self.parser.cmds {
            match cmd {
                Command::Nick { nick } => {
                    // ewww, clone
                    last_nick = Some(nick.clone())
                }
                Command::RgbLight {
                    id,
                    light_type,
                    red,
                    green,
                    blue,
                } => {
                    // Look for a light with a given id
                    let light = self
                        .lights
                        .get_mut(&id);
                    if light.is_none() {
                        eprintln!("Unknown light id {}", id);
                        continue;
                    }
                    let light = light.unwrap();
                        // .ok_or_else(|| MapperError::UnknownAddr(*id))?;
                    // Check that its type matches the command
                    // TODO: Actually do that.

                    // Issue a command to its host
                    let host = &mut self.light_hosts[light.host_index];
                    host.take_command(&LightCommand {
                        id: *id as usize,
                        address: light.address,
                        red: *red,
                        green: *green,
                        blue: *blue,
                    })
                    // And record that the host needs a flush
                    // TODO: Actually do that.
                }
            }
        }

        for host in &mut self.light_hosts {
            host.flush();
        }

        Ok(())
    }
}
