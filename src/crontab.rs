use either::Either;
use regex::Regex;
use serde::{Deserialize, Serialize};
use serde_json;
use std::{io::Write, process::Command};

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct RepoArgs {
    pub whitelist: String,
    pub branch: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ItemArgs {
    pub group: String,
    pub name: String,
    pub repo_args: Option<RepoArgs>,
}

#[derive(Debug, Clone, Serialize)]
pub struct Item {
    pub schedule: String,
    pub cmd: String,
    pub args: Either<ItemArgs, String>,
}

pub fn get() -> Result<Vec<Item>, std::io::Error> {
    let output = Command::new("crontab").arg("-l").output()?;

    let stdout = String::from_utf8(output.stdout).expect("Found invalid UTF-8");
    let stderr = String::from_utf8(output.stderr).expect("Found invalid UTF-8");

    if !stderr.is_empty() {
        return Err(std::io::Error::new(std::io::ErrorKind::Other, stderr));
    }

    let mut blocks = Vec::new();
    let mut block = String::new();
    for line in stdout.lines() {
        block.push_str(line);
        block.push('\n');
        if !line.starts_with("#") && block.len() > 1 {
            block.pop();
            blocks.push(block.clone());
            block.clear();
        }
    }

    let arg_reg = Regex::new(r"@light-dragon: (.*)").unwrap();
    return Ok(blocks
        .iter()
        .map(|block| (block, arg_reg.captures(&block)))
        .map(|(block, cap)| {
            if cap.is_none() {
                return (block, None);
            }
            let cap = cap.unwrap();
            let args = cap.get(1).unwrap().as_str();
            let args: ItemArgs = serde_json::from_str(args).unwrap();
            (block, Some(args))
        })
        .map(|(block, args)| {
            let last_line = block.lines().last().unwrap();
            let parts = last_line.split_ascii_whitespace().collect::<Vec<_>>();
            let schedule = &parts[0..5].join(" ");
            let cmd = &parts[5..].join(" ");
            Item {
                schedule: schedule.to_string(),
                cmd: cmd.to_string(),
                args: args.map_or_else(|| Either::Right(block.to_string()), Either::Left),
            }
        })
        .collect::<Vec<_>>());
}

fn gen_crontab_str(items: Vec<Item>) -> String {
    let mut buf = String::new();

    for item in items {
        let line = if item.args.is_left() {
            let args = serde_json::to_string(&item.args.unwrap_left()).unwrap();
            format!(
                "# @light-dragon: {}\n{} {}\n",
                args, item.schedule, item.cmd
            )
        } else {
            format!("{}\n", item.args.unwrap_right())
        };
        buf += &line;
    }
    buf
}

pub fn set(items: Vec<Item>) -> Result<(), std::io::Error> {
    // write to tmp file
    let tmp_path = "/tmp/light-dragon-crontab";
    let mut tmp_file = std::fs::File::create(tmp_path)?;
    tmp_file.write(gen_crontab_str(items).as_bytes())?;

    // set crontab
    let output = Command::new("crontab").arg(tmp_path).output()?;
    let stderr = String::from_utf8(output.stderr).expect("Found invalid UTF-8");
    if !stderr.is_empty() {
        return Err(std::io::Error::new(std::io::ErrorKind::Other, stderr));
    }

    Ok(())
}
