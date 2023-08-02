mod crontab;
mod env;
mod repo;

use clap::{Args, Parser, Subcommand};
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
    repo: String,

    /// Regex for whitelist
    #[serde(default = "default_whitelist")]
    whitelist: String,

    schedule: String,

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

#[derive(Debug, Args, Deserialize)]
struct ListTasksArg {
    name: String,
}

#[derive(Subcommand, Debug)]
enum Commands {
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
        std::fs::create_dir_all(&cli.work_dir).unwrap();
    }

    match cli.command {
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
                    let tabs = crontab::get().unwrap();
                    let repos = repo::list(&tabs);
                    response(&serde_json::to_string(&repos).unwrap())
                },
                (POST) (/api/repo/listTasks) => {
                    let arg: ListTasksArg = rouille::try_or_400!(rouille::input::json_input(request));
                    let tabs = crontab::get().unwrap();
                    let tasks = repo::list_tasks(&tabs, &arg.name);
                    response(&serde_json::to_string(&tasks).unwrap())
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
                    response(&serde_json::to_string(&map).unwrap())
                },
                _ => rouille::Response::empty_404()
            )
        }),
    }
}

fn response(json: &str) -> rouille::Response {
    rouille::Response::from_data(
        "application/json; charset=utf-8",
        format!("{{\"code\": 200, \"data\": {}}}", json),
    )
}
