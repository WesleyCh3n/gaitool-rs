mod core;
mod utils;
mod args;

use crate::args::*;
use crate::core::concat::concater;
use crate::core::export::exporter;
use crate::core::filter::filter;
use crate::core::swrite::swrite;
use crate::core::split::split;
use crate::core::check::check;
use crate::core::clean::clean;

use clap::Parser;

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
            if let Err(e) = split(args.file_dir, args.save, args.percent) {
                println!("{}", e)
            };
        }
        Commands::Check(args) => {
            if let Err(e) = check(args.file_dir) {
                println!("{}", e)
            };
        }
        Commands::Clean(args) => {
            if let Err(e) = clean(args.file_dir, args.save) {
                println!("{}", e)
            };
        }
    }
}
