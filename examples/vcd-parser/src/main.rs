use std::{
    fs::File,
    io::{self, ErrorKind},
    path::Path,
};

use infrared::Receiver;

fn main() -> io::Result<()> {

    let mut args = std::env::args();
    let _ = args.next();

    // vcd file
    let path = args.next().unwrap_or("samsung-tv.vcd".to_owned());
    // wire name
    let wire_name = args.next().unwrap_or("ir".to_owned());

    let (parser, resolution, irdata) = vcd_ir_parser(&path, &wire_name)?;

    println!("Samples captured at: {:?} Hz", resolution);

    let mut ir_recv = Receiver::builder()
        .resolution(resolution)
        .monotonic::<u64>()
        .nec_samsung()
        .build();

    let mut clock = 0;

    for vc in parser {
        let vc = vc?;
        match vc {
            vcd::Command::ChangeScalar(i, v) if i == irdata => {
                let edge = v == vcd::Value::V1;
                match ir_recv.event_instant(clock, edge) {
                    Ok(Some(cmd)) => {
                        println!("Cmd: {:?}", cmd);
                    }
                    Ok(None) => {}
                    Err(err) => {
                        println!("Infrared Receiver Error: {:?}", err);
                    }
                }
            }
            vcd::Command::Timestamp(ts) => {
                clock = ts;
            }
            _ => (),
        }
    }

    Ok(())
}

pub fn vcd_ir_parser<P: AsRef<Path>>(
    path: P,
    wire_name: &str,
) -> io::Result<(vcd::Parser<File>, u32, vcd::IdCode)> {
    let file = File::open(path)?;
    let mut parser = vcd::Parser::new(file);

    // Parse the header and find the wires
    let header = parser.parse_header()?;
    let data = header
        .find_var(&["top", wire_name])
        .ok_or_else(|| {
            io::Error::new(
                ErrorKind::InvalidInput,
                format!("no wire top.{}", wire_name),
            )
        })?
        .code;

    let timescale = header.timescale.unwrap();

    let samplerate = match timescale.1 {
        vcd::TimescaleUnit::MS => 1_000 / timescale.0,
        vcd::TimescaleUnit::US => 1_000_000 / timescale.0,
        _ => panic!("unsupported"),
    };

    Ok((parser, samplerate, data))
}
