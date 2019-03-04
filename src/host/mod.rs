use serialport;
use std::io;

mod udpclient;
use udpclient::{LightParam, UdpClient};

type DMXPayload = [u8; 512];

pub trait LightHost {
    fn take_command(&mut self, cmd: &LightCommand);
    fn flush(&mut self) -> io::Result<()>;
}

pub struct Enttec {
    /// Output port.
    port: Option<Box<dyn serialport::SerialPort>>,
    /// Buffer for raw DMX message data..
    payload: DMXPayload,
}

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

impl Enttec {
    /// Construct a new Enttec-type lighting host.
    ///
    /// TODO: Use something smarter than &String?
    pub fn new(path: Option<&String>) -> io::Result<Enttec> {
        println!("Enttec @ {:?}", path);
        let port = match path {
            Some(path) => {
                let mut port = serialport::open(path)?;
                port.set_baud_rate(57600)?;
                Some(port)
            }
            None => None,
        };

        Ok(Enttec {
            payload: [0; 512],
            port,
        })
    }
}

impl LightHost for Enttec {
    /// Write a single light's control data into the buffer.
    ///
    /// TODO: Is this the right API for this? Should this just take raw buffers?
    /// The buffers could then be mixed somewhere else.
    fn take_command(&mut self, cmd: &LightCommand) {
        // println!("take command: {:?}", cmd);
        let offset = cmd.address;
        // let offset = index * 5 + 1;
        if offset > 507 {
            panic!("Invalid DMX bus offset: {}", offset);
        }
        self.payload[offset] = cmd.red;
        self.payload[offset + 1] = cmd.green;
        self.payload[offset + 2] = cmd.blue;
        self.payload[offset + 3] = 255;
        self.payload[offset + 4] = 0;
    }

    /// Flush current buffer into the bus.
    ///
    /// Call this after issuing all commands.
    fn flush(&mut self) -> io::Result<()> {
        // println!("Flushing?");
        if let Some(port) = self.port.as_mut() {
            // Send DMX payload with Enttec header
            let mut writer = io::BufWriter::with_capacity(517, port);
            use std::io::Write;
            writer.write(&[0x7e, 6, 0, 2])?;
            writer.write(&self.payload)?;
            writer.write(&[0xe7])?;
            writer.flush()?;
            println!("Wrote DMX controller payload");
        }
        Ok(())
    }
}

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
        let res = self.client.set("esrs proxy", self.cmds.as_slice());
        self.cmds.clear();

        Ok(())
    }
}
