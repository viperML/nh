use crate::*;
use color_eyre::eyre::Context;
use interface::SearchArgs;

use std::{process::Stdio, time::Instant};
use tracing::{debug, trace};

use elasticsearch_dsl::*;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
#[allow(non_snake_case, dead_code)]
struct SearchResult {
    // r#type: String,
    package_attr_name: String,
    package_attr_set: String,
    package_pname: String,
    package_pversion: String,
    package_platforms: Vec<String>,
    package_outputs: Vec<String>,
    package_default_output: Option<String>,
    package_programs: Vec<String>,
    // package_license: Vec<License>,
    package_license_set: Vec<String>,
    // package_maintainers: Vec<HashMap<String, String>>,
    package_description: Option<String>,
    package_longDescription: Option<String>,
    package_hydra: (),
    package_system: String,
    package_homepage: Vec<String>,
    package_position: Option<String>,
}

macro_rules! print_hyperlink {
    ($text:expr, $link:expr) => {
        print!("\x1b]8;;{}\x07", $link);
        print!("{}", $text.underline());
        println!("\x1b]8;;\x07");
    };
}

impl NHRunnable for SearchArgs {
    fn run(&self) -> Result<()> {
        trace!("args: {self:?}");

        let nixpkgs_path = std::thread::spawn(|| {
            std::process::Command::new("nix")
                .stderr(Stdio::inherit())
                .args(["eval", "nixpkgs#path"])
                .output()
        });

        // let mut nixpkgs_path = std::process::Command::new("nix")
        // .context("Evaluating the nixpkgs path for results positions")?;

        let query = Search::new().from(0).size(self.limit).query(
            Query::bool().filter(Query::term("type", "package")).must(
                Query::dis_max()
                    .tie_breaker(0.7)
                    .query(
                        Query::multi_match(
                            [
                                "package_attr_name^9",
                                "package_attr_name.*^5.3999999999999995",
                                "package_programs^9",
                                "package_programs.*^5.3999999999999995",
                                "package_pname^6",
                                "package_pname.*^3.5999999999999996",
                                "package_description^1.3",
                                "package_description.*^0.78",
                                "package_longDescription^1",
                                "package_longDescription.*^0.6",
                                "flake_name^0.5",
                                "flake_name.*^0.3",
                            ],
                            self.query.as_str(),
                        )
                        .r#type(TextQueryType::CrossFields)
                        .analyzer("whitespace")
                        .auto_generate_synonyms_phrase_query(false)
                        .operator(Operator::And),
                    )
                    .query(
                        Query::wildcard("package_attr_name", format!("*{}*", self.query))
                            .case_insensitive(true),
                    ),
            ),
        );

        println!(
            "Querying search.nixos.org, with channel {}...",
            self.channel
        );
        let then = Instant::now();

        let client = reqwest::blocking::Client::new();
        let req = client
            // I guess 42 is the version of the backend API
            // TODO: have a GH action or something check if they updated this thing
            .post(format!(
                "https://search.nixos.org/backend/latest-42-{}/_search",
                self.channel
            ))
            .json(&query)
            .header("User-Agent", format!("nh/{}", crate::NH_VERSION))
            // Hardcoded upstream
            // https://github.com/NixOS/nixos-search/blob/744ec58e082a3fcdd741b2c9b0654a0f7fda4603/frontend/src/index.js
            .basic_auth("aWVSALXpZv", Some("X8gPHnzL52wFEekuxsfQ9cSh"))
            .build()
            .context("building search query")?;

        debug!(?req);

        let response = client
            .execute(req)
            .context("querying the elasticsearch API")?;
        let elapsed = then.elapsed();
        debug!(?elapsed);
        trace!(?response);
        println!("Took {}ms", elapsed.as_millis());
        println!("Most relevant results at the end");
        println!();

        let parsed_response: SearchResponse = response
            .json()
            .context("parsing response into the elasticsearch format")?;
        trace!(?parsed_response);

        let documents = parsed_response
            .documents::<SearchResult>()
            .context("parsing search document")?;

        let hyperlinks = supports_hyperlinks::supports_hyperlinks();
        debug!(?hyperlinks);

        let nixpkgs_path = String::from_utf8(
            nixpkgs_path
                .join()
                .unwrap()
                .context("Evaluating the nixpkgs path location")?
                .stdout,
        )
        .unwrap();

        for elem in documents.iter().rev() {
            println!();
            use owo_colors::OwoColorize;
            trace!("{elem:#?}");

            print!("{}", elem.package_attr_name.blue());
            let v = &elem.package_pversion;
            if !v.is_empty() {
                print!(" ({})", v.green());
            }

            println!();

            if let Some(ref desc) = elem.package_description {
                let desc = desc.replace('\n', " ");
                for line in textwrap::wrap(&desc, textwrap::Options::with_termwidth()) {
                    println!("  {}", line);
                }
            }

            for url in elem.package_homepage.iter() {
                print!("  Homepage: ");
                if hyperlinks {
                    print_hyperlink!(url, url);
                } else {
                    println!("{}", url);
                }
            }

            if let Some(position) = &elem.package_position {
                print!("  Position: ");
                if hyperlinks {
                    let postion_trimmed = position
                        .split(':')
                        .next()
                        .expect("Removing line number from position");

                    print_hyperlink!(position, format!("file://{nixpkgs_path}/{postion_trimmed}"));
                } else {
                    println!("{}", position);
                }
            }
        }

        Ok(())
    }
}
