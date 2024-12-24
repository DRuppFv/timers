use clap::Parser;
#[derive(Parser, Debug)]
#[command(about)]
pub struct Args {
    ///Time in seconds
    pub time: Option<String>,
}
