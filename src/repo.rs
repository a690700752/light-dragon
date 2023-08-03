use std::io::BufRead;

use either::Either::Left;

use crate::{crontab, env::get_env_path};

const GROUP_REPO: &str = "_repo";

fn filter_by_group<'a>(tabs: &'a Vec<crontab::Item>, group: &str) -> Vec<&'a crontab::Item> {
    tabs.iter()
        .filter(|i| i.args.as_ref().is_left() && i.args.as_ref().unwrap_left().group == group)
        .collect()
}

fn resolve_to_abspath(path: &str) -> Result<String, std::io::Error> {
    std::fs::canonicalize(path).map(|p| p.to_str().unwrap().to_string())
}

pub fn list(tabs: &Vec<crontab::Item>) -> Vec<&crontab::Item> {
    filter_by_group(tabs, GROUP_REPO)
}

pub fn list_tasks<'a>(tabs: &'a Vec<crontab::Item>, name: &str) -> Vec<&'a crontab::Item> {
    filter_by_group(tabs, name)
}

fn get_repo_name(repo: &str) -> String {
    let parts = repo.split("/").collect::<Vec<_>>();
    let name = parts[parts.len() - 1];
    let name = name.split(".").collect::<Vec<_>>();
    name[0].to_string()
}

fn clone_repo(repo: &str, path: &str, branch: &str, force: bool) -> Result<(), std::io::Error> {
    // remove if exists
    if std::path::Path::new(path).exists() {
        if force {
            std::fs::remove_dir_all(path)?;
        } else {
            return Ok(());
        }
    }

    let mut cmd = std::process::Command::new("git");
    cmd.arg("clone");
    cmd.arg(repo);
    cmd.arg("-b");
    cmd.arg(branch);
    cmd.arg(path);
    let o = cmd.output()?;

    if !o.status.success() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::Other,
            String::from_utf8(o.stderr).unwrap(),
        ));
    }

    Ok(())
}

fn find_files_by_regex_helper(
    base_dir: &str,
    dir: &str,
    regex: &regex::Regex,
    files: &mut Vec<String>,
) -> Result<(), std::io::Error> {
    for entry in std::fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();

        // ignore dot files
        if path.file_name().unwrap().to_str().unwrap().starts_with(".") {
            continue;
        }

        if path.is_dir() {
            find_files_by_regex_helper(base_dir, path.to_str().unwrap(), regex, files)?;
        } else {
            let path = path.to_str().unwrap();
            let path = path.replace(base_dir, "");
            let path = path.trim_start_matches("/");
            if regex.is_match(path) {
                files.push(path.to_string());
            }
        }
    }
    Ok(())
}

fn find_files_by_regex(dir: &str, regex: &str) -> Result<Vec<String>, std::io::Error> {
    let mut files = Vec::new();
    let re = regex::Regex::new(regex).unwrap();
    find_files_by_regex_helper(dir, dir, &re, &mut files)?;

    Ok(files)
}

fn has_shebang(file: &str) -> Result<bool, std::io::Error> {
    let file = std::fs::File::open(file)?;
    let reader = std::io::BufReader::new(file);
    for line in reader.lines() {
        let line = line?;
        if line.starts_with("#!") {
            return Ok(true);
        }
    }

    Ok(false)
}

fn gen_launcher(file: &str) -> String {
    // detect by suffix
    let suffix = std::path::Path::new(file)
        .extension()
        .unwrap()
        .to_str()
        .unwrap();
    let suffix = suffix.to_lowercase();

    if suffix == "py" {
        "/usr/bin/env python3"
    } else if suffix == "js" {
        "/usr/bin/env node"
    } else {
        "/bin/sh"
    }
    .to_string()
}

fn find_cron_in_file(file: &str) -> Result<String, std::io::Error> {
    let mut re_list = vec![];
    re_list.push((r"@cron +(.*)".to_string(), 1));

    for (re, idx) in re_list {
        let re = regex::Regex::new(&re).unwrap();
        let file = std::fs::File::open(file)?;
        let reader = std::io::BufReader::new(file);
        for line in reader.lines() {
            let line = line?;
            if re.is_match(&line) {
                let cap = re.captures(&line).unwrap();
                return Ok(cap
                    .get(idx)
                    .unwrap()
                    .as_str()
                    .to_string()
                    .trim()
                    .to_string());
            }
        }
    }

    Err(std::io::Error::new(
        std::io::ErrorKind::Other,
        "no cron found in file",
    ))
}

fn find_cron_files(dir: &str, whitelist: &str) -> Result<Vec<(String, String)>, std::io::Error> {
    let files = find_files_by_regex(&dir, whitelist)?;
    let files: Vec<_> = files
        .iter()
        .map(|f| (f, find_cron_in_file(&format!("{}/{}", dir, f))))
        .filter(|(f, cron)| {
            if cron.is_ok() {
                println!("Info: found cron for file {}", f);
            }
            cron.is_ok()
        })
        .map(|(f, cron)| (f.clone(), cron.unwrap()))
        .collect();
    Ok(files)
}

