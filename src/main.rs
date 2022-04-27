mod core;
mod utils;

use self::core::filter::filter;
use self::core::swrite::swrite;
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
    #[clap(arg_required_else_help = true)]
    Swrite(Swrite),
}

#[derive(Debug, Args)]
struct Filter {
    #[clap(short, long, required = true)]
    file: String,
    #[clap(short, long, required = true)]
    save: String,
}

#[derive(Debug, Args)]
struct Swrite {
    #[clap(short, long, required = true)]
    file: String,
    #[clap(short, long, required = true)]
    save: String,
    #[clap(short, long, required = true)]
    value: String,
}

fn main() {
    let args = Cli::parse();
    match args.command {
        Commands::Filter(args) => {
            if let Err(e) = filter(args.file, args.save) {
                println!("{}", e)
            };
        }
        Commands::Swrite(args) => {
            if let Err(e) = swrite(args.file, args.save, args.value) {
                println!("{}", e)
            };
        }
    }
}
