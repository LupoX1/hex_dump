#[macro_use]
extern crate log;

use structopt::StructOpt;
use std::io::{BufReader, Read};
use std::fs::File;

#[derive(Debug, StructOpt)]
#[structopt(name="hex_dump", about="Creates a file dump in hex and ascii format")]
pub struct CommandLine {
    #[structopt(short="i", long="input")]
    input : std::path::PathBuf,

    #[structopt(short="c", long="columns", default_value="16")]
    columns : usize,
}

pub fn dump(cli: CommandLine) -> std::io::Result<()>{
    let _output = cli.input.join(".dump");

    let mut file = File::open(cli.input)?;
    let mut reader = BufReader::new(file);
    let mut buffer = [0u8; 32];

    loop{
        match reader.read(&mut buffer[..]) {
            Ok(bytes_read) if bytes_read > 0 => {
                info!("READ: {}", String::from_utf8(Vec::from(&buffer[0..bytes_read])).unwrap());
            }
            Err(err) => {
                error!("{}", err);
                break;
            }
            _ => {
                info!("EOF");
                break;
            }
        }
    }

    Ok(())
}