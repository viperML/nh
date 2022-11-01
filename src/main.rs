pub mod cli;

use crate::cli::NHParser;

fn main() {
    let args = <NHParser as clap::Parser>::parse();
    // let command = <NHParser as clap::CommandFactory>::command().get_matches();
}
