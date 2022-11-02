use crate::interface;

use log::{debug, info};


// pub fn rebuild(args: &cli::RebuildArgs, rebuild_type: crate::cli::RebuildType) {
//     println!("{}", args.dry);
//     todo!();
// }

impl interface::RebuildArgs {
    pub fn rebuild(&self, rebuild_type: interface::RebuildType) {
        todo!()
    }
}

impl interface::NHCommand {
    pub fn run(&self) {
        match self {
            interface::NHCommand::Switch(r) => {
                r.rebuild(interface::RebuildType::Switch);
            },
            // NHCommand::Boot(r) => {
            //     nixos::rebuild(r, RebuildType::Boot)
            // },
            // NHCommand::Test(r) => {
            //     nixos::rebuild(r, RebuildType::Test)
            // },
            variant => todo!("{variant:?}")
        }
    }
}
