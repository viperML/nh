use log::trace;
use subprocess::Redirection;

use crate::{
    commands::{mk_temp, run_command, NHRunnable},
    interface::{FlakeRef, HomeArgs, HomeRebuildArgs, HomeSubcommand},
};

#[derive(Debug)]
enum HomeRebuildError {
    OutputName,
    NoConfirm,
}

impl std::fmt::Display for HomeRebuildError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl std::error::Error for HomeRebuildError {}

impl NHRunnable for HomeArgs {
    fn run(&self) -> anyhow::Result<()> {
        // self.subcommand
        match &self.subcommand {
            HomeSubcommand::Switch(args) => args.rebuild()?,
            HomeSubcommand::Info => home_info()?,
        }

        Ok(())
    }
}

impl HomeRebuildArgs {
    fn rebuild(&self) -> anyhow::Result<()> {
        let out_link = mk_temp("/tmp/nh/home-result-");

        let output = get_home_output(&self.flakeref)?;
        trace!("{output}");

        let cmd_build = vec![
            "nix",
            "build",
            "--out-link",
            &out_link,
            &format!(
                "{}#homeConfigurations.{}.config.home.activationPackage",
                &self.flakeref, output
            ),
        ]
        .join(" ");

        let username = std::env::var("USER").expect("Couldn't get username");

        run_command(&cmd_build, self.dry, Some("Building configuration"))?;

        run_command(
            &vec![
                "nvd",
                "diff",
                &format!("/nix/var/nix/profiles/per-user/{}/home-manager", &username),
                &out_link,
            ]
            .join(" "),
            self.dry,
            Some("Comparing changes"),
        )?;

        if self.ask {
            let confirmation = dialoguer::Confirm::new()
                .with_prompt("Apply the config?")
                .default(false)
                .interact()?;

            if !confirmation {
                return Err(HomeRebuildError::NoConfirm.into());
            }
        }

        run_command(
            &vec![format!("{}/activate", out_link)].join(" "),
            self.dry,
            Some("Activating"),
        )?;

        Ok(())
    }
}

fn home_info() -> anyhow::Result<()> {
    Ok(())
}

fn get_home_output(flakref: &FlakeRef) -> Result<String, HomeRebuildError> {
    // Replicate these heuristics
    // https://github.com/nix-community/home-manager/blob/433e8de330fd9c157b636f9ccea45e3eeaf69ad2/home-manager/home-manager#L110

    let hostname = hostname::get()
        .expect("Couldn't get hostname")
        .into_string()
        .unwrap();

    let username = std::env::var("USER").expect("Couldn't get username");

    let full_flakeref_hostname = format!("{}#homeConfigurations", flakref);

    let query = format!(r#" x: x ? "{}@{}" "#, &username, &hostname);

    let args_check = vec!["eval", &full_flakeref_hostname, "--apply", &query];

    trace!("{args_check:?}");
    let _output = subprocess::Exec::cmd("nix")
        .args(&args_check)
        .stdout(Redirection::Pipe)
        .capture()
        .map_err(|_| HomeRebuildError::OutputName)?
        .stdout_str();

    let output = _output.trim();

    trace!("{}", output);

    match output {
        "true" => Ok(format!("{}@{}", &username, &hostname)),
        "false" => Ok(username.to_string()),
        _ => Err(HomeRebuildError::OutputName),
    }
}
