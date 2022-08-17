mod args;
use std::fs;

use args::*;

use gaitool_rs::core::check::*;
use gaitool_rs::core::clean::*;
use gaitool_rs::core::concat::*;
use gaitool_rs::core::diff::diff_column;
use gaitool_rs::core::export::*;
use gaitool_rs::core::split::*;

use clap::Parser;

///
/// Example command and output
/// Command:  analyze-rs export -f file/csv/v3.18.44-en-sample.csv -s file/export -r "1 12" -r "15 22"
/// Response: {"ExportFile":"v3.18.44-en-sample-result.csv"}
///
/// Command:  analyze-rs concat -f file/export/v3.18.44-en-sample-result.csv -f file/export/v3.18.44-en-sample-result.csv -s file/export
/// Response: {"ConcatFile":"concat.csv"}
///

fn main() {
    let args = Cli::parse();
    match args.command {
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
                    &args.remap_csv_dir,
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
