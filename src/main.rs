mod core;
mod utils;

use self::core::export::exporter;
use self::core::filter::filter;
use self::core::swrite::swrite;
use self::core::concat::concater;

use clap::{Args, Parser, Subcommand};
use std::path::PathBuf;

///
/// Example command and output
/// Command: ./bin/analyze_polars filter -f ./file/raw/v3.18.44-en-sample.csv -s file/csv
/// Response: {
///     "FltrFile":{
///         "cyDb":"db.csv",
///         "cyGt":"gait.csv",
///         "cyLt":"ls.csv",
///         "cyRt":"rs.csv",
///         "rslt":"v3.18.44-en-sample.csv"
///     },
///     "Range":[{"End":15.965,"Start":4.37},{"End":35.755,"Start":25.375}]
/// }
///
/// Command:  ./bin/analyze_polars export -f file/csv/v3.18.44-en-sample.csv -s file/export -r 1 12
/// Response: {"ExportFile":"v3.18.44-en-sample-result.csv"}
///
/// Command:  ./bin/analyze_polars swrite -f file/raw/v3.18.44-en-sample.csv -s file/cleaning -v 4.37-15.965
/// Response: {"CleanFile":"v3.18.44-en-sample.csv"}
///
/// Command:  ./bin/analyze_polars concat -f file/export/v3.18.44-en-sample-result.csv -f file/export/v3.18.44-en-sample-result.csv -s file/export
/// Response: {"ConcatFile":"concat.csv"}
///

#[derive(Debug, Parser)]
#[clap(name = "analyze-rs")]
#[clap(about = "analyze human GAIT cycle", long_about = None)]
struct Cli {
    #[clap(subcommand)]
    command: Commands,
}

#[derive(Debug, Subcommand)]
enum Commands {
    #[clap(arg_required_else_help = true)]
    Filter(Filter),
    #[clap(arg_required_else_help = true)]
    Export(Export),
    #[clap(arg_required_else_help = true)]
    Swrite(Swrite),
    #[clap(arg_required_else_help = true)]
    Concat(Concat),
}

#[derive(Debug, Args)]
struct Filter {
    #[clap(short, long, required = true)]
    file: PathBuf,
    #[clap(short, long, required = true)]
    save: PathBuf,
}

#[derive(Debug, Args)]
struct Export {
    #[clap(short, long, required = true)]
    file: PathBuf,
    #[clap(short, long, required = true)]
    save: PathBuf,
    #[clap(short, long, parse(try_from_str = parse_range_tuple), required = true)]
    ranges: Vec<(u32, u32)>,
}

#[derive(Debug, Args)]
struct Swrite {
    #[clap(short, long, required = true)]
    file: PathBuf,
    #[clap(short, long, required = true)]
    save: PathBuf,
    #[clap(short, long, required = true)]
    value: String,
}

#[derive(Debug, Args)]
struct Concat {
    #[clap(short, long, parse(from_os_str), required = true)]
    files: Vec<PathBuf>,
    #[clap(short, long, required = true)]
    save: PathBuf,
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
    let vec = s.split(' ').take(2).collect::<Vec<&str>>();
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
        Commands::Concat(args) => {
            if let Err(e) = concater(args.files, args.save) {
                println!("{}", e)
            };
        }
    }
}
