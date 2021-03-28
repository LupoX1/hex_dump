use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(name="hex_dump", about="Creates a file dump in hex and ascii format")]
pub struct Cli{
    #[structopt(short="i", long="input")]
    input : std::path::PathBuf,

    #[structopt(short="c", long="columns", default_value="16")]
    columns : usize,
}