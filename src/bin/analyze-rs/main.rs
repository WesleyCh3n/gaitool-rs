mod args;
use std::fs;

use args::*;

use analyze::core::check::*;
use analyze::core::clean::*;
use analyze::core::concat::*;
use analyze::core::export::*;
use analyze::core::filter::*;
use analyze::core::split::*;
use analyze::core::swrite::*;
use analyze::core::diff::diff_column;

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
            if let Ok(resp) =
                filter(args.file, args.save, args.remap_csv, args.web_csv)
            {
                println!("{}", resp)
            };
        }
        Commands::Swrite(args) => {
            if let Ok(resp) =
                swrite(args.file, args.save, args.value, args.remap_csv)
            {
                println!("{}", resp)
            };
        }
        Commands::Export(args) => {
            if let Ok(resp) = exporter(args.file, args.save, args.ranges) {
                println!("{}", resp)
            };
        }
        Commands::Concat(args) => {
            if let Err(e) = concater(args.file, args.save) {
                println!("{}", e)
            };
        }
        Commands::Split(args) => {
            let paths = fs::read_dir(&args.file_dir).unwrap_or_else(|e| {
                panic!("Failed to read {:?}. {}", args.file_dir, e)
            });
            for file in paths {
                let file = file.unwrap_or_else(|e| panic!("{}", e)).path();
                match split(
                    &file,
                    &args.save,
                    args.percent,
                    &args.remap_csv,
                    None,
                ) {
                    Ok(()) => {
                        println!("{}: Success", file.display());
                    }
                    Err(e) => {
                        println!("{}: {}", file.display(), e)
                    }
                };
            }
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
        Commands::Diff(args) => {
            if let Err(e) = diff_column(&args.file, &args.remap_csv) {
                println!("{}", e);
            }
        }
    }
}
