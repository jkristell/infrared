use std::{
    fs::File,
    io::{self, ErrorKind},
    path::Path,
};

use infrared::{
    protocols::NecSamsung,
    recv::EventReceiver,
};

fn main() -> io::Result<()> {
    let (parser, resolution, irdata) = vcd_ir_parser("examples/samsung-tv.vcd", "ir")?;

    println!("samplerate: {:?}", resolution);

    let mut ir_recv: EventReceiver<NecSamsung> = EventReceiver::new(resolution);

    let mut clock = 0;

    for vc in parser {
        let vc = vc?;
        match vc {
            vcd::Command::ChangeScalar(i, v) if i == irdata => {
                let one = v == vcd::Value::V1;
                match ir_recv.update(one, clock) {
                    Ok(Some(cmd)) => {
                        // We have a parsed command
                        println!("Got Command: {:?}", cmd);
                    }
                    Ok(None) => {}
                    Err(err) => {
                        println!("Infrared Receiver Error: {:?}", err);
                    }
                }
            }
            vcd::Command::Timestamp(ts) => {
                clock = ts as u32;
            }
            _ => (),
        }
    }

    Ok(())
}

pub fn vcd_ir_parser<P: AsRef<Path>>(path: P, wire_name: &str) -> io::Result<(vcd::Parser<File>, u32, vcd::IdCode)> {
    let file = File::open(path)?;
    let mut parser = vcd::Parser::new(file);

    // Parse the header and find the wires
    let header = parser.parse_header()?;
    let data = header
        .find_var(&["top", wire_name])
        .ok_or_else(|| io::Error::new(ErrorKind::InvalidInput, format!("no wire top.{}", wire_name)))?
        .code;

    let timescale = header.timescale.unwrap();

    let samplerate = match timescale.1 {
            vcd::TimescaleUnit::MS => 1_000 / timescale.0,
            vcd::TimescaleUnit::US => 1_000_000 / timescale.0,
            _ => panic!("unsupported"),
        };

    Ok((parser, samplerate, data))
}
