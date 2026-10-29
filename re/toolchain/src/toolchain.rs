use kdl::KdlDocument;
use std::collections::HashMap;
use std::fs::read_to_string;
use std::path::Path;
use std::process::Command;
use which::which_in;

#[derive(Debug, Clone)]
pub struct Toolchain {
    pub use_wine: bool,
    pub env: HashMap<String, String>,
}

impl Toolchain {
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self, Box<dyn std::error::Error>> {
        let s = read_to_string(path)?;
        let doc: KdlDocument = s.parse()?;
        let mut tc = Toolchain {
            use_wine: false,
            env: HashMap::new(),
        };
        for node in doc {
            match node.name().value() {
                "wine" => {
                    tc.use_wine = true;
                }
                "env" => {
                    for e in node.entries() {
                        let k = e.name().unwrap().value();
                        let v = e.value().as_string().unwrap();
                        tc.env.insert(k.into(), v.into());
                    }
                }
                n => panic!("unknown node {n}"),
            }
        }
        Ok(tc)
    }

    pub fn command(&self, cmd: &str) -> Result<Command, Box<dyn std::error::Error>> {
        let mut res: Command;
        if self.use_wine {
            res = Command::new("wine");
            res.arg(cmd);
        } else if let Some(path) = self.env.get("PATH") {
            let rcmd = which_in(cmd, Some(path), "/")?;
            res = Command::new(rcmd);
        } else {
            res = Command::new(cmd);
        }
        for (k, v) in self.env.iter() {
            res.env(k, v);
        }
        Ok(res)
    }
}
