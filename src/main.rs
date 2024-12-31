use clap::Parser;
use isp::py32f0xx_isp::PY_CODE_ADDR;
use serial::SerialPort;
use serialport::available_ports;

use std::fs;
use std::io::Read;
use std::{thread::sleep, time::Duration};

mod isp;

/// Simple program to greet a person
#[derive(Parser, Debug)]
#[command(version, about,
    long_about = Some("Programming Tool for PUYA PY32F0xx Microcontrollers"),
    author = "Alingsos",
    version = "0.1.0")]
struct Args {
    /// Name of seral port
    #[arg(short, long)]
    serial: Option<String>,

    /// Cycle mode for flash many times
    #[arg(short, long, default_value_t = false)]
    cycle: bool,

    /// Product mode for flash many times
    #[arg(long, default_value_t = false)]
    product: bool,

    /// flash BIN file name
    #[arg(short, long)]
    file: Option<String>,

    /// print serial
    #[arg(short, long, default_value_t = false)]
    probe: bool,

    /// Run to...
    #[arg(short, long, default_value_t = false)]
    go: bool,
    // #[arg(short, long, default_value_t = false)]
    // console: bool,
}

fn main() {
    let args = Args::parse();

    let mut binary = Vec::<u8>::new();
    if let Some(file) = args.file {
        if fs::metadata(&file).is_err() {
            println!("Not such file: {}", file);
            return;
        }
        if !file.ends_with(".bin") {
            println!("Only support flash binary file");
            return;
        }
        let mut file = fs::File::open(file).expect("Can't Open the file");
        let size = file.read_to_end(&mut binary).expect("Cant't read the file");
        if size == 0 {
            println!("Empty file");
            return;
        }
    }

    let serial_list = available_ports().unwrap();

    // 打印当前电脑所有的串口
    if args.probe {
        for p in serial_list {
            println!("\t port: {}", p.port_name);
        }

        return;
    }

    let mut serial_name = String::new();

    if let Some(com) = args.serial {
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

    loop {
        // 输入了串口
        if !serial_name.is_empty() {
            let mut serial = match serial::open(&serial_name) {
                Ok(serial) => serial,
                Err(e) => {
                    println!("{}", e.to_string());
                    sleep(Duration::from_millis(1 * 1000));
                    if !args.product {
                        return;
                    }
                    continue;
                }
            };

            let _ = serial.reconfigure(&|s| {
                let _ = s.set_baud_rate(serial::BaudRate::Baud115200);
                s.set_char_size(serial::CharSize::Bits8);
                s.set_parity(serial::Parity::ParityEven);
                s.set_stop_bits(serial::StopBits::Stop1);
                s.set_flow_control(serial::FlowControl::FlowNone);
                Ok(())
            });

            let _ = serial.set_timeout(Duration::from_millis(100));

            let mut isp = isp::py32f0xx_isp::Py32F0xxIsp::new(serial);

            let mut try_handshake = 0;
            let hand_shake = loop {
                try_handshake += 1;
                isp.boot_into();
                match isp.hand_shake() {
                    Ok(()) => {
                        println!("Connected");
                        break true;
                    }
                    Err(isp::Error::Serial) => {
                        break false;
                    }
                    Err(e) => {
                        println!("{:?}", e);
                    }
                };
                sleep(Duration::from_millis(1000));

                if try_handshake == 10 {
                    println!("Faild to handshake...");
                    break false;
                }
                println!("try to connect: {try_handshake}");
            };
            if !hand_shake {
                continue;
            }

            println!("get: {:02x?}", isp.get());
            println!("id:  {:04x?}", isp.get_id());
            println!("ver: {:04x?}", isp.get_version());
            // println!("unlock: {:?}", isp.read_unlock());
            println!("read option: {:x?}", isp.read_option());

            // 当存在文件才烧录
            if !binary.is_empty() {
                println!("erase: {:?}", isp.erase_chip());
                println!("flash: {:?}", isp.write_flash(PY_CODE_ADDR, &binary));

                if args.go {
                    println!("go:  {:?}", isp.go(PY_CODE_ADDR));
                }
            }
        }

        // 生产模式
        if !args.product {
            if args.cycle {
                println!("Wait for push key boot {}...", serial_name);

                sleep(Duration::from_millis(1 * 1000));
            } else {
                break;
            }
        } else {
            while serial::open(&serial_name).is_ok() {
                println!("Wait for disconnect {}...", serial_name);

                sleep(Duration::from_millis(1 * 1000));
            }
        }
    }
}
