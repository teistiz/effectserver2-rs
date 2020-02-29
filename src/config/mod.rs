//! Configuration file format and loader.

use std::collections::HashMap;
use std::path::Path;
use std::{fs, io};

use serde::{Deserialize, Serialize};

/// Configuration root level.
#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Root {
    /// Server configuration.
    pub server: Server,
    /// Effect devices.
    pub hosts: HashMap<String, Host>,
    /// Logical device mapping.
    pub mapping: Mapping,
}

/// API server configuration.
#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Server {
    /// UDP host address to accept packets on.
    pub udp_addr: String,
    /// Host address to serve the Web page and API on.
    pub web_addr: String,
    /// Host address to accept WebSocket connections on.
    pub websocket_addr: String,
}

/// Maps logical addresses to physical devices.
#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Mapping {
    /// Map of logical address -> Light info
    pub lights: HashMap<u8, Light>,
}

/// Individual light source that can be controlled over DMX or similar bus.
#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase", tag = "type")]
pub enum Light {
    Rgb {
        /// Host device to use
        host: String,
        /// DMX address to use.
        address: u16,
        /// Human-readable name.
        name: Option<String>,
    },
    Uv {
        /// Host device to use
        host: String,
        /// DMX address to use.
        address: u16,
        /// Human-readable name.
        name: Option<String>,
    }
}

/// Host device configuration.
#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase", tag = "type")]
pub enum Host {
    Enttec {
        /// Path to a serial device.
        path: Option<String>,
    },
    Proxy {
        // Target UDP address.
        addr: String,
    }
}

/// Read configuration from a JSON file.
///
/// May perform some validation before returning the config root.
pub fn read_config_json<T: AsRef<Path>>(path: T) -> io::Result<Root> {
    let file = fs::File::open(path)?;
    let reader = io::BufReader::new(file);
    let root: Root = serde_json::from_reader(reader).map_err(|err| {
        eprintln!("Error reading config file: {:?}", err);
        io::Error::from(io::ErrorKind::InvalidData)
    })?;

    check_config(root)
}

pub fn read_config_yaml<T: AsRef<Path>>(path: T) -> io::Result<Root> {
    let file = fs::File::open(path)?;
    let reader = io::BufReader::new(file);
    let root: Root = serde_yaml::from_reader(reader).map_err(|err| {
        eprintln!("Error reading config file: {:?}", err);
        io::Error::from(io::ErrorKind::InvalidData)
    })?;

    check_config(root)
}

/// Do a quick sanity check for the config.
fn check_config(root: Root) -> io::Result<Root> {
    let hosts = &root.hosts;
    let lights = &root.mapping.lights;

    for (id, light) in lights {
        match light {
            Light::Rgb { host, .. } => {
                // RGB lights should refer to a valid host.
                if !hosts.contains_key(host) {
                    eprintln!("RGB light {} refers to invalid host: {}", id, host);
                    return Err(io::Error::from(io::ErrorKind::InvalidData))
                }
            },
            Light::Uv { host, .. } => {
                // UV lights should refer to a valid host.
                if !hosts.contains_key(host) {
                    eprintln!("UV light {} refers to invalid host: {}", id, host);
                    return Err(io::Error::from(io::ErrorKind::InvalidData))
                }
            },
        }
    }
    Ok(root)
}
