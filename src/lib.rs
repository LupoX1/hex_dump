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

impl CommandLine{
    pub fn valid(&self) -> bool{
        [8usize,16usize,32usize,64usize].contains(&self.columns)
    }
}

type Result<T> = std::result::Result<T, Box<dyn Error>>;

#[derive(Debug, Clone)]
struct IoError{
    message : String,
}

impl Display for IoError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
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

    let mut reader = BufReader::new(file_input);

    let file_output =  File::create(&output).map_err(|e| {
        let message = format!("{} : {:?}", e, output);
        Box::new(IoError { message })
    })?;

    let mut writer = BufWriter::new(file_output);

    let mut buffer: Vec<u8> = Vec::new();
    buffer.resize(cli.columns, 0);
    //let mut buffer:[u8;16] = [0; 16];
    let mut address = 0;
    loop {
        match reader.read(&mut buffer){
            Ok(bytes_read) if bytes_read == 0 => {
                info!("EOF");
                return Ok(());
            }
            Ok(bytes_read) => {
                if address % (16 * cli.columns as u32) == 0 {
                    write!(writer, "{}", locations_header(cli.columns))?;
                }
                let slice = &buffer[0..bytes_read];
                let row = data_row(address, slice);
                address += bytes_read as u32;
                info!("Read {} Row: {}", bytes_read, row);
                write!(writer, "{}", row)?;
            }
            Err(e) => {
                let message = format!("{} : {:?}", e, cli.input);
                return Err(Box::new(IoError { message }));
            }
        }
    }
}

fn locations_header(columns: usize) -> String {
    let sep = columns as u8 / 2;

    let locations_low = (0..sep as u8).map(|i| byte_to_hex(i)).collect::<Vec<_>>().join(" ");
    let locations_high = (sep..16 as u8).map(|i| byte_to_hex(i)).collect::<Vec<_>>().join(" ");

    format!("            {}  {}                   \n", locations_low, locations_high).to_lowercase()
}

fn data_row(address: u32, data: &[u8]) -> String {
    let address = address_to_hex(address);

    let sep = data.len()  / 2;
    let low = &data[0..sep];
    let high = &data[sep..data.len()];

    let locations_low = low.into_iter().map(|b|byte_to_hex(*b)).collect::<Vec<_>>().join(" ");
    let locations_high = high.into_iter().map(|b|byte_to_hex(*b)).collect::<Vec<_>>().join(" ");

    let text_low = low.into_iter().map(|i|{
        let c = char::from(*i);
        if c.is_alphanumeric() {
            *i
        }else{
            b'.'
        }
    }).collect::<Vec<_>>();

    let text_high = high.into_iter().map(|i|{
        let c = char::from(*i);
        if c.is_alphanumeric() {
            *i
        }else{
            b'.'
        }
    }).collect::<Vec<_>>();

    let text_low = String::from_utf8(text_low).unwrap();
    let text_high = String::from_utf8(text_high).unwrap();

    format!("{}  {}  {}  {} {}\n", address, locations_low, locations_high, text_low, text_high)
}

fn byte_to_hex(byte: u8) -> String {
    format!("{:02X}", byte)
}

fn address_to_hex(address: u32) -> String {
    format!("{:#010x}", address)
}

#[cfg(test)]
mod tests{
    use super::*;

    #[test]
    fn test_byte_to_hex() {
        assert_eq!("00", byte_to_hex(0u8));
        assert_eq!("0F", byte_to_hex(15u8));
        assert_eq!("10", byte_to_hex(16u8));
    }

    #[test]
    fn test_address_to_hex() {
        assert_eq!("0x00000000", address_to_hex(0));
        assert_eq!("0x00000010", address_to_hex( 16));
        assert_eq!("0x000000ff", address_to_hex(255));
        assert_eq!("0xdeadbeef", address_to_hex(3735928559));
    }

    #[test]
    fn test_header() {
        assert_eq!("            00 01 02 03 04 05 06 07  08 09 0a 0b 0c 0d 0e 0f                   \n", locations_header(16));
    }

    #[test]
    fn test_row() {
        let data: [u8; 16] = [48+0,48+1,48+2,48+3,48+4,48+5,48+6,48+7,48+8,48+9,55+10,55+11,55+12,55+13,55+14,55+15];
        assert_eq!("0xdeadbeef  30 31 32 33 34 35 36 37  38 39 41 42 43 44 45 46  01234567 89ABCDEF\n", data_row(3735928559, &data));
    }
}
