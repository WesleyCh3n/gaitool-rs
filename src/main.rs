mod core;
mod utils;

use self::core::filter::filter;
use self::core::swrite::swrite;
use self::core::exporter::exporter;
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
    #[clap(arg_required_else_help = true)]
    Export(Export),
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

#[derive(Debug, Args)]
struct Export {
    #[clap(short, long, required = true)]
    file: String,
    #[clap(short, long, required = true)]
    save: String,
    #[clap(short, long, parse(try_from_str = parse_range_tuple), required = true)]
    ranges: Vec<(u32, u32)>,
}

fn parse_range_tuple<T, U>(
    s: &str,
) -> Result<(T, U), Box<dyn std::error::Error + Send + Sync + 'static>>
where
    T: std::str::FromStr,
    T::Err: std::error::Error + Send + Sync + 'static,
    U: std::str::FromStr,
    U::Err: std::error::Error + Send + Sync + 'static,
{
    let vec = s
        .split(' ')
        .take(2)
        .collect::<Vec<&str>>();
    Ok((vec[0].parse()?, vec[1].parse()?))
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
        Commands::Export(args) => {
            if let Err(e) = exporter(args.file, args.save, args.ranges) {
                println!("{}", e)
            };
        }
    }
}
