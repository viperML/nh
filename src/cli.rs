use clap;

#[derive(clap::Parser, Debug, )]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true)]
pub struct NHParser {
   #[command(subcommand)]
   command: NHSwitch
}

#[derive(clap::Subcommand, Debug)]
enum NHSwitch {
    Switch {
        dry_run: Option<bool>
    },
    Boot {
        name: Option<bool>
    }
}
