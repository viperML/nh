

use clap_complete::{generate_to, shells};
use std::env;
use std::io::Error;

include!("src/interface.rs");

fn main() -> Result<(), Error> {
    let outdir = env::var_os("OUT_DIR").expect("OUT_DIR not set by the build system!");
    let mut cmd = <NHParser as clap::CommandFactory>::command();

    let bash_output = generate_to(
        shells::Bash,
        &mut cmd, // We need to specify what generator to use
        "nh",  // We need to specify the bin name manually
        &outdir,   // We need to specify where to write to
    )?;
    println!("cargo:warning=Built Bash completions to {bash_output:?}");


    let zsh_output = generate_to(
        shells::Zsh,
        &mut cmd, // We need to specify what generator to use
        "nh",  // We need to specify the bin name manually
        &outdir,   // We need to specify where to write to
    )?;
    println!("cargo:warning=Built Bash completions to {zsh_output:?}");

    let fish_output = generate_to(
        shells::Fish,
        &mut cmd, // We need to specify what generator to use
        "nh",  // We need to specify the bin name manually
        &outdir,   // We need to specify where to write to
    )?;
    println!("cargo:warning=Built Bash completions to {fish_output:?}");


    Ok(())
}
