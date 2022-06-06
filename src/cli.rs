use clap::Parser;

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
pub struct Args {
    #[clap(short, long, default_value = "/")]
    pub dir: String,

    #[clap(short, long, default_value = "")]
    pub ignore: String,

    #[clap(long)]
    pub home: bool,

    #[clap(long)]
    pub hidden: bool
}

pub fn get_args() -> Args {
    Args::parse()
}