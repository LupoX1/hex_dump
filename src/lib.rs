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
                let row = data_row(address, slice, cli.columns);
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

    let mut blocks : Vec<String> = Vec::new();

    for block in 0 .. columns / 8 {
        let start = block as u8 * 8;
        let end = start + 8;
        let locations: String = (start..end).map(|n| byte_to_hex(n)).collect::<Vec<String>>().join(" ").to_lowercase();
        blocks.push(locations);
    }

    let blocks = blocks.join("  ");

    format!("{0:10}  {1}  {0:text_size$}\n", " ", blocks, text_size = columns).to_lowercase()
}

fn data_row(address: u32, data: &[u8], columns : usize) -> String {
    let address = address_to_hex(address);

    let mut blocks : Vec<String> = Vec::new();

    for block in 0 .. columns / 8 {
        let start = block * 8;
        let end = start + 8;
        let data = &data[start..end];
        let result = data.into_iter().map(|n| byte_to_hex(*n)).collect::<Vec<_>>().join(" ").to_lowercase();
        blocks.push(result);
    }

    let blocks = blocks.join("  ");

    let mut texts: Vec<String> = Vec::new();
    for block in 0 .. columns / 8 {
        let start = block * 8;
        let end = start + 8;
        let data = &data[start..end];
        let result = data.into_iter().map(|n| byte_to_char(*n)).collect::<Vec<_>>();
        texts.push(String::from_utf8(result).unwrap());
    }

    let texts = texts.join(" ");

    format!("{}  {}  {}\n", address, blocks, texts)
}

fn byte_to_char(byte: u8) -> u8 {
    let c = char::from(byte);
    if c.is_alphanumeric() {
        byte
    }else{
        b'.'
    }
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
    fn test_header_8() {
        assert_eq!("            00 01 02 03 04 05 06 07          \n", locations_header(8));
    }

    #[test]
    fn test_header_16() {
        assert_eq!("            00 01 02 03 04 05 06 07  08 09 0a 0b 0c 0d 0e 0f                  \n", locations_header(16));
    }

    #[test]
    fn test_header_32() {
        assert_eq!("            00 01 02 03 04 05 06 07  08 09 0a 0b 0c 0d 0e 0f  10 11 12 13 14 15 16 17  18 19 1a 1b 1c 1d 1e 1f                                  \n", locations_header(32));
    }

    #[test]
    fn test_row() {
        let data: [u8; 16] = [48+0,48+1,48+2,48+3,48+4,48+5,48+6,48+7,48+8,48+9,55+10,55+11,55+12,55+13,55+14,55+15];
        assert_eq!("0xdeadbeef  30 31 32 33 34 35 36 37  38 39 41 42 43 44 45 46  01234567 89ABCDEF\n", data_row(3735928559, &data, 16));
    }

    #[test]
    fn test_short_row() {
        let data: [u8; 4] = [48+0,48+1,48+2,48+3];
        assert_eq!("0xdeadbeef  30 31 32 33 .. .. .. ..  .. .. .. .. .. .. .. ..  01234.... ........\n", data_row(3735928559, &data, 16));

        let data: [u8; 12] = [48+0,48+1,48+2,48+3,48+4,48+5,48+6,48+7,48+8,48+9,55+10,55+11];
        assert_eq!("0xdeadbeef  30 31 32 33 34 35 36 37  38 39 41 42 .. .. .. ..  01234567 89AB....\n", data_row(3735928559, &data, 16));
    }
}
