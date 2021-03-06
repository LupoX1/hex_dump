#[macro_use]
extern crate log;
use structopt::StructOpt;
use hex_dump::{CommandLine, dump};

fn main() {
    env_logger::init();

    info!("Starting Dump");

    let opt = CommandLine::from_args();

    if !opt.valid() {
        error!("Accepted values for columns are 8,16,32,64");
        std::process::exit(1);
    }

    let code = match dump(opt){
        Ok(_) => {0}
        Err(error) => {
            error!("{}", error);
            1
        }
    };

    info!("End Dump");

    std::process::exit(code);
}
