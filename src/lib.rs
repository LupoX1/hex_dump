#[macro_use]
extern crate log;

use structopt::StructOpt;
use std::io::{BufReader, Read, BufWriter, Write};
use std::fs::File;
use std::error::Error;
use std::fmt::{Display, Formatter};
use ascii_utils::Check;
use indicatif::{ProgressBar, ProgressStyle};

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

    let bar = ProgressBar::new(file_input.metadata().unwrap().len() as u64);
    bar.set_style(ProgressStyle::default_bar()
        .template("[{eta_precise}] {bar:40.cyan/blue} {bytes:>7}/{total_bytes:7} {msg}")
        .progress_chars("##-"));

    let mut reader = BufReader::new(file_input);

    let file_output =  File::create(&output).map_err(|e| {
        let message = format!("{} : {:?}", e, output);
        Box::new(IoError { message })
    })?;

    let mut writer = BufWriter::new(file_output);

    let mut buffer: Vec<u8> = Vec::new();
    buffer.resize(cli.columns, 0);

    write!(writer, "{}", locations_header(cli.columns))?;

    let mut address = 0;
    loop {
        match reader.read(&mut buffer){
            Ok(bytes_read) if bytes_read == 0 => {
                info!("EOF");
                bar.finish_and_clear();
                return Ok(());
            }
            Ok(bytes_read) => {
                if address % (16 * cli.columns as u32) == 0 {
                    write!(writer, "\n")?;
                }
                let slice = &buffer[0..bytes_read];
                let row = data_row(address, slice, cli.columns);
                address += bytes_read as u32;
                write!(writer, "{}", row)?;
                bar.inc(bytes_read as u64);
            }
            Err(e) => {
                bar.abandon();
                let message = format!("{} : {:?}", e, cli.input);
                return Err(Box::new(IoError { message }));
            }
        }
    }
}

fn gen_block(data: &[u8], fun : fn(&u8) -> String, columns: usize, sep: &str, filler: &str) -> Vec<String> {
    let mut blocks : Vec<String> = Vec::new();

    for block in 0 .. columns / 8 {
        let start = usize::min(data.len(), block * 8);
        let end = usize::min(data.len(), start + 8);

        let data: &[u8] = &data[start..end];

        let mut result = data.into_iter().map(fun).collect::<Vec<_>>();
        while result.len() < 8 {
            result.push(String::from(filler));
        }

        let result = result.join(sep);

        blocks.push(result);
    }

    blocks
}

fn locations_header(columns: usize) -> String {

    let data: Vec<u8> = (0..columns as u8).collect();
    let blocks = gen_block(&data, byte_to_hex, columns, " ", "..");
    let blocks = blocks.join("  ");

    format!("{0:10}  {1}  {0:text_size$}\n", " ", blocks, text_size = columns).to_lowercase();

    let address = format!("{:10}", " ");
    let text = format!("{:size$}", " ", size=columns);

    create_row(&address, &blocks, &text).to_lowercase()
}

fn data_row(address: u32, data: &[u8], columns : usize) -> String {
    let address = address_to_hex(address);
    let blocks = gen_block(data, byte_to_hex, columns, " ", "..");
    let blocks = blocks.join("  ");
    let texts = gen_block(data, byte_to_string, columns, "", ".");
    let texts = texts.join("");

    create_row(&address, &blocks, &texts)
}

fn create_row(address: &str, data: &str, text: &str) -> String {
    format!("{}  {}  {}\n", address, data, text)
}

fn byte_to_string(byte: &u8) -> String {
    let c = char::from(*byte);
    if c.is_printable() {
        String::from(c)
    }else{
        String::from('.')
    }
}

fn byte_to_hex(byte: &u8) -> String {
    format!("{:02X}", byte)
}

fn address_to_hex(address: u32) -> String {
    format!("{:#010x}", address)
}

#[cfg(test)]
mod tests{
    use super::*;

    fn init() {
        let _ = env_logger::builder().is_test(true).try_init();
    }

    #[test]
    fn test_byte_to_char() {
        init();

        assert_eq!(".", byte_to_string(&0u8));
        assert_eq!("A", byte_to_string(&65u8));
    }

    #[test]
    fn test_byte_to_hex() {
        init();

        assert_eq!("00", byte_to_hex(&0u8));
        assert_eq!("0F", byte_to_hex(&15u8));
        assert_eq!("10", byte_to_hex(&16u8));
    }

    #[test]
    fn test_address_to_hex() {
        init();

        assert_eq!("0x00000000", address_to_hex(0));
        assert_eq!("0x00000010", address_to_hex( 16));
        assert_eq!("0x000000ff", address_to_hex(255));
        assert_eq!("0xdeadbeef", address_to_hex(3735928559));
    }

    #[test]
    fn test_header_8() {
        init();

        assert_eq!("            00 01 02 03 04 05 06 07          \n", locations_header(8));
    }

    #[test]
    fn test_header_16() {
        init();

        assert_eq!("            00 01 02 03 04 05 06 07  08 09 0a 0b 0c 0d 0e 0f                  \n", locations_header(16));
    }

    #[test]
    fn test_header_32() {
        init();

        assert_eq!("            00 01 02 03 04 05 06 07  08 09 0a 0b 0c 0d 0e 0f  10 11 12 13 14 15 16 17  18 19 1a 1b 1c 1d 1e 1f                                  \n", locations_header(32));
    }

    #[test]
    fn test_row() {
        init();

        let data: [u8; 16] = [48+0,48+1,48+2,48+3,48+4,48+5,48+6,48+7,48+8,48+9,55+10,55+11,55+12,55+13,55+14,55+15];
        assert_eq!("0xdeadbeef  30 31 32 33 34 35 36 37  38 39 41 42 43 44 45 46  0123456789ABCDEF\n", data_row(3735928559, &data, 16));
    }

    #[test]
    fn test_short_row() {
        init();

        let data: [u8; 4] = [48+0,48+1,48+2,48+3];
        assert_eq!("0xdeadbeef  30 31 32 33 .. .. .. ..  .. .. .. .. .. .. .. ..  0123............\n", data_row(3735928559, &data, 16));

        let data: [u8; 12] = [48+0,48+1,48+2,48+3,48+4,48+5,48+6,48+7,48+8,48+9,55+10,55+11];
        assert_eq!("0xdeadbeef  30 31 32 33 34 35 36 37  38 39 41 42 .. .. .. ..  0123456789AB....\n", data_row(3735928559, &data, 16));
    }

    #[test]
    fn test_create_row() {
        init();

        assert_eq!("address  data  text\n", create_row("address", "data", "text"));
    }
}
