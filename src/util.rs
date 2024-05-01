extern crate semver;

use color_eyre::{eyre, Result};
use semver::Version;

use std::fs::OpenOptions;
use std::io;
use std::process::Command;
use std::str;

/// Compares two semantic versions and returns their order.
///
/// This function takes two version strings, parses them into `semver::Version` objects, and compares them.
/// It returns an `Ordering` indicating whether the current version is less than, equal to, or
/// greater than the target version.
///
/// # Arguments
///
/// * `current` - A string slice representing the current version.
/// * `target` - A string slice representing the target version to compare against.
///
/// # Returns
///
/// * `Result<std::cmp::Ordering>` - The comparison result.
pub fn compare_semver(current: &str, target: &str) -> Result<std::cmp::Ordering> {
    let current = Version::parse(current)?;
    let target = Version::parse(target)?;

    Ok(current.cmp(&target))
}

/// Retrieves the installed Nix version as a string.
///
/// This function executes the `nix --version` command, parses the output to extract the version string,
/// and returns it. If the version string cannot be found or parsed, it returns an error.
///
/// # Returns
///
/// * `Result<String>` - The Nix version string or an error if the version cannot be retrieved.
pub fn get_nix_version() -> Result<String> {
    let output = Command::new("nix").arg("--version").output()?;

    let output_str = str::from_utf8(&output.stdout)?;
    let version_str = output_str
        .lines()
        .next()
        .ok_or_else(|| eyre::eyre!("No version string found"))?;

    // Extract the version substring using a regular expression
    let re = regex::Regex::new(r"\d+\.\d+\.\d+")?;
    if let Some(captures) = re.captures(version_str) {
        let version = captures
            .get(0)
            .ok_or_else(|| eyre::eyre!("No version match found"))?
            .as_str();
        return Ok(version.to_string());
    }

    Err(eyre::eyre!("Failed to extract version"))
}

/// Checks if the current user has permission to read, write, or create files at the specified path.
///
/// This function attempts to open a file at the given path with read, write permissions, but without
/// creating it if it doesn't exist. It returns an error if the operation fails due to insufficient permissions.
///
/// # Parameters
///
/// * `path` - A string slice representing the path to the file or directory to check permissions for.
///
/// # Errors
///
/// Returns an `io::Error` if the operation fails. The specific error kind will be `PermissionDenied`, indicating
/// that the current user lacks the necessary permissions to perform the requested operation.
pub fn check_perms(path: &str) -> io::Result<()> {
    let mut options = OpenOptions::new();
    let file = options.read(true).write(true).create(false).open(path);

    // Match on the result of opening the file
    match file {
        Ok(_) => Ok(()),
        Err(e) => Err(io::Error::new(
            io::ErrorKind::PermissionDenied,
            format!("Cannot update flakes owned by root. Error: {:?}", e),
        )),
    }
}
