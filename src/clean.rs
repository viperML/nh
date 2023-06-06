use std::ffi::OsStr;
use std::fs::{self};
use std::path::{Path, PathBuf};

use color_eyre::Result;
use derive_builder::Builder;
use log::{info, trace, warn};

use crate::interface::NHRunnable;
use crate::interface::{CleanArgs, CleanMode, CleanProxy};

// Reference: https://github.com/NixOS/nix/blob/master/src/nix-collect-garbage/nix-collect-garbage.cc

#[async_trait::async_trait]
impl NHRunnable for CleanMode {
    async fn run(&self) -> Result<()> {
        match self {
            CleanMode::Info => todo!(),
            CleanMode::User(args) => clean_user(args).await,
            CleanMode::All(args) => clean_user(args).await,
        }
    }
}

async fn clean_user(args: &CleanArgs) -> Result<()> {
    let profile = Profile {
        parent: Path::new("/home/ayats/.local/state/nix/profiles"),
        name: OsStr::new("profile"),
    };

    trace!("profile: {:?}", profile);

    Ok(())
}

#[derive(Debug, Clone)]
struct Profile<'profile> {
    parent: &'profile Path,
    name: &'profile OsStr,
}
