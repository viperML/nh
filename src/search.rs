use std::{collections::HashMap, ops::Deref, process::Command};
use tracing::trace;
use crate::*;
use interface::SearchArgs;

#[derive(Debug, serde::Deserialize)]
struct RawEntry<'a> {
    description: &'a str,
    pname: &'a str,
    version: &'a str,
}

type RawResults<'a> = HashMap<&'a str, RawEntry<'a>>;

impl NHRunnable for SearchArgs {
    fn run(&self) -> Result<()> {
        trace!("args: {self:?}");

        let results = Command::new("nix")
            .arg("search")
            .arg(self.flake.deref())
            .arg(&self.query)
            .arg("--json")
            .output()?;

        let parsed: RawResults = serde_json::from_slice(&results.stdout)?;

        trace!("{:?}", parsed);

        todo!();
    }
}
