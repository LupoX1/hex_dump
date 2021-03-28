#[macro_use]
extern crate log;

use structopt::StructOpt;
use std::io::{BufReader, Read, BufWriter, Write};
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
    let output = cli.input.join(".dump");

    let file_input = File::open(cli.input)?;
    let file_output = File::create(output)?;
    let mut reader = BufReader::new(file_input);
    let mut writer = BufWriter::new(file_output);

    let mut buffer = [0u8; 32];

    loop{
        match reader.read(&mut buffer[..]) {
            Ok(bytes_read) if bytes_read > 0 => {
                info!("READ: {}", String::from_utf8(Vec::from(&buffer[0..bytes_read])).unwrap());
                writer.write(&buffer[0..bytes_read])?;
            }
            Err(err) => {
                return Err(err);
            }
            _ => {
                info!("EOF");
                return Ok(());
            }
        }
    }
}