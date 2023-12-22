use once_cell::sync::Lazy;
use regex::Regex;
use tracing::{trace, debug};

use crate::*;

// Reference: https://github.com/NixOS/nix/blob/master/src/nix-collect-garbage/nix-collect-garbage.cc

impl NHRunnable for interface::CleanMode {
    fn run(&self) -> Result<()> {

        match self {
            interface::CleanMode::All(args) => {
                let uid = nix::unistd::Uid::effective();
                trace!(?uid);
                if !uid.is_root() {
                    debug!("nh clean all called as root user, re-executing with sudo");

                }



            },
            interface::CleanMode::User(_) => todo!(),
        }


        Ok(())
    }
}



// static PROFILE_PATTERN: Lazy<Regex> = Lazy::new(|| Regex::new(r"^(.*)-(\d+)-link$").unwrap());

// fn parse_profile(s: &str) -> Option<(&str, u32)> {
//     let captures = PROFILE_PATTERN.captures(s)?;

//     let base = captures.get(1)?.as_str();
//     let number = captures.get(2)?.as_str().parse().ok()?;

//     Some((base, number))
// }

// #[test]
// fn test_parse_profile() {
//     assert_eq!(
//         parse_profile("home-manager-3-link"),
//         Some(("home-manager", 3))
//     );
//     assert_eq!(
//         parse_profile("home-manager-30-link"),
//         Some(("home-manager", 30))
//     );
//     assert_eq!(parse_profile("home-manager"), None);
//     assert_eq!(
//         parse_profile("foo-bar-baz-0-link"),
//         Some(("foo-bar-baz", 0))
//     );
//     assert_eq!(parse_profile("foo-bar-baz-X-link"), None);
// }
