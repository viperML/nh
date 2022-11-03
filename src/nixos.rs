use clean_path::Clean;
use log::{debug, info, warn};
use rand::distributions::Alphanumeric;
use rand::{thread_rng, Rng};

use crate::interface;

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
                "{}#nixosConfigurations.{}.config.system.build.toplevel",
                &flake_clean.to_string_lossy(),
                &hostname.to_string_lossy()
            ),
        ]
        .join(" ");

        // let foo: Result<(), RunError> = {
        //     run_command(&cmd, self.dry)?;
        //     Ok(())
        // };
        let output = run_command(&cmd, self.dry);

        match output {
            Err(why) => warn!("Failed to run command! {:?}", why),
            Ok(_) => debug!("OK")
        }

        todo!("rebuild not implemented!");
    }
}
