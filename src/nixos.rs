use clean_path::Clean;
use log::info;
use rand::distributions::Alphanumeric;
use rand::{thread_rng, Rng};

use crate::interface;

const SYSTEM_PROFILE: &str = "/nix/var/nix/profiles/system";

fn run_command(cmd: &str, dry: bool) -> std::io::Result<()> {
    // let output = std::process::Command::new()
    // info!("{arg0}");
    info!("{cmd}");
    if !dry {
        let mut argv = cmd.split(" ");
        let arg0 = argv.nth(0).expect("Bad command");
        let output = std::process::Command::new(arg0).args(argv).spawn()?;
    };

    Ok(())
}

impl interface::RebuildArgs {
    pub fn rebuild(&self, rebuild_kind: interface::RebuildType) {
        let hostname = hostname::get().expect("Failed to get hostname!");

        let flake_clean = self.flake.clean();

        let suffix_bytes = thread_rng()
            .sample_iter(&Alphanumeric)
            .take(10)
            .collect::<Vec<_>>();
        let suffix = String::from_utf8_lossy(&suffix_bytes);

        let cmd = vec![
            "nix",
            "build",
            "--out-link",
            &format!("/tmp/nh/result-{}", suffix),
            "--profile",
            SYSTEM_PROFILE,
            &format!(
                "{}#{}",
                &flake_clean.to_string_lossy(),
                &hostname.to_string_lossy()
            ),
        ]
        .join(" ");

        run_command(&cmd, self.dry);

        todo!("rebuild not implemented!");
    }
}
