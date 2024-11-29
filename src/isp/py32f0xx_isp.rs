use super::Error;
use byteorder::{BigEndian, ByteOrder};
use std::io::{Read, Write};

// Device and Memory constants
#[allow(unused)]
const PY_CHIP_PID: u16 = 0x440;
const PY_BLOCKSIZE: usize = 128;
#[allow(unused)]
const PY_FLASH_ADDR: u32 = 0x08000000;
pub const PY_CODE_ADDR: u32 = 0x08000000;
#[allow(unused)]
const PY_SRAM_ADDR: u32 = 0x20000000;
#[allow(unused)]
const PY_BOOT_ADDR: u32 = 0x1fff0000;
#[allow(unused)]
const PY_UID_ADDR: u32 = 0x1fff0e00;
const PY_OPTION_ADDR: u32 = 0x1fff0e80;
#[allow(unused)]
const PY_CONFIG_ADDR: u32 = 0x1fff0f00;

const PY_FLASH_SIZE: u32 = 64 * 1024;

// Command codes
const PY_CMD_GET: u8 = 0x00;
const PY_CMD_VER: u8 = 0x01;
const PY_CMD_PID: u8 = 0x02;
const PY_CMD_READ: u8 = 0x11;
const PY_CMD_WRITE: u8 = 0x31;
const PY_CMD_ERASE: u8 = 0x44;
const PY_CMD_GO: u8 = 0x21;
#[allow(unused)]
const PY_CMD_W_LOCK: u8 = 0x63;
#[allow(unused)]
const PY_CMD_W_UNLOCK: u8 = 0x73;
#[allow(unused)]
const PY_CMD_R_LOCK: u8 = 0x82;
const PY_CMD_R_UNLOCK: u8 = 0x92;

// Reply codes
const PY_REPLY_ACK: u8 = 0x79;
const PY_REPLY_NACK: u8 = 0x1f;
#[allow(unused)]
const PY_REPLY_BUSY: u8 = 0xaa;

// Other codes
const PY_SYNCH: u8 = 0x7f;

// Default option bytes
#[allow(unused)]
const PY_OPTION_DEFAULT: [u8; 16] = [
    0xaa, 0xbe, 0x55, 0x41, 0xff, 0x00, 0x00, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0x00, 0x00,
];

#[allow(unused)]
const PY_FRAME_MAX_LEN: usize = 256;

#[derive(PartialEq)]
enum Command {
    Get,
    GetVersion,
    GetId,
    ReadMemory,
    Go,
    WriteMemory,
    EraseMemory,
    ReadUnlock,

    Other(u8),
}

impl From<Command> for u8 {
    fn from(value: Command) -> Self {
        match value {
            Command::EraseMemory => PY_CMD_ERASE,
            Command::Get => PY_CMD_GET,
            Command::GetId => PY_CMD_PID,
            Command::Go => PY_CMD_GO,
            Command::ReadMemory => PY_CMD_READ,
            Command::WriteMemory => PY_CMD_WRITE,
            Command::GetVersion => PY_CMD_VER,
            Command::ReadUnlock => PY_CMD_R_UNLOCK,
            Command::Other(c) => c,
        }
    }
}

impl From<u8> for Command {
    fn from(value: u8) -> Self {
        match value {
            PY_CMD_GET => Self::Get,
            PY_CMD_PID => Self::GetId,
            PY_CMD_READ => Self::ReadMemory,
            PY_CMD_GO => Self::Go,
            PY_CMD_WRITE => Self::WriteMemory,
            PY_CMD_ERASE => Self::EraseMemory,
            PY_CMD_VER => Self::GetVersion,
            PY_CMD_R_LOCK => Self::ReadUnlock,
            c => Self::Other(c),
        }
    }
}

pub struct Py32F0xxIsp<T: Read + Write> {
    serial: T,
}

impl<T: Read + Write> Py32F0xxIsp<T> {
    pub fn new(serial: T) -> Self {
        Self { serial }
    }

    pub fn hand_shake(&mut self) -> Result<(), Error> {
        self.clear_serial();
        self.write_to_serial(&[PY_SYNCH])
    }

    pub fn clear_serial(&mut self) {
        let mut v: [u8; 256] = [0; 256];
        // 消耗掉 buf
        let _ = self.serial.read(&mut v);
    }

    fn write_to_serial(&mut self, buf: &[u8]) -> Result<(), Error> {
        self.serial.write_all(buf).map_err(|_| Error::Serial)?;
        self.check_ack()
    }

    fn check_ack(&mut self) -> Result<(), Error> {
        let mut ack: [u8; 1] = [0; 1];
        self.serial
            .read_exact(&mut ack)
            .map_err(|_| Error::NoReply)?;

        if ack[0] != PY_REPLY_ACK && ack[0] != PY_REPLY_NACK {
            // println!("repy: {:02x}", ack[0]);
            let mut buf: [u8; 10] = [0; 10];
            let _ = self.serial.read(&mut buf);

            // println!("{:02x?}", buf);
            return Err(Error::NoAck);
        }

        Ok(())
    }

