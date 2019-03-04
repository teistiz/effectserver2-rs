use std::io::{self, Read};
use byteorder::ReadBytesExt;

/// Command buffer and message parser.
pub struct CommandParser {
    pub cmds: Vec<Command>,
}

/// Abstract commands recognized by the command mapper.
pub enum Command {
    /// Set nick for the next commands.
    Nick { nick: String },
    /// Set an RGB light's state.
    RgbLight { id: u8, light_type: u8, red: u8, green: u8, blue: u8 },
}

pub type ParserResult<T> = Result<T, ParserError>;

#[derive(Debug)]
pub enum ParserError {
    InvalidProtocolVersion(u8),
    UnsupportedLightType(u8),
    UnknownCommand(u8),
    InvalidNick,
    IoError(io::Error),
}

impl From<io::Error> for ParserError {
    fn from(err: io::Error) -> ParserError {
        ParserError::IoError(err)
    }
}

impl CommandParser {
    pub fn new() -> CommandParser {
        CommandParser {
            cmds: vec![],
        }
    }

    pub fn read_from(&mut self, buf: &mut Read) -> ParserResult<()> {
        self.cmds.clear();
        // This mostly just verifies the protocol version, for now.
        self.read_header(buf)?;

        while self.read_cmd(buf)? { }

        Ok(())
    }

    fn read_header(&mut self, buf: &mut Read) -> ParserResult<()> {
        match buf.read_u8() {
            Ok(1) => Ok(()),
            Ok(ver) => Err(ParserError::InvalidProtocolVersion(ver)),
            Err(io_error) => Err(ParserError::IoError(io_error)),
        }
    }

    fn read_cmd(&mut self, buf: &mut Read) -> ParserResult<bool> {
        // Read cmd tag
        match buf.read_u8() {
            Ok(0) => self.read_cmd_nick(buf),
            Ok(1) => self.read_cmd_light(buf),
            Ok(cmd) => Err(ParserError::UnknownCommand(cmd)),
            Err(io_error) => match io_error.kind() {
                io::ErrorKind::UnexpectedEof => {
                    return Ok(false);
                },
                _ => return Err(ParserError::IoError(io_error)),
            },
        };
        return Ok(true);
    }

    fn read_cmd_nick(&mut self, buf: &mut Read) -> ParserResult<()> {
        let mut tmp = vec![];
        // Read bytes until we hit a zero
        loop {
            let byte = buf.read_u8()?;
            if byte == 0 {
                break;
            }
            tmp.push(byte);
        }

        let nick = String::from_utf8_lossy(&tmp).to_string();
        self.cmds.push(Command::Nick { nick });
        Ok(())
    }

    fn read_cmd_light(&mut self, buf: &mut Read) -> ParserResult<()> {
        self.cmds.push(Command::RgbLight {
            id: buf.read_u8()?,
            light_type: buf.read_u8()?,
            red: buf.read_u8()?,
            green: buf.read_u8()?,
            blue: buf.read_u8()?,
        });
        Ok(())
    }
}