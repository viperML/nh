use chrono::{DateTime, Local, TimeZone, Utc};
use serde_json::json;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process;

pub fn generation_from_dir(generation_dir: &Path) -> Option<String> {
    generation_dir
        .file_name()
        .and_then(|os_str| os_str.to_str())
        .map(|generation_base| {
            let no_link_gen = generation_base.trim_end_matches("-link");
            no_link_gen
                .rsplit_once('-')
                .map(|(_, gen)| gen.to_string())
                .unwrap_or_default()
        })
}

pub fn describe_generation(
    generation_dir: &Path,
    current_profile: &Path,
) -> HashMap<String, serde_json::Value> {
    let generation_number = generation_from_dir(generation_dir).unwrap_or_default();
    let nixos_version = fs::read_to_string(generation_dir.join("nixos-version"))
        .unwrap_or_else(|_| "Unknown".to_string());
    let kernel_dir = generation_dir
        .join("kernel")
        .canonicalize()
        .ok()
        .and_then(|path| path.parent().map(|p| p.to_path_buf()))
        .unwrap_or_else(|| PathBuf::from("Unknown"));
    let kernel_version = fs::read_dir(kernel_dir.join("lib/modules"))
        .and_then(|entries| {
            let mut versions = vec![];
            for entry in entries {
                if let Ok(entry) = entry {
                    if let Some(name) = entry.file_name().to_str() {
                        versions.push(name.to_string());
                    }
                }
            }
            Ok(versions.join(", "))
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
    let mut result = HashMap::new();
    result.insert("generation".to_string(), json!(generation_number));
    result.insert("date".to_string(), json!(build_date));
    result.insert("nixosVersion".to_string(), json!(nixos_version));
    result.insert("kernelVersion".to_string(), json!(kernel_version));
    result.insert(
        "configurationRevision".to_string(),
        json!(configuration_revision),
    );
    result.insert("specialisations".to_string(), json!(specialisations));
    result.insert("current".to_string(), json!(current));
    result
}

pub fn print_generations(generations: Vec<HashMap<String, serde_json::Value>>) {
    for generation in generations {
        let date_str = generation["date"].as_str().unwrap_or_default();
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
        let current_str = if generation["current"].as_bool().unwrap_or(false) {
            "current"
        } else {
            ""
        };
        let specialisations = generation["specialisations"]
            .as_array()
            .unwrap_or(&vec![])
            .iter()
            .map(|s| s.as_str().unwrap_or_default())
            .collect::<Vec<_>>()
            .join(" ");
        let tsv_line = format!(
            "{}\t{}\t{}\t{}\t{}\t{}",
            format!(
                "{} {}",
                generation["generation"].as_str().unwrap_or_default(),
                current_str
            ),
            formatted_date,
            generation["nixosVersion"].as_str().unwrap_or_default(),
            generation["kernelVersion"].as_str().unwrap_or_default(),
            generation["configurationRevision"]
                .as_str()
                .unwrap_or_default(),
            specialisations
        );
        println!("{}", tsv_line);
    }
}
