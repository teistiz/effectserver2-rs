//! The Mapper maps logical addresses to host device commands.

use std::collections::HashMap;
use std::net::IpAddr;
use std::sync::Arc;
use tokio::sync::watch;

use crate::config::{self, Root};
use crate::host::{self, EffectHost, LightCommand};
use crate::parser::{Command, CommandParser, ParserError};

mod monitor;
use monitor::StatusMessage;

#[derive(Debug)]
struct Effect {
    /// Name to use for this effect.
    name: Arc<String>,
    /// Index of the host this effect is connected to.
    host_index: usize,
    /// Number that may mean something to the host.
    /// TODO would be nice if this didn't have to know about DMX addresses or whatever.
    address: usize,
    /// Last IP address that set this.
    last_ip: Option<IpAddr>,
    /// Last tag that set this.
    last_tag: Option<Arc<String>>,
    /// Detailed description of the effect.
    kind: EffectKind,
}
#[derive(Debug)]
enum EffectKind {
    Rgb(LightRgb),
    Uv(LightUv),
    // Smoke(),
}

/// A single RGB light's state in the mapper.
#[allow(dead_code)]
#[derive(Debug)]
struct LightRgb {
    /// Last known red intensity.
    red: u8,
    /// Last known green intensity.
    green: u8,
    /// Last known blue intensity.
    blue: u8,
}

/// A single RGB light's state in the mapper.
#[allow(dead_code)]
#[derive(Debug)]
struct LightUv {
    /// Last known intensity.
    intensity: u8,
}

/// Mappers read commands and issue them to host devices.
pub struct Mapper {
    /// Configured lights.
    effects: HashMap<u8, Effect>,
    /// Configured light effect hosts.
    effect_hosts: Vec<Box<dyn EffectHost>>,
    /// Command parser/buffer.
    parser: CommandParser,
    /// Message bus for watching teh status.
    sender: watch::Sender<monitor::StatusMessage>,
}

/// Result type for various Mapper actions.
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

/// Parser errors can become Mapper errors.
impl From<ParserError> for MapperError {
    fn from(err: ParserError) -> MapperError {
        MapperError::ParserError(err)
    }
}

impl Mapper {
    /// Try to set up a mapper and its host devices from a configuration.
    pub fn from_config(config: &Root) -> MapperResult<Mapper> {
        let mut effects: HashMap<u8, Effect> = HashMap::new();
        let mut effect_hosts: Vec<Box<dyn EffectHost>> = vec![];

        // Helper for assigning lights to hosts.
        let mut hosts_lookup: HashMap<String, usize> = HashMap::new();

        // Read host device information.
        for (id, host) in &config.hosts {
            hosts_lookup.insert(id.clone(), effect_hosts.len());
            let host_device: Box<dyn EffectHost> = match host {
                config::Host::Enttec { path } => Box::new(
                    host::Enttec::new(path.as_ref()).expect("Unable to initialize Enttec device!"),
                ),
                config::Host::Proxy { addr } => {
                    Box::new(host::UdpProxy::new(addr).expect("Unable to initialize Proxy device!"))
                }
            };
            effect_hosts.push(host_device);
        }

        let get_host = |host: &String| -> usize {
            match hosts_lookup.get(host) {
                Some(host_index) => *host_index,
                None => {
                    panic!("Unknown host: {}", host);
                }
            }
        };

        // Set up lights and their host device mapping.
        for (id, light) in &config.mapping.lights {
            match light {
                config::Light::Rgb {
                    host,
                    address,
                    name,
                } => {
                    let host_index = get_host(host);

                    effects.insert(
                        *id,
                        Effect {
                            name: Arc::new(
                                name.clone().unwrap_or_else(|| format!("{}-{}", host, id)),
                            ),
                            host_index,
                            address: *address as usize,
                            last_ip: None,
                            last_tag: None,
                            kind: EffectKind::Rgb(LightRgb {
                                red: 0,
                                green: 0,
                                blue: 0,
                            }),
                        },
                    );
                }
                config::Light::Uv {
                    host,
                    address,
                    name,
                } => {
                    let host_index = get_host(host);

                    effects.insert(
                        *id,
                        Effect {
                            name: Arc::new(
                                name.clone().unwrap_or_else(|| format!("{}-{}", host, id)),
                            ),
                            host_index,
                            address: *address as usize,
                            last_ip: None,
                            last_tag: None,
                            kind: EffectKind::Uv(LightUv { intensity: 0 }),
                        },
                    );
                }
            }
        }

        let (sender, receiver) = watch::channel(Self::get_status_message(&effects));

        monitor::start_monitor_thread(&config.server.web_addr, receiver);

        Ok(Mapper {
            effects,
            effect_hosts,
            parser: CommandParser::new(),
            sender,
        })
    }

