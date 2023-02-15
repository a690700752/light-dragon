mod crontab;
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Add repo
    RepoAdd {
        repo: String,
        #[clap(short, long, value_delimiter(','))]
        whitelist: Vec<String>,
        #[clap(short, long, value_delimiter(','))]
        blacklist: Vec<String>,
    },
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::RepoAdd { .. } => {
            println!("Adding repo: {:?}", cli.command);
            println!(
                "Current crontab: {:?}",
                crontab::set(crontab::get().unwrap())
            );
        }
    }
}
