mod core;
mod utils;

use self::core::concat::concater;
use self::core::export::exporter;
use self::core::filter::filter;
use self::core::swrite::swrite;
use self::core::split::split;

use clap::{Args, Parser, Subcommand};
use std::path::PathBuf;

///
/// Example command and output
/// Command: analyze-rs filter -f ./file/raw/v3.18.44-en-sample.csv -s file/csv
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
/// Command:  analyze-rs export -f file/csv/v3.18.44-en-sample.csv -s file/export -r "1 12" -r "15 22"
/// Response: {"ExportFile":"v3.18.44-en-sample-result.csv"}
///
/// Command:  analyze-rs swrite -f file/raw/v3.18.44-en-sample.csv -s file/cleaning -v 4.37-15.965
/// Response: {"CleanFile":"v3.18.44-en-sample.csv"}
///
/// Command:  analyze-rs concat -f file/export/v3.18.44-en-sample-result.csv -f file/export/v3.18.44-en-sample-result.csv -s file/export
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
    /// filter raw file
    #[clap(arg_required_else_help = true)]
    Filter(Filter),
    /// export calculation in selection range
    #[clap(arg_required_else_help = true)]
    Export(Export),
    /// write selection range into raw file
    #[clap(arg_required_else_help = true)]
    Swrite(Swrite),
    /// concat export files
    #[clap(arg_required_else_help = true)]
    Concat(Concat),
    #[clap(arg_required_else_help = true)]
    Split(Split),
}

#[derive(Debug, Args)]
struct Filter {
    /// input file
    #[clap(short, long, required = true)]
    file: PathBuf,
    /// output directory
    #[clap(short, long, required = true)]
    save: PathBuf,
}

#[derive(Debug, Args)]
struct Export {
    /// input file
    #[clap(short, long, required = true)]
    file: PathBuf,
    /// output directory
    #[clap(short, long, required = true)]
    save: PathBuf,
    /// selection range index. e.g. "2 5"
    #[clap(short, long, parse(try_from_str = parse_range_tuple), required = true)]
    ranges: Vec<(u32, u32)>,
}

#[derive(Debug, Args)]
struct Swrite {
    /// input file
    #[clap(short, long, required = true)]
    file: PathBuf,
    /// output directory
    #[clap(short, long, required = true)]
    save: PathBuf,
    /// selection values to write
    #[clap(short, long, required = true)]
    value: String,
}

#[derive(Debug, Args)]
struct Concat {
    /// inputs file (can be multiple, e.g. "-f file1 -f file2")
    #[clap(short, long, parse(from_os_str), required = true)]
    file: Vec<PathBuf>,
    /// output directory
    #[clap(short, long, required = true)]
    save: PathBuf,
}

#[derive(Debug, Args)]
struct Split {
    #[clap(short, long, required = true)]
    file_dir: PathBuf,
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
            if let Err(e) = concater(args.file, args.save) {
                println!("{}", e)
            };
        }
        Commands::Split(args) => {
            if let Err(e) = split(args.file_dir, args.save) {
                println!("{}", e)
            };
        }
    }
}
