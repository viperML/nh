use core::fmt;
use std::fmt::Write;

#[derive(Debug, Clone)]
pub enum Installable {
    Flake(FlakeInstallable),
}

#[derive(Debug, Clone)]
pub struct FlakeInstallable {
    pub reference: String,
    pub attribute: Vec<String>,
}

impl From<&str> for Installable {
    fn from(value: &str) -> Self {
        // FIXME
        let x = value.split_once('#').unwrap();
        Installable::flake(x.0, &x.1.split('.').collect::<Vec<_>>())
    }
}

impl Installable {
    pub fn flake<S>(reference: S, attribute: &[S]) -> Self
    where
        S: AsRef<str>,
    {
        Installable::Flake(FlakeInstallable {
            reference: reference.as_ref().to_string(),
            attribute: attribute.iter().map(|s| s.as_ref().to_string()).collect(),
        })
    }

    pub fn to_args(&self) -> Vec<String> {
        let mut res = Vec::new();

        match &self {
            Installable::Flake(flake) => {
                let mut f = String::new();
                write!(f, "{}", flake.reference).unwrap();

                if !flake.attribute.is_empty() {
                    write!(f, "#").unwrap();

                    let mut first = true;

                    for elem in &flake.attribute {
                        if !first {
                            write!(f, ".").unwrap();
                        }

                        if elem.contains('.') {
                            write!(f, r#""{}""#, elem).unwrap();
                        } else {
                            write!(f, "{}", elem).unwrap();
                        }

                        first = false;
                    }

                    res.push(f);
                }
            }
        }

        return res;
    }
}

impl fmt::Display for Installable {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut first = true;

        for elem in self.to_args() {
            if !first {
                write!(f, " ")?;
            } else {
                first = false;
            }

            write!(f, "{}", elem)?;
        }

        Ok(())
    }
}

#[test]
fn test_display() {
    let installable = Installable::flake(".", &["foo", "bar.local", "baz"]);

    let args = installable.to_args();
    assert_eq!(args, vec![String::from(".#foo.\"bar.local\".baz")]);

    let displayed = format!("{}", installable);
    assert_eq!(".#foo.\"bar.local\".baz", displayed);
}
