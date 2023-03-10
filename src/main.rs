mod crontab;
mod dep_js;
mod dep_py;
mod env;
mod repo;

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    #[arg(short, long, default_value = "~/.local/share/light-dragon")]
    work_dir: String,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Add repo
    RepoAdd {
        #[arg(value_parser = |v: &str| {
            if v.starts_with("https://") || v.starts_with("git@") {
                Ok(v.to_string())
            } else {
                Err("repo must start with https:// or git@".to_string())
            }
        })]
        repo: String,

        /// Regex for whitelist
        #[arg(short, long, default_value = r".*\.(sh|py|js)$")]
        whitelist: String,

        #[arg(short, long)]
        schedule: String,

        #[arg(short, long, default_value = "master")]
        branch: String,

        #[arg(short, long, default_value = "false")]
        violence: bool,
    },
    /// Remove repo in crontab
    RepoRm {
        index: usize,
    },
    /// Clean unused repo files
    RepoClean,
    /// List repo in crontab
    RepoList,
    /// Sync repo's cron files
    RepoReadd,
    /// Add environment variable
    EnvAdd {
        name: String,
        value: String,
    },
    /// Remove environment variable
    EnvRm {
        name: String,
    },
    /// List environment variable
    EnvList,
    DepJsList,
    DepJsAdd {
        name: String,
        version: Option<String>,
    },
    DepJsRm {
        name: String,
    },
    DepPyList,
    DepPyAdd {
        name: String,
        version: Option<String>,
    },
    DepPyRm {
        name: String,
    },
}

fn main() {
    let mut cli = Cli::parse();
    cli.work_dir = shellexpand::tilde(&cli.work_dir).to_string();

    if !std::path::Path::new(&cli.work_dir).exists() {
        std::fs::create_dir(&cli.work_dir).unwrap();
    }

    match cli.command {
        Commands::RepoAdd {
            repo,
            whitelist,
            schedule,
            branch,
            violence,
        } => {
            let mut tabs = crontab::get().unwrap();
            repo::add(
                &mut tabs,
                &repo,
                &schedule,
                &whitelist,
                &cli.work_dir,
                &branch,
                false,
                violence,
            )
            .unwrap();
            crontab::set(tabs).unwrap();
        }
        Commands::RepoRm { index } => {
            let tabs = crontab::get().unwrap();
            let tabs = repo::rm_by_index(&tabs, index).unwrap();
            crontab::set(tabs).unwrap();
        }
        Commands::RepoClean => {
            let tabs = crontab::get().unwrap();
            repo::clean_files(&tabs, &cli.work_dir).unwrap();
        }
        Commands::RepoList => {
            let tabs = crontab::get().unwrap();
            let repos = repo::list(&tabs);

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
        Commands::RepoReadd => {
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
                    &cli.work_dir,
                    &repo_args.branch,
                    false,
                    repo_args.violence,
                )
                .unwrap();
            }
            crontab::set(tabs).unwrap();
        }
        Commands::EnvAdd { name, value } => env::add(&cli.work_dir, &name, &value).unwrap(),
        Commands::EnvRm { name } => env::rm(&cli.work_dir, &name).unwrap(),
        Commands::EnvList => {
            let map = env::list(&cli.work_dir).unwrap();
            for (name, value) in map {
                println!("{}={}", name, value);
            }
        }
        Commands::DepJsList => {
            let m = dep_js::list(&cli.work_dir);
            for (name, version) in m {
                println!("{}={}", name, version);
            }
        }
        Commands::DepJsAdd { name, version } => {
            dep_js::add(&cli.work_dir, &name, version.as_deref()).unwrap();
        }
        Commands::DepJsRm { name } => {
            dep_js::rm(&cli.work_dir, &name).unwrap();
        }
        Commands::DepPyList => {
            let m = dep_py::list();
            for (name, version) in m {
                println!("{}={}", name, version);
            }
        }
        Commands::DepPyAdd { name, version } => {
            dep_py::add(&name, version.as_deref()).unwrap();
        }
        Commands::DepPyRm { name } => {
            dep_py::rm(&name).unwrap();
        }
    }
}
