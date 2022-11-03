use crate::interface;

impl interface::NHCommand {
    pub fn run(&self) {
        match self {
            interface::NHCommand::Switch(args) => args.rebuild(interface::RebuildType::Switch),
            interface::NHCommand::Boot(args) => args.rebuild(interface::RebuildType::Boot),
            interface::NHCommand::Test(args) => args.rebuild(interface::RebuildType::Test),
            variant => todo!("nh command not implemented {variant:?}"),
        }
    }
}
