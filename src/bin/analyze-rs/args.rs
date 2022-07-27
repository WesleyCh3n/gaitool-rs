use clap::{Args, Parser, Subcommand};
use std::path::PathBuf;

#[derive(Debug, Parser)]
#[clap(name = "analyze-rs")]
#[clap(about = "analyze human GAIT cycle", long_about = None)]
pub struct Cli {
    #[clap(subcommand)]
    pub command: Commands,
}

#[derive(Debug, Subcommand)]
pub enum Commands {
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
    /// auto select valid selection
    #[clap(arg_required_else_help = true)]
    Split(Split),
    /// batch check if header and file num correct
    #[clap(arg_required_else_help = true)]
    Check(Check),
    /// batch de-identify header info
    #[clap(arg_required_else_help = true)]
    Clean(Clean),
    ///
    #[clap(arg_required_else_help = true)]
    Diff(Diff),
}

#[derive(Debug, Args)]
pub struct Filter {
    /// input file
    #[clap(short, long, required = true)]
    pub file: PathBuf,
    /// output directory
    #[clap(short, long, required = true)]
    pub save: PathBuf,
    #[clap(short, long, default_value = "./assets/all.csv")]
    pub remap_csv: PathBuf,
    #[clap(short, long, default_value = "./assets/filter.csv")]
    pub web_csv: PathBuf,
}

#[derive(Debug, Args)]
pub struct Export {
    /// input file
    #[clap(short, long, required = true)]
    pub file: PathBuf,
    /// output directory
    #[clap(short, long, required = true)]
    pub save: PathBuf,
    /// selection range index. e.g. "2 5"
    #[clap(short, long, parse(try_from_str = parse_range_tuple), required = true)]
    pub ranges: Vec<(u32, u32)>,
}

#[derive(Debug, Args)]
pub struct Swrite {
    /// input file
    #[clap(short, long, required = true)]
    pub file: PathBuf,
    /// output directory
    #[clap(short, long, required = true)]
    pub save: PathBuf,
    /// selection values to write
    #[clap(short, long, required = true)]
    pub value: String,
    #[clap(short, long, default_value = "./assets/all.csv")]
    pub remap_csv: PathBuf,
}

#[derive(Debug, Args)]
pub struct Concat {
    /// inputs file (can be multiple, e.g. "-f file1 -f file2")
    #[clap(short, long, parse(from_os_str), required = true)]
    pub file: Vec<PathBuf>,
    /// output directory
    #[clap(short, long, required = true)]
    pub save: PathBuf,
}

#[derive(Debug, Args)]
pub struct Split {
    /// input directory
    #[clap(short, long, required = true)]
    pub file_dir: PathBuf,
    /// output directory
    #[clap(short, long, required = true)]
    pub save: PathBuf,
    /// valid percentage
    #[clap(short, long, required = true)]
    pub percent: usize,
    #[clap(short, long, default_value = "./assets/all.csv")]
    pub remap_csv: PathBuf,
}

#[derive(Debug, Args)]
pub struct Check {
    /// input directory
    #[clap(short, long, required = true)]
    pub file_dir: PathBuf,
}

#[derive(Debug, Args)]
pub struct Clean {
    /// input directory
    #[clap(short, long, required = true)]
    pub file_dir: PathBuf,
    #[clap(short, long, required = true)]
    pub save: PathBuf,
}

#[derive(Debug, Args)]
pub struct Diff {
    /// input directory
    #[clap(short, long, required = true)]
    pub file: PathBuf,
    #[clap(short, long, default_value = "./assets/all.csv")]
    pub remap_csv: PathBuf,
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