    fn read_from_serial(&mut self, buf: &mut [u8]) -> Result<(), Error> {
        self.serial.read_exact(buf).map_err(|e| {
            println!("{}", e);
            Error::Serial
        })
    }

    fn send_command(&mut self, cmd: Command) -> Result<(), Error> {
        let cmd: u8 = cmd.into();
        self.write_to_serial(&[cmd, cmd ^ 0xff])
    }

    pub fn send_address(&mut self, addr: u32) -> Result<(), Error> {
        let mut cmd: [u8; 5] = [0; 5];
        BigEndian::write_u32(&mut cmd, addr);
        let parity = cmd[0..4].iter().fold(0, |xor, x| xor ^ x);
        // self.serial.write_all(&cmd).map_err(|_| Error::Serial)?;
        cmd[4] = parity;
        self.write_to_serial(&cmd)
    }
}

impl<T: Read + Write> Py32F0xxIsp<T> {
    pub fn go(&mut self, addr: u32) -> Result<(), Error> {
        self.send_command(Command::Go)?;
        self.send_address(addr)
    }

    pub fn get(&mut self) -> Result<(u8, Vec<u8>), Error> {
        self.clear_serial();
        self.send_command(Command::Get)?;
        let mut len: [u8; 1] = [0; 1];
        self.read_from_serial(&mut len)?;

        let mut ver: [u8; 1] = [0; 1];
        self.read_from_serial(&mut ver)?;

        let len = len[0] + 1;
        let mut v = Vec::new();
        let mut tmp: [u8; 1] = [0; 1];
        for _ in 0..len {
            self.read_from_serial(&mut tmp)?;
            v.push(tmp[0]);
        }

        Ok((ver[0], v))
    }

    pub fn write_flash(&mut self, addr: u32, data: &[u8]) -> Result<(), Error> {
        self.clear_serial();

        if data.is_empty() {
            return Ok(());
        }

        let item = data.chunks(PY_BLOCKSIZE);
        let size = data.len();

        let mut cnt = 0;
        for data in item {
            let len = data.len() as u8 - 1;
            let parity = data.iter().fold(len, |parity, x| parity ^ *x);

            self.send_command(Command::WriteMemory)?;
            self.send_address(addr + cnt)?;
            self.serial.write_all(&([len])).map_err(|_| Error::Serial)?;
            self.serial.write_all(data).map_err(|_| Error::Serial)?;
            self.write_to_serial(&[parity])?;

            println!(
                "address: {:08x}, chunk: {}, process: {:03}/100",
                addr + cnt,
                data.len(),
                (cnt * 100 / size as u32)
            );

            cnt += data.len() as u32;
        }
        Ok(())
    }

    pub fn read_flash(&mut self, addr: u32, buf: &mut [u8]) -> Result<(), Error> {
        self.clear_serial();
        let items = buf.chunks_mut(PY_BLOCKSIZE);
        let mut cnt = 0;
        for data in items {
            let len = data.len() as u8 - 1;
            self.send_command(Command::ReadMemory)?;
            self.send_address(addr + cnt)?;
            self.send_command(Command::Other(len))?;
            self.read_from_serial(data)?;
            cnt += data.len() as u32;
        }
        Ok(())
    }

    pub fn get_id(&mut self) -> Result<u16, Error> {
        self.clear_serial();

        self.send_command(Command::GetId)?;
        let mut len: [u8; 1] = [0; 1];
        self.read_from_serial(&mut len[..])?;
        let mut pid: [u8; 2] = [0; 2];
        self.read_from_serial(&mut pid)?;
        self.check_ack()?;

        Ok(BigEndian::read_u16(&pid))
    }

    pub fn get_version(&mut self) -> Result<u8, Error> {
        self.clear_serial();
        self.send_command(Command::GetVersion)?;
        let mut ver: [u8; 3] = [0; 3];
        self.read_from_serial(&mut ver[..])?;
        self.check_ack()?;

        Ok(ver[0])
    }

    pub fn read_unlock(&mut self) -> Result<(), Error> {
        self.clear_serial();
        self.send_command(Command::ReadUnlock)
    }

    pub fn erase_chip(&mut self) -> Result<(), Error> {
        self.send_command(Command::EraseMemory)?;
        self.write_to_serial(&[0xff, 0xff, 0x00])
    }

    pub fn read_option(&mut self) -> Result<[u8; 16], Error> {
        let mut buf: [u8; 16] = [0; 16];
        self.read_flash(PY_OPTION_ADDR, &mut buf)?;
        Ok(buf)
    }
}
