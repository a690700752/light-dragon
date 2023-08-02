mod crontab;
mod env;
mod repo;

use clap::{Args, Parser, Subcommand};
use crontab::Item;
use rouille::router;
use serde::Deserialize;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    #[arg(short, long, default_value = "~/.local/share/light-dragon")]
    work_dir: String,
}

#[derive(Debug, Args, Deserialize)]
struct RepoAddArg {
    #[arg(value_parser = |v: &str| {
            if v.starts_with("https://") || v.starts_with("git@") {
                Ok(v.to_string())
            } else {
                Err("repo must start with https:// or git@".to_string())
            }
        })]
    repo: String,

    /// Regex for whitelist
    #[arg(short, long, default_value = r".*\.ts$")]
    #[serde(default = "default_whitelist")]
    whitelist: String,

    #[arg(short, long)]
    schedule: String,

    #[arg(short, long, default_value = "master")]
    #[serde(default = "default_branch")]
    branch: String,
}

fn default_whitelist() -> String {
    r".*\.ts$".to_string()
}

fn default_branch() -> String {
    "master".to_string()
}

#[derive(Debug, Args, Deserialize)]
struct RepoRmArg {
    index: usize,
}

#[derive(Debug, Args, Deserialize)]
struct EnvAddArg {
    name: String,
    value: String,
}

#[derive(Debug, Args, Deserialize)]
struct EnvRmArg {
    name: String,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Add repo
    RepoAdd(RepoAddArg),
    /// Remove repo in crontab
    RepoRm(RepoRmArg),
    /// Clean unused repo files
    RepoClean,
    /// List repo in crontab
    RepoList,
    /// Sync cron tasks by repo
    TaskSync,
    /// Add environment variable
    EnvAdd(EnvAddArg),
    /// Remove environment variable
    EnvRm(EnvRmArg),
    /// List environment variable
    EnvList,
    Rpc {},
}

fn cmd_repo_add(work_dir: &str, repo: &str, whitelist: &str, schedule: &str, branch: &str) {
    let mut tabs = crontab::get().unwrap();
    repo::add(
        &mut tabs, repo, schedule, whitelist, work_dir, branch, false,
    )
    .unwrap();
    crontab::set(tabs).unwrap();
}

fn cmd_repo_list() -> Vec<Item> {
    let tabs = crontab::get().unwrap();
    repo::list(&tabs).into_iter().cloned().collect()
}

fn cmd_repo_rm(index: usize) {
    let tabs = crontab::get().unwrap();
    let tabs = repo::rm_by_index(&tabs, index).unwrap();
    crontab::set(tabs).unwrap();
}

fn cmd_repo_clean(work_dir: &str) {
    let tabs = crontab::get().unwrap();
    repo::clean_files(&tabs, work_dir).unwrap();
}

fn cmd_repo_readd(work_dir: &str) {
    let tabs = crontab::get().unwrap();
    let repos = repo::list(&tabs);

    let mut tabs = tabs.clone();
    for repo in repos {
        let args = repo.args.as_ref().unwrap_left();
        let repo_args = args.repo_args.as_ref().unwrap();
        tabs = repo::rm_by_repo(&tabs, &args.name);

        repo::add(
            &mut tabs,
            &args.name,
            &repo.schedule,
            &repo_args.whitelist,
            work_dir,
            &repo_args.branch,
            false,
        )
        .unwrap();
    }
    crontab::set(tabs).unwrap();
}

fn main() {
    let mut cli = Cli::parse();
    cli.work_dir = shellexpand::tilde(&cli.work_dir).to_string();

    if !std::path::Path::new(&cli.work_dir).exists() {
        std::fs::create_dir(&cli.work_dir).unwrap();
    }

    match cli.command {
        Commands::RepoAdd(arg) => cmd_repo_add(
            &cli.work_dir,
            &arg.repo,
            &arg.whitelist,
            &arg.schedule,
            &arg.branch,
        ),
        Commands::RepoRm(arg) => cmd_repo_rm(arg.index),
        Commands::RepoClean => cmd_repo_clean(&cli.work_dir),
        Commands::RepoList => {
            let repos = cmd_repo_list();

            for (i, repo) in repos.iter().enumerate() {
                println!(
                    "{}\t{}\t{}\t{}",
                    i,
                    repo.schedule,
                    repo.args.as_ref().unwrap_left().name,
                    repo.args
                        .as_ref()
                        .unwrap_left()
                        .repo_args
                        .as_ref()
                        .unwrap()
                        .whitelist,
                )
            }
        }
        Commands::TaskSync => cmd_repo_readd(&cli.work_dir),
        Commands::EnvAdd(arg) => env::add(&cli.work_dir, &arg.name, &arg.value).unwrap(),
        Commands::EnvRm(arg) => env::rm(&cli.work_dir, &arg.name).unwrap(),
        Commands::EnvList => {
            let map = env::list(&cli.work_dir).unwrap();
            for (name, value) in map {
                println!("{}={}", name, value);
            }
        }
        Commands::Rpc {} => rouille::start_server("localhost:8000", move |request| {
            router!(request,
                (GET) (/) => {
                    rouille::Response::text("hello world")
                },
                (POST) (/api/repo/add) => {
                    let arg: RepoAddArg = rouille::try_or_400!(rouille::input::json_input(request));
                    cmd_repo_add(&cli.work_dir, &arg.repo, &arg.whitelist, &arg.schedule, &arg.branch);
                    response("null")
                },
                (POST) (/api/repo/list) => {
                    let repos = cmd_repo_list();
                    rouille::Response::json(&repos)
                },
                (POST) (/api/repo/rm) => {
                    let arg: RepoRmArg = rouille::try_or_400!(rouille::input::json_input(request));
                    cmd_repo_rm(arg.index);
                    response("null")
                },
                (POST) (/api/repo/clean) => {
                    cmd_repo_clean(&cli.work_dir);
                    response("null")
                },
                (POST) (/api/repo/readd) => {
                    cmd_repo_readd(&cli.work_dir);
                    response("null")
                },
                (POST) (/api/env/add) => {
                    let arg: EnvAddArg = rouille::try_or_400!(rouille::input::json_input(request));
                    env::add(&cli.work_dir, &arg.name, &arg.value).unwrap();
                    response("null")
                },
                (POST) (/api/env/rm) => {
                    let arg: EnvRmArg = rouille::try_or_400!(rouille::input::json_input(request));
                    env::rm(&cli.work_dir, &arg.name).unwrap();
                    response("null")
                },
                (POST) (/api/env/list) => {
                    let map = env::list(&cli.work_dir).unwrap();
                    rouille::Response::json(&map)
                },
                _ => rouille::Response::empty_404()
            )
        }),
    }
}

fn response(json: &str) -> rouille::Response {
    rouille::Response::from_data(
        "application/json",
        format!("{{\"code\": 200, \"data\": {}}}", json),
    )
}