    /// Turn the current effect status into a StatusMessage.
    fn get_status_message(effects: &HashMap<u8, Effect>) -> StatusMessage {
        use monitor::LightStatus;
        let mut lights = Vec::with_capacity(effects.len());
        for (id, effect) in effects {
            let (r, g, b) = match effect.kind {
                EffectKind::Rgb(LightRgb { red, green, blue }) => (red, green, blue),
                EffectKind::Uv(LightUv { intensity }) => (intensity, intensity, intensity),
            };
            lights.push(LightStatus {
                id: *id,
                r,
                g,
                b,
                tag: effect.last_tag.clone(),
                ip: effect.last_ip.clone(),
            });
        }

        lights.sort_unstable_by_key(|light| light.id);

        StatusMessage { lights }
    }

    /// Read a message from a buffer and issue some commands.
    ///
    /// TODO: Should the messages be parsed by the servers themselves?
    /// Or would that move too much "business logic" into them?
    pub fn take_msg(&mut self, buf: &[u8], ip: Option<IpAddr>) -> MapperResult<()> {
        let mut reader = std::io::BufReader::new(buf);
        self.parser.read_from(&mut reader)?;

        let mut last_nick: Option<Arc<String>> = None;

        for cmd in &self.parser.cmds {
            match cmd {
                Command::Nick { nick } => {
                    last_nick = Some(Arc::new(nick.clone()));
                }
                Command::RgbLight {
                    id,
                    light_type,
                    red,
                    green,
                    blue,
                } => {
                    let mut effect = if let Some(effect) = self.effects.get_mut(&id) {
                        effect
                    } else {
                        eprintln!("Unknown light id {}", id);
                        continue;
                    };

                    if *light_type != 0 {
                        eprintln!("Unknown light type {}", light_type);
                    }

                    effect.last_ip = ip;
                    effect.last_tag = last_nick.clone();

                    let host = &mut self.effect_hosts[effect.host_index];

                    match &mut effect.kind {
                        EffectKind::Rgb(light) => {
                            light.red = *red;
                            light.green = *green;
                            light.blue = *blue;
                            host.take_command(&LightCommand::Rgb {
                                id: *id as usize,
                                address: effect.address,
                                red: light.red,
                                green: light.green,
                                blue: light.blue,
                            });
                        }
                        EffectKind::Uv(light) => {
                            let luma = (*red as u16) * 2 + (*green as u16) * 7 + (*blue as u16);
                            light.intensity = (luma / 10) as u8;
                            host.take_command(&LightCommand::Uv {
                                id: *id as usize,
                                address: effect.address,
                                intensity: light.intensity,
                            });
                        }
                    }

                    // TODO Keep track of which hosts have received commands
                }
            }
        }

        // TODO: Only flush the hosts that were used.
        for host in &mut self.effect_hosts {
            host.flush().ok();
        }

        // Update the status bus.
        self.sender
            .send(Self::get_status_message(&self.effects))
            .ok();

        Ok(())
    }
}
