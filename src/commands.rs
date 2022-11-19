use log::{error, trace};

use crate::interface::{self, NHCommand};

pub trait NHRunnable {
    fn run(&self) -> anyhow::Result<()>;
}

impl NHRunnable for interface::NHCommand {
    fn run(&self) -> anyhow::Result<()> {
        match self {
            NHCommand::Os(a) => a.run(),
            s => todo!("Subcommand {s:?} not yet implemented!"),
        }
    }
}

// impl interface::NHCommand {
//     pub fn run(&self) -> anyhow::Result<()> {
        // match self {
        //     // Switch(a) | Test(a) | Boot(a) => match a.rebuild(self.rebuild_type().unwrap()) {
        //     //     Ok(_) => trace!("OK"),
        //     //     Err(RunError::NoConfirm) => trace!("OK"),
        //     //     Err(RunError::SpecialisationError(s)) => {
        //     //         error!("Specialisation \"{}\" doesn't exist!", s);
        //     //         error!("Use the --specialisation flag to set the correct one");
        //     //     },
        //     //     Err(why) => error!("Error while running! {:?}", why),
        //     // },
        //     interface::NHCommand::Clean(a) => a.clean(),
        //     interface::NHCommand::Search(a) => a.search(),
        //     variant => todo!("nh command not implemented {variant:?}"),
        // }

        // Ok(())
    // }

    // pub fn rebuild_type(&self) -> Option<interface::RebuildType> {
    //     match self {
    //         // Boot(_) => Some(interface::RebuildType::Boot),
    //         // Switch(_) => Some(interface::RebuildType::Switch),
    //         // Test(_) => Some(interface::RebuildType::Test),
    //         _ => None,
    //     }
    // }
// }


impl NHRunnable for interface::OsArgs {
    fn run(&self) -> anyhow::Result<()> {
        todo!()
    }
}

// impl NHRunnable for interface::SearchArgs {
//     fn run(&self) -> anyhow::Result<()> {
//         todo!()
//     }
// }

// impl NHRunnable for interface::HomeArgs {
//     fn run(&self) -> anyhow::Result<()> {
//         todo!()
//     }
// }

// impl NHRunnable for interface::CleanArgs {
//     fn run(&self) -> anyhow::Result<()> {
//         todo!()
//     }
// }
