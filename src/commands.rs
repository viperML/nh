

use log::{debug, warn};

use crate::interface::{self, NHCommand::Boot, NHCommand::Switch, NHCommand::Test};

impl interface::NHCommand {
    pub fn run(&self) {
        match self {
            Switch(a) | Test(a) | Boot(a) => match a.rebuild(self.rebuild_type().unwrap()) {
                Ok(_) => debug!("OK"),
                Err(why) => warn!("Error while running! {:?}", why),
            },
            variant => todo!("nh command not implemented {variant:?}"),
        }
    }

    pub fn rebuild_type(&self) -> Option<interface::RebuildType> {
        match self {
            Boot(_) => Some(interface::RebuildType::Boot),
            Switch(_) => Some(interface::RebuildType::Switch),
            Test(_) => Some(interface::RebuildType::Test),
            _ => None,
        }
    }
}
