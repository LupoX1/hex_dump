use log::info;
use structopt::StructOpt;
use hex_dump::Cli;

fn main() {
    env_logger::init();

    let opt = Cli::from_args();

    info!("{:?}", opt);
}
