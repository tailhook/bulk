use std::process::Command;

use config::Config;


#[derive(Debug)]
pub struct Metadata {
    pub name: String,
    pub architecture: String,
    pub short_description: String,
    pub long_description: String,
    pub version: String,
}

fn run_script(cmdtext: &String) -> Result<String, String> {
    let mut cmd = Command::new("/bin/sh");
    cmd.arg("-c");
    cmd.arg(cmdtext);
    match cmd.output() {
        Err(e) => Err(format!("Error executing command {:?}: {}", cmd, e)),
        Ok(output) => {
            if output.status.success() {
                String::from_utf8(output.stdout)
                .map(|x| x.trim().to_string())
                .map_err(|e| format!("Error executing command {:?}: \
                    error decoding output: {}", cmd, e))
            } else {
                 Err(format!("Error executing command {:?}: {:?}",
                    cmd, output.status))
            }
        }
    }
}

pub fn populate(config: &Config) -> Result<Metadata, String> {
    // TODO(tailhook) allow multiple errors at once
    let name = if let Some(ref x) = config.metadata.name { x.clone() }
        else {
            try!(config.scripts.name.as_ref()
                .ok_or(format!("No name specified and no script found"))
                .and_then(run_script))
        };
    let arch = if let Some(ref x) = config.metadata.architecture { x.clone() }
        else {
            try!(config.scripts.architecture.as_ref()
                .ok_or(format!("No architecture specified and no script found"))
                .and_then(run_script))
        };
    let descr = if let Some(ref x) = config.metadata.short_description {
            x.clone()
        } else if let Some(ref cmd) = config.scripts.short_description {
            try!(run_script(cmd))
        } else {
            name.clone()
        };
    let descr = descr.replace("\n", " ");
    let long = if let Some(ref x) = config.metadata.long_description {
            x.clone()
        } else if let Some(ref cmd) = config.scripts.long_description {
            try!(run_script(cmd))
        } else {
            descr.clone()
        };
    let mut version = if let Some(ref x) = config.metadata.version {
            x.clone()
        } else {
            try!(config.scripts.version.as_ref()
                .ok_or(format!("No version specified and no script found"))
                .and_then(run_script))
        };
    // TODO(tailhook) validate version better
    if version.starts_with("v") {
        version.remove(0);
    }
    Ok(Metadata {
        name: name,
        architecture: arch,
        short_description: descr,
        long_description: long,
        version: version,
    })
}
