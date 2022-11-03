use clean_path::Clean;
use log::{debug, info, warn};
use rand::distributions::Alphanumeric;
use rand::{thread_rng, Rng};

use crate::interface::{self, RebuildType};

const SYSTEM_PROFILE: &str = "/nix/var/nix/profiles/system";

#[derive(Debug)]
enum RunError {
    PopenError,
    ExitError,
}

impl From<subprocess::PopenError> for RunError {
    fn from(_: subprocess::PopenError) -> Self {
        RunError::PopenError
    }
}

fn run_command(cmd: &str, dry: bool) -> Result<(), RunError> {
    // let output = std::process::Command::new()
    // info!("{arg0}");
    info!("{cmd}");
    if !dry {
        let mut argv = cmd.split(" ");
        let arg0 = argv.nth(0).expect("Bad command");
        let output = subprocess::Exec::cmd(arg0)
            .args(&argv.collect::<Vec<_>>())
            .capture()?;

        if ! output.success() {
            return Err(RunError::ExitError)
        }

        debug!("{:?}", output);
    };

    Ok(())
}

impl interface::RebuildArgs {
    pub fn rebuild(&self, rebuild_kind: RebuildType) {
        let hostname = hostname::get().expect("Failed to get hostname!");

        let flake_clean = self.flake.clean();

        let suffix_bytes = thread_rng()
            .sample_iter(&Alphanumeric)
            .take(10)
            .collect::<Vec<_>>();
        let suffix = String::from_utf8_lossy(&suffix_bytes);

        let out_link: &str = &format!("/tmp/nh/result-{}", suffix);

        let cmd_build = vec![
            "nix",
            "build",
            "--out-link",
            out_link,
            "--profile",
            SYSTEM_PROFILE,
            &format!(
                "{}#nixosConfigurations.{}.config.system.build.toplevel",
                &flake_clean.to_string_lossy(),
                &hostname.to_string_lossy()
            ),
        ]
        .join(" ");

        run_command(&cmd_build, self.dry);

        match rebuild_kind {
            RebuildType::Test | RebuildType::Switch => {
                let prefix: String = match self.specialisation {
                    None => "/".to_string(),
                    Some(s) => format!("/specialisation/{}", s),
                };
                let file: &str = &format!("{}/bin/switch-to-configuration", out_link);
                let cmd_activate: String = vec![file, "test"].join(" ");
                run_command(&cmd_activate, self.dry);
            }
            RebuildType::Boot => {}
        }

        todo!("rebuild not implemented!");
    }
}
