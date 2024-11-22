pub mod py32f0xx_isp;

pub enum Command {
    Get,
    Ver,
    Pid,
    Read,
    Write,
    Erase,
    Go,
    WriteLock,
    WriteUnlock,
    ReadLock,
    ReadUnlock,
}

pub enum Error {
    Unsupport,
    Address,
    NoReply,
    Len(usize),

    Other,
}

pub trait Isp {
    fn ability(cmd: Command) -> Result<bool, Error>;
    fn go() -> Result<(), Error>;
    fn read(address: u32, buf: &mut [u32]) -> Result<(), Error>;
    fn write(address: u32, buf: &[u32]) -> Result<(), Error>;
    fn erase(address: u32) -> Result<(), Error>;
    fn lock(lock: bool) -> Result<(), Error>;
}
