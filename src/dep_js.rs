use std::{collections::HashMap, process::Command};

use serde_json::Value;

/**
 *  execute npm list --json
 *  parse json
 *  return list of packages
 */
pub fn list(work_dir: &str) -> HashMap<String, String> {
    let mut cmd = Command::new("npm");
    cmd.arg("list")
        .arg("--json")
        .current_dir(work_dir.to_string() + "/repo");
    let output = cmd.output().unwrap();
    let json = String::from_utf8(output.stdout).unwrap();
    let json: Value = serde_json::from_str(&json).unwrap();
    let deps = json["dependencies"].as_object().unwrap();

    let mut packages = HashMap::new();
    for (name, dep) in deps {
        let version = dep["version"].as_str().unwrap();
        packages.insert(name.to_string(), version.to_string());
    }
    packages
}

pub fn add(work_dir: &str, name: &str, version: Option<&str>) -> Result<(), std::io::Error> {
    let mut cmd = Command::new("npm");
    cmd.arg("install");
    if version.is_none() {
        cmd.arg(name);
    } else {
        cmd.arg(format!("{}@{}", name, version.as_ref().unwrap()));
    }
    cmd.current_dir(work_dir.to_string() + "/repo");
    let output = cmd.output()?;
    if !output.status.success() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::Other,
            String::from_utf8(output.stderr).unwrap(),
        ));
    }

    Ok(())
}

pub fn rm(work_dir: &str, name: &str) -> Result<(), std::io::Error> {
    let mut cmd = Command::new("npm");
    cmd.arg("uninstall");
    cmd.arg(name);
    cmd.current_dir(work_dir.to_string() + "/repo");
    let output = cmd.output()?;
    if !output.status.success() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::Other,
            String::from_utf8(output.stderr).unwrap(),
        ));
    }

    Ok(())
}
