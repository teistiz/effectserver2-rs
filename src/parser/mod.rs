//! Effect server v1 message format parser.

use std::io::{self, Read};
use byteorder::ReadBytesExt;

/// Command buffer and message parser.
pub struct CommandParser {
    /// Commands parsed so far.
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
    // InvalidNick,
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

    pub fn read_from(&mut self, buf: &mut dyn Read) -> ParserResult<()> {
        // Clear temp command buffer.
        self.cmds.clear();
        // Check the header.
        self.read_header(buf)?;
        while self.read_cmd(buf)? { }
        Ok(())
    }

    ///
    fn read_header(&mut self, buf: &mut dyn Read) -> ParserResult<()> {
        match buf.read_u8() {
            Ok(1) => Ok(()),
            Ok(ver) => Err(ParserError::InvalidProtocolVersion(ver)),
            Err(io_error) => Err(ParserError::IoError(io_error)),
        }
    }

    /// Read the next command, if there is any data left. Returns Ok(false) on end-of-data.
    fn read_cmd(&mut self, buf: &mut dyn Read) -> ParserResult<bool> {
        // Read cmd tag
        match buf.read_u8() {
            Ok(0) => self.read_cmd_nick(buf),
            Ok(1) => self.read_cmd_light(buf),
            Ok(cmd) => Err(ParserError::UnknownCommand(cmd)),
            Err(io_error) => match io_error.kind() {
                io::ErrorKind::UnexpectedEof => {
                    // Running out of data right before a command is ok.
                    return Ok(false);
                },
                _ => return Err(ParserError::IoError(io_error)),
            },
        }?;
        return Ok(true);
    }

    /// Read a nickname command.
    fn read_cmd_nick(&mut self, buf: &mut dyn Read) -> ParserResult<()> {
        let mut tmp = vec![];
        // Read bytes until we hit a zero
        loop {
            let byte = buf.read_u8()?;
            if byte == 0 {
                break;
            }
            tmp.push(byte);
        }

        // Let's just tolerate bad UTF-8 for now.
        let nick = String::from_utf8_lossy(&tmp).to_string();
        self.cmds.push(Command::Nick { nick });
        Ok(())
    }

    /// Read a basic light command from the buffer.
    fn read_cmd_light(&mut self, buf: &mut dyn Read) -> ParserResult<()> {
        let id = buf.read_u8()?;

        let light_type = buf.read_u8()?;
        if light_type != 0 {
            return Err(ParserError::UnsupportedLightType(light_type));
        }

        self.cmds.push(Command::RgbLight {
            id,
            light_type,
            red: buf.read_u8()?,
            green: buf.read_u8()?,
            blue: buf.read_u8()?,
        });
        Ok(())
    }
}