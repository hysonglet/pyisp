pub mod py32f0xx_isp;

#[derive(PartialEq, Debug)]
pub enum Error {
    // Unsupport,
    Address,
    NoReply,
    // Len(usize),

    // 解析错误
    // Parse,
    // 串口错误
    Serial,
    // 数据不完整
    // NoComplet,

    // 无 ACK
    NoAck,
    // Other,
}

// pub trait Isp {
//     fn get() -> Result<Vec<IspCommand>, Error>;
//     fn ability(cmd: IspCommand) -> Result<bool, Error>;
//     fn go() -> Result<(), Error>;
//     fn read(address: u32, buf: &mut [u32]) -> Result<(), Error>;
//     fn write(address: u32, buf: &[u32]) -> Result<(), Error>;
//     fn erase(address: u32) -> Result<(), Error>;
//     fn lock(lock: bool) -> Result<(), Error>;
// }
