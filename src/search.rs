use crate::*;
use color_eyre::eyre::Context;
use interface::SearchArgs;
use owo_colors::OwoColorize;
use serde_json::{json, Value};
use std::{collections::HashMap, ops::Deref, process::Command, time::Instant};
use tracing::{debug, info, trace};

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

#[derive(Debug, Deserialize)]
struct License {
    url: String,
    fullName: String,
}

impl NHRunnable for SearchArgs {
    fn run(&self) -> Result<()> {
        trace!("args: {self:?}");

        let query = json!({
          "from": 0,
          "size": self.limit,
          "sort": [
            {
              "_score": "desc",
              "package_attr_name": "desc",
              "package_pversion": "desc"
            }
          ],
          "aggs": {
            "package_attr_set": {
              "terms": {
                "field": "package_attr_set",
                "size": 20
              }
            },
            "package_license_set": {
              "terms": {
                "field": "package_license_set",
                "size": 20
              }
            },
            "package_maintainers_set": {
              "terms": {
                "field": "package_maintainers_set",
                "size": 20
              }
            },
            "package_platforms": {
              "terms": {
                "field": "package_platforms",
                "size": 20
              }
            },
            "all": {
              "global": {},
              "aggregations": {
                "package_attr_set": {
                  "terms": {
                    "field": "package_attr_set",
                    "size": 20
                  }
                },
                "package_license_set": {
                  "terms": {
                    "field": "package_license_set",
                    "size": 20
                  }
                },
                "package_maintainers_set": {
                  "terms": {
                    "field": "package_maintainers_set",
                    "size": 20
                  }
                },
                "package_platforms": {
                  "terms": {
                    "field": "package_platforms",
                    "size": 20
                  }
                }
              }
            }
          },
          "query": {
            "bool": {
              "filter": [
                {
                  "term": {
                    "type": {
                      "value": "package",
                      "_name": "filter_packages"
                    }
                  }
                },
                {
                  "bool": {
                    "must": [
                      {
                        "bool": {
                          "should": []
                        }
                      },
                      {
                        "bool": {
                          "should": []
                        }
                      },
                      {
                        "bool": {
                          "should": []
                        }
                      },
                      {
                        "bool": {
                          "should": []
                        }
                      }
                    ]
                  }
                }
              ],
              "must": [
                {
                  "dis_max": {
                    "tie_breaker": 0.7,
                    "queries": [
                      {
                        "multi_match": {
                          "type": "cross_fields",
                          "query": self.query,
                          "analyzer": "whitespace",
                          "auto_generate_synonyms_phrase_query": false,
                          "operator": "and",
                          "_name": "multi_match_xd",
                          "fields": [
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
                            "flake_name.*^0.3"
                          ]
                        }
                      },
                      {
                        "wildcard": {
                          "package_attr_name": {
                            "value": "*xd*",
                            "case_insensitive": true
                          }
                        }
                      }
                    ]
                  }
                }
              ]
            }
          }
        });

        let client = reqwest::blocking::Client::new();

        let req = client
            .post("https://search.nixos.org/backend/latest-42-nixos-23.11/_search")
            .json(&query)
            .header("User-Agent", format!("nh/{}", crate::NH_VERSION))
            .basic_auth("aWVSALXpZv", Some("X8gPHnzL52wFEekuxsfQ9cSh"))
            .build()
            .context("building search query")?;

        debug!(?req);

        let then = Instant::now();
        let response = client
            .execute(req)
            .context("querying the elasticsearch API")?;
        let elapsed = then.elapsed();
        debug!(?elapsed, "took");
        debug!(?response);

        let search: SearchResponse = response
            .json()
            .context("parsing response into the elasticsearch format")?;
        debug!(?search);

        let x = search
            .documents::<SearchResult>()
            .context("parsing search document")?;

        for elem in x.iter().rev() {
            trace!("{elem:#?}");
            println!(
                "{} ({})",
                elem.package_attr_name.blue(),
                elem.package_pversion.green()
            );
            if let Some(ref description) = elem.package_description {
              println!(" {}", description);
            }
            println!();
        }

        Ok(())
    }
}
