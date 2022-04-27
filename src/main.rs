mod core;
mod utils;

use self::core::filter::filter;
use clap::{Args, Parser, Subcommand};

#[derive(Debug, Parser)]
#[clap(name = "analyse")]
#[clap(about = "analyse gait", long_about = None)]
struct Cli {
    #[clap(subcommand)]
    command: Commands,
}

#[derive(Debug, Subcommand)]
enum Commands {
    #[clap(arg_required_else_help = true)]
    Filter(Filter),
}

#[derive(Debug, Args)]
struct Filter {
    #[clap(short, long, required = true)]
    file: String,
    #[clap(short, long, required = true)]
    save: String,
}

fn main() {
    let args = Cli::parse();
    match args.command {
        Commands::Filter(f) => {
            println!("file: {}, save: {}", f.file, f.save);
            filter(f.file, f.save).unwrap();
        }
    }
}
