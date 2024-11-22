use clap::Parser;

mod isp;

/// Simple program to greet a person
#[derive(Parser, Debug)]
#[command(version, about,
    long_about = Some("Programming Tool for PUYA PY32F0xx Microcontrollers"), 
    author = "Alingsos",
    version = "0.1.0")]
struct Args {
    /// Name of the person to greet
    #[arg(short, long)]
    com: String,

    /// lock chip (set read protection)
    #[arg(short, long, default_value_t = false)]
    lock: bool,

    /// perform chip erase (implied with -f)
    #[arg(short, long, default_value_t = false)]
    erase: bool,

    /// reset option bytes
    #[arg(short, long, default_value_t = false)]
    rstoption: bool,

    /// make nRST pin a RESET pin(false: nRST pin as GPIO pin)
    #[arg(short, long, default_value_t = false)]
    nrst_as_reset: bool,

    /// write BIN file to flash and verify
    #[arg(short, long, default_value_t = 0x8000000)]
    addr: u32,

    /// flash BIN file name
    #[arg(short, long)]
    file: String,
}

fn main() {
    let args = Args::parse();
}
