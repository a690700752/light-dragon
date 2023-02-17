use std::{
    collections::HashMap,
    io::{BufRead, Write},
};

pub fn add(work_dir: &str, name: &str, value: &str) -> Result<(), std::io::Error> {
    let mut env_file = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(get_env_path(work_dir))?;
    env_file.write_all(format!("export {}={}\n", name, value).as_bytes())?;
    Ok(())
}

pub fn list(work_dir: &str) -> Result<HashMap<String, String>, std::io::Error> {
    let env_file = std::fs::File::open(get_env_path(work_dir))?;
    let reader = std::io::BufReader::new(env_file);

    let mut map = HashMap::new();
    for line in reader.lines() {
        let line = line?;
        let line = &line["export ".len()..];
        map.insert(
            line.split('=').nth(0).unwrap().to_string(),
            line.split('=').nth(1).unwrap().to_string(),
        );
    }
    Ok(map)
}

pub fn rm(work_dir: &str, name: &str) -> Result<(), std::io::Error> {
    let begin = format!("export {}=", name);
    let env_file = std::fs::File::open(get_env_path(work_dir))?;
    let reader = std::io::BufReader::new(env_file);
    let mut lines: Vec<String> = Vec::new();
    for line in reader.lines() {
        let line = line?;
        if !line.starts_with(&begin) {
            lines.push(line);
        }
    }

    let mut env_file = std::fs::OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open(get_env_path(work_dir))?;
    for line in lines {
        env_file.write_all(format!("{}\n", line).as_bytes())?;
    }
    Ok(())
}

pub fn get_env_path(work_dir: &str) -> String {
    std::path::Path::new(&format!("{}/{}", work_dir, "light-dragon.env"))
        .canonicalize()
        .unwrap()
        .to_str()
        .unwrap()
        .to_string()
}