fn get_repo_dir(repo: &str, work_dir: &str) -> String {
    format!("{}/repo/{}", work_dir, get_repo_name(repo))
}

fn list_fs_repos(work_dir: &str) -> Result<Vec<String>, std::io::Error> {
    let mut repos = Vec::new();
    let dir = format!("{}/repo", work_dir);
    for entry in std::fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            repos.push(path.file_name().unwrap().to_str().unwrap().to_string());
        }
    }
    Ok(repos)
}

pub fn add(
    tabs: &mut Vec<crontab::Item>,
    repo: &str,
    schedule: &str,
    whitelist: &str,
    work_dir: &str,
    branch: &str,
    force_clone: bool,
) -> Result<(), std::io::Error> {
    let repo_tabs = list(tabs);
    let f = repo_tabs
        .iter()
        .find(|i| i.args.as_ref().unwrap_left().name == repo);

    if f.is_some() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::Other,
            "repo already exists",
        ));
    }

    let repo_path = get_repo_dir(repo, work_dir);
    let is_git_repo = repo.starts_with("http://") || repo.starts_with("https://");

    if is_git_repo {
        clone_repo(repo, &repo_path, branch, force_clone)?;
    } else {
        if !std::path::Path::new(&repo_path).exists() {
            std::fs::create_dir_all(&repo_path)?;
        }
    }

    let files = find_cron_files(&repo_path, whitelist)?;

    let item = crontab::Item {
        schedule: schedule.to_string(),
        cmd: if is_git_repo {
            format!(
                "cd {} && git fetch origin {} && git reset --hard origin/{} && {} repo-readd",
                resolve_to_abspath(&repo_path).unwrap(),
                branch,
                branch,
                std::env::current_exe().unwrap().to_str().unwrap()
            )
        } else {
            ":".to_string()
        },
        args: Left(crontab::ItemArgs {
            group: GROUP_REPO.to_string(),
            name: repo.to_string(),
            repo_args: Some(crontab::RepoArgs {
                whitelist: whitelist.to_string(),
                branch: branch.to_string(),
            }),
        }),
    };
    tabs.push(item);

    if files.is_empty() {
        println!("Warning: no files added in repo")
    }
    for (f, cron) in files {
        let file_path = format!("{}/{}", resolve_to_abspath(&repo_path).unwrap(), f);
        let shebang = has_shebang(&file_path)?;

        let item = crontab::Item {
            schedule: cron,
            cmd: format!(
                ". {} && {} {}",
                get_env_path(work_dir),
                if shebang {
                    "".to_string()
                } else {
                    gen_launcher(&f)
                },
                file_path
            ),
            args: Left(crontab::ItemArgs {
                group: repo.to_string(),
                name: f.to_string(),
                repo_args: None,
            }),
        };
        tabs.push(item);
    }

    Ok(())
}

pub fn rm_by_repo(tabs: &Vec<crontab::Item>, repo: &str) -> Vec<crontab::Item> {
    let res = tabs
        .iter()
        .filter(|i| {
            i.args
                .as_ref()
                .map_left(|a| !((a.group == GROUP_REPO && a.name == repo) || &a.group == repo))
                .left_or(true)
        })
        .cloned()
        .collect();

    res
}

pub fn rm_by_index(
    tabs: &Vec<crontab::Item>,
    index: usize,
) -> Result<Vec<crontab::Item>, std::io::Error> {
    let repo_tabs = list(tabs);
    let t = repo_tabs.get(index);
    if t.is_none() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::Other,
            "repo not found",
        ));
    }
    let t = t.unwrap();

    let repo = &t.args.as_ref().unwrap_left().name;
    Ok(rm_by_repo(tabs, repo))
}

fn rm_repo_files(work_dir: &str, repo: &str) {
    let repo_name = get_repo_name(repo);
    if std::fs::remove_dir_all(get_repo_dir(repo, work_dir)).is_err() {
        println!("Warning: failed to remove repo: {}", &repo_name)
    }
}

pub fn clean_files(tabs: &Vec<crontab::Item>, work_dir: &str) -> Result<(), std::io::Error> {
    let tab_repos = list(tabs);
    let fs_repos = list_fs_repos(work_dir)?;

    for r in fs_repos {
        if tab_repos
            .iter()
            .find(|i| i.args.as_ref().unwrap_left().name == r)
            .is_none()
        {
            rm_repo_files(work_dir, &r);
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    // Note this useful idiom: importing names from outer (for mod tests) scope.
    use super::*;

    #[test]
    fn test() {}
}
