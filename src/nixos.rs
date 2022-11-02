use clean_path::Clean;
use log::info;
use rand::distributions::Alphanumeric;
use rand::{thread_rng, Rng};
use std::path::PathBuf;

use crate::interface;

const SYSTEM_PROFILE: &str = "/nix/var/nix/profiles/system";

impl interface::RebuildArgs {
    pub fn rebuild(&self, rebuild_kind: interface::RebuildType) {
        let hostname = hostname::get().expect("Failed to get hostname!");

        let flake_clean = self.flake.clean();

        let suffix_bytes = thread_rng()
            .sample_iter(&Alphanumeric)
            .take(10)
            .collect::<Vec<_>>();
        let suffix = String::from_utf8_lossy(&suffix_bytes);

        let command = vec![
            "nix",
            "build",
            "--out-link",
            &format!("/tmp/nh/result-{}", suffix),
            "--profile",
            SYSTEM_PROFILE,
            &format!("{}#{}", &flake_clean.to_string_lossy(), &hostname.to_string_lossy())
        ]
        .join(" ");

        info!("{command}");

        todo!("rebuild not implemented!");
    }
}

impl interface::NHCommand {
    pub fn run(&self) {
        match self {
            interface::NHCommand::Switch(r) => {
                r.rebuild(interface::RebuildType::Switch);
            }
            interface::NHCommand::Boot(r) => r.rebuild(interface::RebuildType::Boot),
            interface::NHCommand::Test(r) => r.rebuild(interface::RebuildType::Test),
            variant => todo!("nh command not implemented {variant:?}"),
        }
    }
}
