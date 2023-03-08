use std::{collections::HashMap, io, process::Command};

use serde_json::Value;

/**
 * execute pip3 list --format json
 */
pub fn list() -> HashMap<String, String> {
    let output = Command::new("pip3")
        .arg("list")
        .arg("--format")
        .arg("json")
        .output()
        .expect("failed to execute process");
    let stdout = String::from_utf8(output.stdout).unwrap();
    let json: Value = serde_json::from_str(&stdout).unwrap();
    let mut map = HashMap::new();
    for item in json.as_array().unwrap() {
        let name = item["name"].as_str().unwrap().to_string();
        let version = item["version"].as_str().unwrap().to_string();
        map.insert(name, version);
    }
    map
}

pub fn add(name: &str, version: Option<&str>) -> Result<(), io::Error> {
    let mut cmd = Command::new("pip3");
    cmd.arg("install");
    cmd.arg(name);
    if let Some(version) = version {
        cmd.arg(format!("=={}", version));
    }
    let output = cmd.output()?;
    if !output.status.success() {
        return Err(io::Error::new(
            io::ErrorKind::Other,
            String::from_utf8(output.stderr).unwrap(),
        ));
    }
    Ok(())
}

pub fn rm(name: &str) -> Result<(), io::Error> {
    let mut cmd = Command::new("pip3");
    cmd.arg("uninstall");
    cmd.arg(name);
    cmd.arg("-y");
    let output = cmd.output()?;
    if !output.status.success() {
        return Err(io::Error::new(
            io::ErrorKind::Other,
            String::from_utf8(output.stderr).unwrap(),
        ));
    }
    Ok(())
}
