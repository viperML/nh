use std::path::PathBuf;

use crate::Result;


#[test]
fn api_design() {
    let host = Local::new();
    let target = LocalRoot::new();
    let system = (host, target);

    let result = system.build("github:NixOS/nixpkgs#hello^out");
    system.run([
        "ln",
        "-vsfT",
        result,
        "/tmp/foo"
    ]);
}

pub trait System {
    fn new() -> Self;

    fn build(&self) -> Result<PathBuf>;
}

pub struct SystemT<Host, Target> {
    host: Host,
    target: Target
}
