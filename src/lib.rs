#[macro_use]
extern crate log;

use structopt::StructOpt;
use std::io::{BufReader, Read, BufWriter, Write};
use std::fs::File;
use std::error::Error;
use std::fmt::{Display, Formatter};

#[derive(Debug, StructOpt)]
#[structopt(name="hex_dump", about="Creates a file dump in hex and ascii format")]
pub struct CommandLine {
    #[structopt(short="i", long="input")]
    input : std::path::PathBuf,

    #[structopt(short="c", long="columns", default_value="16")]
    columns : usize,
}

type Result<T> = std::result::Result<T, Box<dyn Error>>;

#[derive(Debug, Clone)]
struct IoError{
    message : String,
}

impl Display for IoError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "IoError: {}", self.message)
    }
}

impl Error for IoError{}

pub fn dump(cli: CommandLine) -> Result<()> {
    let mut output = cli.input.clone();
    output.set_extension("dump");

    let file_input = File::open(&cli.input).map_err(|e| {
        let message = format!("{} : {:?}", e, &cli.input);
        Box::new(IoError { message })
    })?;

    let reader = BufReader::new(file_input);

    let file_output =  File::create(&output).map_err(|e| {
        let message = format!("{} : {:?}", e, output);
        Box::new(IoError { message })
    })?;

    let mut writer = BufWriter::new(file_output);

    for byte in reader.bytes() {
        match byte {
            Ok(byte) => {
                info!("Read {} = {}", byte, char::from(byte));
                write!(writer, "{}", byte)?;
            }
            Err(e) => {
                let message = format!("{} : {:?}", e, cli.input);
                return Err(Box::new(IoError { message }));
            }
        }
    }

    Ok(())
}
