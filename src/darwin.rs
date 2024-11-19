use crate::interface::{DarwinArgs, DarwinRebuildArgs, DarwinReplArgs, DarwinSubcommand};
use crate::Result;

impl DarwinArgs {
    pub fn run(self) -> Result<()> {
        use DarwinRebuildVariant::*;
        match self.subcommand {
            DarwinSubcommand::Switch(args) => args.rebuild(Switch),
            DarwinSubcommand::Build(args) => args.rebuild(Build),
            DarwinSubcommand::Repl(args) => args.run(),
        }
    }
}

enum DarwinRebuildVariant {
    Switch,
    Build,
}

impl DarwinRebuildArgs {
    fn rebuild(self, _variant: DarwinRebuildVariant) -> Result<()> {
        todo!();
    }
}

impl DarwinReplArgs {
    fn run(self) -> Result<()> {
        todo!();
    }
}
