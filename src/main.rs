pub mod cli;

use clap::Parser;

use crate::cli::NHParser;

fn main() {
    let args = <NHParser as clap::Parser>::parse();

    // match &args.command {
    //     None => println!("Please provide a command!"),
    //     Some(c) => {}
    // }
    // let command = <NHParser as clap::CommandFactory>::command().get_matches();
}
