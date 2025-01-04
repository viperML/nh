use std::fs;
use std::path::{Path, PathBuf};
use std::process;

use chrono::{DateTime, Local, TimeZone, Utc};
use tracing::debug;

#[derive(Debug)]
pub struct GenerationInfo {
    /// Number of a generation
    pub generation: String,

    /// Date on switch a generation was built
    pub date: String,

    /// NixOS version derived from `nixos-version`
    pub nixos_version: String,

    /// Version of the bootable kernel for a given generation
    pub kernel_version: String,

    /// Revision for a configuration. This will be the value
    /// set in `config.system.configurationRevision`
    pub configuration_revision: String,

    /// Specialisations, if any.
    pub specialisations: Vec<String>,

    /// Whether a given generation is the current one.
    pub current: bool,
}

pub fn from_dir(generation_dir: &Path) -> Option<u64> {
    generation_dir
        .file_name()
        .and_then(|os_str| os_str.to_str())
        .and_then(|generation_base| {
            let no_link_gen = generation_base.trim_end_matches("-link");
            no_link_gen
                .rsplit_once('-')
                .and_then(|(_, gen)| gen.parse::<u64>().ok())
        })
}

pub fn describe(generation_dir: &Path, current_profile: &Path) -> Option<GenerationInfo> {
    let generation_number = from_dir(generation_dir)?;
    let nixos_version = fs::read_to_string(generation_dir.join("nixos-version"))
        .unwrap_or_else(|_| "Unknown".to_string());
    let kernel_dir = generation_dir
        .join("kernel")
        .canonicalize()
        .ok()
        .and_then(|path| path.parent().map(|p| p.to_path_buf()))
        .unwrap_or_else(|| PathBuf::from("Unknown"));

    let kernel_version = fs::read_dir(kernel_dir.join("lib/modules"))
        .map(|entries| {
            let mut versions = vec![];
            for entry in entries.filter_map(Result::ok) {
                if let Some(name) = entry.file_name().to_str() {
                    versions.push(name.to_string());
                }
            }
            versions.join(", ")
        })
        .unwrap_or_else(|_| "Unknown".to_string());

    let configuration_revision = {
        let nixos_version_path = generation_dir.join("sw/bin/nixos-version");
        if nixos_version_path.exists() {
            process::Command::new(nixos_version_path)
                .arg("--configuration-revision")
                .output()
                .ok()
                .and_then(|output| String::from_utf8(output.stdout).ok())
                .unwrap_or_default()
                .trim()
                .to_string()
        } else {
            String::new()
        }
    };

    let build_date = fs::metadata(generation_dir)
        .and_then(|metadata| metadata.created().or_else(|_| metadata.modified()))
        .map(|system_time| {
            let duration = system_time
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default();
            DateTime::<Utc>::from(std::time::UNIX_EPOCH + duration).to_rfc3339()
        })
        .unwrap_or_else(|_| "Unknown".to_string());

    let specialisations = {
        let specialisation_path = generation_dir.join("specialisation");
        if specialisation_path.exists() {
            fs::read_dir(specialisation_path)
                .map(|entries| {
                    entries
                        .filter_map(|entry| {
                            entry
                                .ok()
                                .and_then(|e| e.file_name().to_str().map(String::from))
                        })
                        .collect::<Vec<String>>()
                })
                .unwrap_or_default()
        } else {
            vec![]
        }
    };

    let current = generation_dir
        .file_name()
        .map(|name| name == current_profile.file_name().unwrap_or_default())
        .unwrap_or(false);

    Some(GenerationInfo {
        generation: generation_number.to_string(),
        date: build_date,
        nixos_version,
        kernel_version,
        configuration_revision,
        specialisations,
        current,
    })
}

pub fn print_info(generations: Vec<GenerationInfo>) {
    // Get path information for the *current generation* from /run/current-system
    // and split it by whitespace to get the size (second part). This should be
    // safe enough.
    let path_info = process::Command::new("nix")
        .arg("path-info")
        .arg("-Sh")
        .arg("/run/current-system")
        .output();

    let closure = if let Ok(output) = path_info {
        let size_info = String::from_utf8_lossy(&output.stdout);
        let size = size_info.split_whitespace().nth(1).unwrap_or("Unknown");
        size.to_string()
    } else {
        "Unknown".to_string()
    };

    let current_generation = generations
        .iter()
        .max_by_key(|gen| gen.generation.parse::<u64>().unwrap_or(0));

    if let Some(current) = current_generation {
        println!("NixOS {}", current.nixos_version);
    } else {
        println!("No generations found!");
    }

    println!("Closure Size: {}", closure);
    println!("List of Generations");

    for generation in generations.iter() {
        let date_str = &generation.date;
        let date = DateTime::parse_from_rfc3339(date_str)
            .map(|dt| dt.with_timezone(&Local))
            .unwrap_or_else(|err| {
                eprintln!(
                    "Failed to parse date `{}` with error: {}. Using default date.",
                    date_str, err
                );
                Local.timestamp_opt(0, 0).unwrap() // default to Unix epoch
            });
        let formatted_date = date.format("%Y-%m-%d %H:%M:%S").to_string();
        let specialisations = generation.specialisations.join(" ");

        println!(
            "{} {} {} {} {} {}",
            generation.generation,
            formatted_date,
            generation.nixos_version,
            generation.kernel_version,
            generation.configuration_revision,
            specialisations
        );
    }
}
