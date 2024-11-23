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
            PY_CMD_PID => IspCommand::Pid,
            PY_CMD_READ => IspCommand::Read,
            PY_CMD_GO => IspCommand::Go,
            PY_CMD_WRITE => IspCommand::Write,
            PY_CMD_ERASE => IspCommand::Erase,
            _ => unreachable!(),
        };
        Self { cmd }
    }
}

pub struct Py32F0xxIsp<T: Read + Write> {
    serial: T,
}

impl<T: Read + Write> Py32F0xxIsp<T> {
    pub fn new(serial: T) -> Self {
        Self { serial }
    }
}

impl<T: Read + Write> Py32F0xxIsp<T> {
    pub fn go(addr: u32) -> Result<(), Error> {
        todo!()
    }

    pub fn get(&mut self) -> Result<(u8, Vec<Command>), Error> {
        let _ = self.serial.write(&Get::cmd()).map_err(|_| Error::Serial)?;
        let mut buf: [u8; PY_FRAME_MAX_LEN] = [0; PY_FRAME_MAX_LEN];
        let len = self.serial.read(&mut buf).map_err(|_| Error::Serial)?;
        let mut get = Get::new();
        let _ = get.parse(&buf[0..len], 0)?;

        Ok((
            get.ver,
            get.cmd.iter().map(|cmd| Command::from(*cmd)).collect(),
        ))
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
    pub ver: u8,
    pub cmd: Vec<u8>,
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

struct GetId {
    id: u16,
}

impl GetId {
    pub fn new() -> Self {
        Self { id: 0xff }
    }
}

impl command for GetId {
    const CMD: u8 = PY_CMD_PID;

    fn parse(&mut self, reply: &[u8], _round: usize) -> Result<Option<Vec<u8>>, Error> {
        if reply.is_empty() {
            return Err(Error::NoReply);
        }

        if reply.len() != 5 {
            return Err(Error::NoComplet);
        }

        if !reply.ends_with(&[Self::ack()]) || reply.starts_with(&[Self::ack()]) {
            return Err(Error::NoAck);
        }

        let msb = reply[2] as u16;
        let lsb = reply[3] as u16;

        self.id = (msb << 8) | lsb;

        Ok(None)
    }
}

struct ReadMemory {
    pub memory: Vec<u8>,
    pub address: u32,
    pub count: u8,
}

impl ReadMemory {
    pub fn new(addr: u32, cnt: u8) -> Self {
        Self {
            address: addr,
            memory: Vec::new(),
            count: cnt,
        }
    }

    pub fn cmd_address(&self) -> [u8; 5] {
        let mut cmd: [u8; 5];
        todo!()
    }

    pub fn cmd_count(&self) -> [u8; 2] {
        todo!()
    }
}

impl command for ReadMemory {
    const CMD: u8 = PY_CMD_READ;

    fn parse(&mut self, reply: &[u8], round: usize) -> Result<Option<Vec<u8>>, Error> {
        todo!()
    }
}

struct Go {
    address: u32,
}

impl Go {
    pub fn new(addr: u32) -> Self {
        Self { address: addr }
    }

    pub fn cmd_address(&self) -> [u8; 5] {
        todo!()
    }
}

impl command for Go {
    const CMD: u8 = PY_CMD_GO;
    fn parse(&mut self, reply: &[u8], round: usize) -> Result<Option<Vec<u8>>, Error> {
        todo!()
    }
}

struct WriteMemory {
    address: u32,
    memory: Vec<u8>,

    round: usize,
}

impl WriteMemory {
    pub fn new(addr: u32, data: Vec<u8>) -> Self {
        Self {
            address: addr,
            memory: data,
            round: 0,
        }
    }

    pub fn cmd_address(&self) -> [u8; 5] {
        todo!()
    }
}

impl command for WriteMemory {
    const CMD: u8 = PY_CMD_WRITE;

    fn parse(&mut self, reply: &[u8], round: usize) -> Result<Option<Vec<u8>>, Error> {
        todo!()
    }
}

struct EraseMemory {
    address: u32,
    erase_type: u8,
}

impl EraseMemory {
    fn new(address: u32, erase_type: u8) -> Self {
        Self {
            address,
            erase_type,
        }
    }
}

impl command for EraseMemory {
    const CMD: u8 = PY_CMD_ERASE;

    fn parse(&mut self, reply: &[u8], round: usize) -> Result<Option<Vec<u8>>, Error> {
        todo!()
    }
}
