use super::{Error, IspCommand};
use std::io::{Read, Write};

// Device and Memory constants
const PY_CHIP_PID: u16 = 0x440;
const PY_BLOCKSIZE: u8 = 128;
const PY_FLASH_ADDR: u32 = 0x08000000;
const PY_CODE_ADDR: u32 = 0x08000000;
const PY_SRAM_ADDR: u32 = 0x20000000;
const PY_BOOT_ADDR: u32 = 0x1fff0000;
const PY_UID_ADDR: u32 = 0x1fff0e00;
const PY_OPTION_ADDR: u32 = 0x1fff0e80;
const PY_CONFIG_ADDR: u32 = 0x1fff0f00;

// Command codes
const PY_CMD_GET: u8 = 0x00;
const PY_CMD_VER: u8 = 0x01;
const PY_CMD_PID: u8 = 0x02;
const PY_CMD_READ: u8 = 0x11;
const PY_CMD_WRITE: u8 = 0x31;
const PY_CMD_ERASE: u8 = 0x44;
const PY_CMD_GO: u8 = 0x21;
const PY_CMD_W_LOCK: u8 = 0x63;
const PY_CMD_W_UNLOCK: u8 = 0x73;
const PY_CMD_R_LOCK: u8 = 0x82;
const PY_CMD_R_UNLOCK: u8 = 0x92;

// Reply codes
const PY_REPLY_ACK: u8 = 0x79;
const PY_REPLY_NACK: u8 = 0x1f;
const PY_REPLY_BUSY: u8 = 0xaa;

// Other codes
const PY_SYNCH: u8 = 0x7f;

// Default option bytes
const PY_OPTION_DEFAULT: [u8; 16] = [
    0xaa, 0xbe, 0x55, 0x41, 0xff, 0x00, 0x00, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0x00, 0x00,
];

const PY_FRAME_MAX_LEN: usize = 256;

struct Command {
    cmd: IspCommand,
}

impl Command {
    pub fn new(cmd: IspCommand) -> Result<Self, Error> {
        Ok(Self { cmd })
    }
    fn host_command_make(&self) -> [u8; 2] {
        let cmd = match self.cmd {
            IspCommand::Get => PY_CMD_GET,
            IspCommand::Pid => PY_CMD_PID,
            _ => unreachable!(),
        };

        [cmd, cmd ^ 0xff]
    }
}

impl From<u8> for Command {
    fn from(value: u8) -> Self {
        let cmd = match value {
            PY_CMD_GET => IspCommand::Get,
            _ => unreachable!(),
        };
        Self { cmd }
    }
}

struct Py32F0xxIsp<T: Read + Write> {
    serial: T,
}

impl<T: Read + Write> Py32F0xxIsp<T> {
    pub fn new(serial: T) -> Self {
        Self { serial }
    }
}

impl<T: Read + Write> Py32F0xxIsp<T> {
    fn go(addr: u32) -> Result<(), Error> {
        todo!()
    }

    fn get(&mut self) -> Result<Vec<Command>, Error> {
        let _ = self.serial.write(&Get::cmd()).map_err(|_| Error::Serial)?;
        let mut buf: [u8; PY_FRAME_MAX_LEN] = [0; PY_FRAME_MAX_LEN];
        let len = self.serial.read(&mut buf).map_err(|_| Error::Serial)?;
        let mut get = Get::new();
        get.parse(&buf[0..len], 0)?;
    }
}

trait command {
    const CMD: u8;
    fn cmd() -> [u8; 2] {
        [Self::CMD, Self::CMD ^ 0xff]
    }

    fn ack() -> u8 {
        PY_REPLY_ACK
    }

    fn parse(&mut self, reply: &[u8], round: usize) -> Result<Option<Vec<u8>>, Error>;
}

struct Get {
    ver: u8,
    cmd: Vec<u8>,
}

impl Get {
    pub fn new() -> Self {
        Self {
            ver: 0,
            cmd: Vec::new(),
        }
    }
}

impl command for Get {
    const CMD: u8 = 0x00;

    fn parse(&mut self, reply: &[u8], _round: usize) -> Result<Option<Vec<u8>>, Error> {
        if reply.is_empty() {
            return Err(Error::NoReply);
        }

        // 检查 ack
        if reply[0] != Self::ack() {
            return Err(Error::NoAck);
        }

        if reply.len() < 2 {
            return Err(Error::NoComplet);
        }

        let len = reply[1] as usize + 4;

        if len != reply.len() {
            return Err(Error::NoComplet);
        }

        // 获取版本
        self.ver = reply[2];

        // 获取支持的命令
        for i in 3..=len - 2 {
            self.cmd.push(reply[i]);
        }

        if reply[len - 1] != Self::ack() {
            return Err(Error::Parse);
        }

        Ok(None)
    }
}
