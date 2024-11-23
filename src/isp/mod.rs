pub mod py32f0xx_isp;

const COMMAND_CNT: usize = 16;

#[derive(PartialEq)]
pub enum IspCommand {
    /// 获取当前自举程序版本及允许使用的命令
    Get,
    Ver,
    /// 获取芯片 ID
    Pid,
    /// 从应用程序指定的地址开始读取最多 256 个字节的存储器空间
    Read,
    /// 从应用程序指定的地址开始将最多 256 个字节的数据写入 RAM 或 Flash
    Write,
    /// 使用双字节寻址模式擦除一个到全部 Flash 页面
    Erase,
    /// 跳转到内部 Flash 或 SRAM 内的应用程序代码
    Go,
    WriteLock,
    WriteUnlock,
    ReadLock,
    ReadUnlock,

    Other,
}

#[derive(PartialEq, Debug)]
pub enum Error {
    Unsupport,
    Address,
    NoReply,
    Len(usize),

    // 解析错误
    Parse,
    // 串口错误
    Serial,
    // 数据不完整
    NoComplet,

    // 无 ACK
    NoAck,
    Other,
}

pub trait Isp {
    fn get() -> Result<Vec<IspCommand>, Error>;
    fn ability(cmd: IspCommand) -> Result<bool, Error>;
    fn go() -> Result<(), Error>;
    fn read(address: u32, buf: &mut [u32]) -> Result<(), Error>;
    fn write(address: u32, buf: &[u32]) -> Result<(), Error>;
    fn erase(address: u32) -> Result<(), Error>;
    fn lock(lock: bool) -> Result<(), Error>;
}
