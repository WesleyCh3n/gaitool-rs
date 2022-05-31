use crate::utils::util::*;

use indicatif::{ProgressBar, ProgressStyle};
use polars::prelude::*;
use std::fs::{self, create_dir_all};
use std::path::{Path, PathBuf};

pub fn clean(file_dir: PathBuf, save_dir: PathBuf) -> Result<()> {
    create_dir_all(&save_dir)?;
    let paths = fs::read_dir(&file_dir)?;
    let file_num = fs::read_dir(&file_dir)?.count();
    let spinner_style = ProgressStyle::default_spinner()
        .template("{prefix:.bold.dim} {spinner} {wide_msg}")
        .tick_chars("⠁⠂⠄⡀⢀⠠⠐⠈ ");

    let pb = ProgressBar::new(file_num as u64);
    for file in paths {
        let file = file?;
        let file = file.path();
        let file = file.display().to_string();
        /* read file */
        let filename = get_file_name(Path::new(&file));
        let file_stem = get_file_stem(Path::new(&file));
        let saved_path = join_path(Path::new(&save_dir), Path::new(&filename));
        pb.set_style(spinner_style.clone());
        pb.set_message(format!("Processing {}", filename));

        let parts = file_stem.split("_").collect::<Vec<&str>>();
        if parts.len() != 2 {
            pb.set_message(format!(
                "Can't parse file name: {}. Skipped!",
                filename
            ));
            continue;
        }

        let mut name_vec = parts[0].split("-").collect::<Vec<&str>>();
        name_vec.append(&mut parts[1].split("-").collect::<Vec<&str>>());

        if name_vec.len() < 11 {
            pb.set_message(format!(
                "Can't parse file name: {}. Skipped!",
                filename
            ));
            continue;
        }
        pb.inc(1);

        /* read/write only header */
        extract_header(&file, &saved_path);
        /* header to dataframe */
        let mut header_df = CsvReader::from_path(&saved_path)?.finish()?;
        header_df = header_df
            .lazy()
            .drop_columns(["last_name", "first_name"])
            .collect()?;
        save_csv(&mut header_df, &save_dir.display().to_string(), &filename);
        let mut df = CsvReader::from_path(&file)?.with_skip_rows(3).finish()?;
        append_df2header(&mut df, &save_dir.display().to_string(), &filename);
    }

    Ok(())
}
