use std::io;
use std::net::UdpSocket;
use std::cell::RefCell;

/// Raw parameters for a RGB light command.
/// The first u8 is the light's logical address.
pub struct LightParam(pub u8, pub u8, pub u8, pub u8);

fn clamp(num: f32, min: f32, max: f32) -> f32 {
    if num > max {
        max
    } else if num < min {
        min
    } else {
        num
    }
}

fn clamp_u8(num: f32) -> u8 {
    clamp(num * 255.0, 0.0, 255.0) as u8
}

impl LightParam {
    pub fn new(num: u8, red: u8, green: u8, blue: u8) -> LightParam {
        LightParam(num, red, green, blue)
    }

    /// Helper for initializing a light command from a number and 0..1 RGB values.
    pub fn new_f(num: u8, red: f32, green: f32, blue: f32) -> LightParam {
        LightParam(
            num,
            clamp_u8(red),
            clamp_u8(green),
            clamp_u8(blue),
        )
    }
}

/// Sends commands to the Effect Server.
pub struct UdpClient {
    /// UDP socket reused between calls.
    socket: UdpSocket,
    /// Buffer reused between calls.
    buf: RefCell<Vec<u8>>,
}

impl UdpClient {
    /// Build a new UdpClient instance set to talk to a specific address.
    /// (try "valot.party:9909")
    pub fn new(addr: &str) -> io::Result<UdpClient> {
        let socket = UdpSocket::bind("0.0.0.0:0")?;
        socket.connect(addr)?;
        Ok(UdpClient {
            socket,
            buf: RefCell::new(Vec::with_capacity(256)),
        })
    }

    /// Send a message to the effect server.
    pub fn set(&self, nick: &str, lights: &[LightParam]) -> io::Result<usize> {
        let count = lights.len();
        let nick_length = nick.len();

        // omg-optimized (should probably just reuse a vec)
        let bytes = count * 6 + 1 + 2 + nick_length;
        let mut buf = self.buf.borrow_mut();
        // let mut buf = Vec::<u8>::with_capacity(bytes);
        buf.resize(bytes, 0);

        buf[0] = 1; // API version
        buf[1] = 0; // nick tag

        let nick_bytes = nick.as_bytes();
        for i in 0..nick_length {
            buf[i + 2] = nick_bytes[i];
        }

        // terminate nick tag with a null
        buf[nick_length + 2] = 0;

        let mut ofs = 3 + nick_length;  // api version, nick tag code and terminator, nick bytes

        for i in 0..count {
            let light = &lights[i];
            buf[ofs] = 1;               // light packet
            buf[ofs + 1] = light.0;     // light number
            buf[ofs + 2] = 0;           // set RGB color
            buf[ofs + 3] = light.1;     // red
            buf[ofs + 4] = light.2;     // green
            buf[ofs + 5] = light.3;     // blue
            ofs += 6;
        }
        self.socket.send(&buf)
    }
}
