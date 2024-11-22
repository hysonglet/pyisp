pub mod py32f0xx_isp;

// Device and Memory constants
const PY_CHIP_PID: u16 = 0x440;
const PY_BLOCKSIZE: u8 = 128;
const PY_FLASH_ADDR: u8 = 0x08000000;
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

enum Command {
    Get = 0x00,
    Ver = 0x01,
    Pid = 0x02,
    Read = 0x11,
    Write = 0x31,
    Erase = 0x44,
    Go = 0x21,
    WriteLock = 0x63,
    WriteUnlock = 0x73,
    ReadLock = 0x82,
    ReadUnlock = 0x92,
}
