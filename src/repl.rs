use std::vec;

use color_eyre::eyre::{bail, Context};
use color_eyre::Result;

use tracing::debug;

use crate::interface::CommonReplArgs;
use crate::*;

/// ReplVariant represents the variant of the target REPL. It should be one of
/// OsRepl or HomeRepl while beig passed to the repl function as an argument.
///
/// Only OsRepl (nh os repl) is supported for the time being!
pub enum ReplVariant {
    OsRepl,
    HomeRepl,
}

impl CommonReplArgs {
    pub fn repl(&self, repl_variant: ReplVariant) -> Result<()> {
        let mut repl_command = vec!["nix", "repl"];

        // Push extra arguments to the repl command BEFORE the installable, is passed
        // to the REPL.
        if !&self.extra_args.is_empty() {
            for arg in &self.extra_args {
                repl_command.push(arg);
            }
        };

        let hostname = match &self.hostname {
            Some(h) => h.to_owned(),
            None => hostname::get().context("Failed to get hostname")?,
        };

        // When (or if) HomeRepl is implemented, this can be changed to a more generic value
        // and made mutable, so that the value is set in the match based on the variant of
        // the REPL. For the time being, I am setting it here to ensure it lives long enough
        // to be borrowed later.
        // P.S. "flakeref" is an incredibly vague name, make sure to change it.
        let flakeref = format!(
            "{}#nixosConfigurations.{}",
            self.flakeref.as_str(),
            hostname.to_string_lossy()
        );

        // TODO: Implement match case for HomeRepl.
        // See nixos.rs for the OsRepl implementation, the interface is now general enough
        // to be reused without friction.
        match repl_variant {
            ReplVariant::OsRepl => repl_command.push(&flakeref),
            ReplVariant::HomeRepl => bail!("OsRepl is not yet supported."),
        }

        debug!("flakeref: {:?}", flakeref);
        debug!("repl_command: {:?}", repl_command);

        commands::CommandBuilder::default()
            .args(repl_command)
            .message("Entering Nix REPL")
            .build()?
            .exec()
            .unwrap();
        Ok(())
    }
}
