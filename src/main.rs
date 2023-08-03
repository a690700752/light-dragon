mod crontab;
mod env;
mod repo;

use clap::{Parser, Subcommand};
use rouille::{router, Request, Response};
use serde::{Deserialize, Serialize};
use std::fs;
#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    #[arg(short, long, default_value = "~/.local/share/light-dragon")]
    work_dir: String,
}

fn default_whitelist() -> String {
    r".*\.ts$".to_string()
}

fn default_branch() -> String {
    "master".to_string()
}

#[derive(Subcommand, Debug)]
enum Commands {
    Rpc {},
}

fn cmd_repo_add(
    work_dir: &str,
    repo: &str,
    whitelist: &str,
    schedule: &str,
    branch: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut tabs = crontab::get()?;
    repo::add(
        &mut tabs, repo, schedule, whitelist, work_dir, branch, false,
    )
    .unwrap();
    crontab::set(tabs)?;
    Ok(())
}

fn cmd_repo_rm(index: usize) -> Result<(), Box<dyn std::error::Error>> {
    let tabs = crontab::get()?;
    let tabs = repo::rm_by_index(&tabs, index)?;
    crontab::set(tabs)?;
    Ok(())
}

fn cmd_repo_clean(work_dir: &str) -> Result<(), Box<dyn std::error::Error>> {
    let tabs = crontab::get()?;
    repo::clean_files(&tabs, work_dir)?;
    Ok(())
}

fn cmd_repo_readd(work_dir: &str) -> Result<(), Box<dyn std::error::Error>> {
    let tabs = crontab::get()?;
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
        )?;
    }
    crontab::set(tabs)?;
    Ok(())
}

#[derive(Deserialize)]
struct PathBody {
    path: String,
}

fn main() {
    let mut cli = Cli::parse();
    cli.work_dir = shellexpand::tilde(&cli.work_dir).to_string();

    if !std::path::Path::new(&cli.work_dir).exists() {
        fs::create_dir_all(&cli.work_dir).unwrap();
    }

    match cli.command {
        Commands::Rpc {} => rouille::start_server("localhost:8000", move |request| {
            match handler(request, &cli.work_dir) {
                Ok(resp) => resp,
                Err(err) => {
                    eprintln!("error: {}", err);
                    Response::json(&serde_json::json!({
                        "code": 1000,
                        "message": format!("{}", err),
                    }))
                }
            }
        }),
    }
}

fn handler(request: &Request, work_dir: &str) -> Result<Response, Box<dyn std::error::Error>> {
    router!(request,
        (GET) (/) => {
            Ok(Response::text("hello world"))
        },
        (POST) (/api/repo/add) => {
            #[derive(Debug, Deserialize)]
            struct RepoAddArg {
                repo: String,

                #[serde(default = "default_whitelist")]
                whitelist: String,

                schedule: String,

                #[serde(default = "default_branch")]
                branch: String,
            }


            let arg: RepoAddArg = rouille::input::json_input(request)?;
            let _ = cmd_repo_add(work_dir, &arg.repo, &arg.whitelist, &arg.schedule, &arg.branch);
            Ok(resp("null"))
        },
        (POST) (/api/repo/list) => {
            let tabs = crontab::get()?;
            let repos = repo::list(&tabs);
            Ok(resp(&serde_json::to_string(&repos)?))
        },
        (POST) (/api/repo/listTasks) => {
            #[derive(Debug, Deserialize)]
            struct ListTasksArg {
                name: String,
            }

            let arg: ListTasksArg = rouille::input::json_input(request)?;
            let tabs = crontab::get()?;
            let tasks = repo::list_tasks(&tabs, &arg.name);
            Ok(resp(&serde_json::to_string(&tasks)?))
        },
        (POST) (/api/repo/rm) => {
            #[derive(Debug, Deserialize)]
            struct RepoRmArg {
                index: usize,
            }

            let arg: RepoRmArg = rouille::input::json_input(request)?;
            let _ = cmd_repo_rm(arg.index);
            Ok(resp("null"))
        },
        (POST) (/api/repo/clean) => {
            let _ = cmd_repo_clean(work_dir);
            Ok(resp("null"))
        },
        (POST) (/api/repo/readd) => {
            let _ = cmd_repo_readd(work_dir);
            Ok(resp("null"))
        },
        (POST) (/api/env/add) => {
            #[derive(Debug, Deserialize)]
            struct EnvAddArg {
                name: String,
                value: String,
            }

            let arg: EnvAddArg = rouille::input::json_input(request)?;
            env::add(work_dir, &arg.name, &arg.value)?;
            Ok(resp("null"))
        },
        (POST) (/api/env/rm) => {
            #[derive(Debug, Deserialize)]
            struct EnvRmArg {
                name: String,
            }

            let arg: EnvRmArg = rouille::input::json_input(request)?;
            env::rm(work_dir, &arg.name)?;
            Ok(resp("null"))
        },
        (POST) (/api/env/list) => {
            let map = env::list(work_dir)?;
            Ok(resp(&serde_json::to_string(&map)?))
        },
        (POST) (/api/fs/ls) => {
            #[derive(Serialize)]
            struct LsItem {
                name: String,
                is_dir: bool,
            }

            let arg: PathBody = rouille::input::json_input(request)?;
            if arg.path.contains("..") {
                return Err(std::io::Error::new(std::io::ErrorKind::Other, "invalid path").into());
            }

            let list = fs::read_dir(format!("{}/repo/{}",work_dir ,arg.path))?
                .map(|entry| {
                    let entry = entry.unwrap();
                    let path = entry.path();
                    let name = path.file_name().unwrap().to_str().unwrap().to_string();
                    let is_dir = path.is_dir();
                    LsItem { name, is_dir }
                })
                .collect::<Vec<_>>();
            Ok(resp(&serde_json::to_string(&list)?))
        },
        (POST) (/api/fs/read) => {
            let arg: PathBody = rouille::input::json_input(request)?;
            if arg.path.contains("..") {
                return Err(std::io::Error::new(std::io::ErrorKind::Other, "invalid path").into());
            }

            let content = fs::read_to_string(format!("{}/repo/{}", work_dir,arg.path))?;
            Ok(resp(&serde_json::to_string(&content)?))
        },
        (POST) (/api/fs/write) => {
            #[derive(Deserialize)]
            struct WriteArg {
                path: String,
                content: String,
            }
            let arg: WriteArg = rouille::input::json_input(request)?;

            if arg.path.contains("..") {
                return Err(std::io::Error::new(std::io::ErrorKind::Other, "invalid path").into());
            }

            fs::write(dbg!(format!("{}/repo/{}", work_dir, arg.path)), &arg.content)?;
            Ok(resp("null"))
        },
        _ => Ok(rouille::Response::empty_404())
    )
}

fn resp(json: &str) -> rouille::Response {
    rouille::Response::from_data(
        "application/json; charset=utf-8",
        format!("{{\"code\": 200, \"data\": {}}}", json),
    )
}
