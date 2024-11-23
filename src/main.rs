use clap::Parser;
use serialport::{available_ports, DataBits, FlowControl, Parity, SerialPortSettings, StopBits};
use std::{thread::sleep, time::Duration};

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
    com: Option<String>,

    /// lock chip (set read protection)
    #[arg(short, long)]
    lock: Option<bool>,

    /// perform chip erase (implied with -f)
    #[arg(short, long)]
    erase: Option<bool>,

    /// reset option bytes
    #[arg(short, long)]
    rstoption: Option<bool>,

    /// make nRST pin a RESET pin(false: nRST pin as GPIO pin)
    #[arg(short, long)]
    nrst_as_reset: Option<bool>,

    /// write BIN file to flash and verify
    #[arg(short, long)]
    addr: Option<u32>,

    /// flash BIN file name
    #[arg(short, long)]
    file: Option<String>,

    /// print serial
    #[arg(short, long, default_value_t = false)]
    probe: bool,
}

fn main() {
    let args = Args::parse();

    let serial_list = available_ports().unwrap();

    if args.probe {
        for p in serial_list {
            println!("\t info: {:?}", p);
        }

        return;
    }

    let mut serial_name = String::new();

    if let Some(com) = args.com {
        for s in &serial_list {
            if s.port_name.ends_with(com.as_str()) {
                serial_name = s.port_name.clone();
                break;
            }
        }

        if serial_name.is_empty() {
            println!("Not found the serial: {}", com);
            println!("Available serial: ");
            for s in &serial_list {
                println!("\t{}", s.port_name);
            }
            return;
        }
    }

    //
    if !serial_name.is_empty() {
        let serial = serialport::open_with_settings(
            serial_name.as_str(),
            &SerialPortSettings {
                baud_rate: 115200,
                data_bits: DataBits::Eight,
                parity: Parity::None,
                flow_control: FlowControl::None,
                stop_bits: StopBits::One,
                timeout: Duration::from_secs(1),
            },
        );

        if let Err(e) = &serial {
            println!("faild to open {}: {}", serial_name, e.description);
            return;
        };

        let mut isp = isp::py32f0xx_isp::Py32F0xxIsp::new(serial.unwrap());

        for i in 1..=10 {
            match isp.hand_shake() {
                Ok(()) => {
                    println!("ok");
                    break;
                }
                Err(isp::Error::Serial) => {
                    return;
                }
                Err(e) => {
                    println!("{:?}", e);
                }
            }
            sleep(Duration::from_millis(1000));
            println!("try to connect: {i}");
        }

        sleep(Duration::from_secs(1));
        println!("get: {:02x?}", isp.get());
        println!("id:  {:04x?}", isp.get_id());
        println!("ver: {:04x?}", isp.get_version());
        println!("go:  {:?}", isp.go(0x8000000));
    } else {
        //自由扫描
    }
}
