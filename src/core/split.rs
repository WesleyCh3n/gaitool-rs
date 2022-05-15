use crate::utils::preprocess::*;
use crate::utils::util::*;

use indicatif::ProgressBar;
use polars::prelude::*;
use std::fs;
use std::path::{Path, PathBuf};

pub fn split(
    file_dir: PathBuf,
    save_dir: PathBuf,
    percent: usize,
) -> Result<()> {
    let paths = fs::read_dir(&file_dir)?;
    let names = fs::read_dir(&file_dir)?;
    let names = names
        .filter_map(|entry| {
            entry.ok().and_then(|e| {
                e.path()
                    .file_name()
                    .and_then(|n| n.to_str().map(|s| String::from(s)))
            })
        })
        .collect::<Vec<String>>();
    let pb = ProgressBar::new(names.len() as u64);
    for file in paths {
        let file = file?;
        let file = file.path();
        let file = file.display().to_string();
        /* read file */
        pb.inc(1);
        let filename = Path::new(&file)
            .file_name()
            .expect("Err get input file stem")
            .to_str()
            .unwrap()
            .to_string();
        let saved_path = Path::new(&save_dir)
            .join(Path::new(&filename))
            .to_str()
            .unwrap()
            .to_string();
        /* TODO: check type */
        let name_vec = filename.split("-").collect::<Vec<&str>>();
        if name_vec[6] == "1" {
            continue;
        }

        /* get remap column csv */
        let (ori_key, new_key) = get_keys("./assets/all.csv")?;
        let mut df = CsvReader::from_path(&file)?
            .with_skip_rows(3)
            // .with_columns(Some(ori_key.clone())) // read only selected column
            .finish()?;
        /* preprocess data df */
        if df.width() > new_key.len() {
            df = df.select(&ori_key)?; // select original key
            rename_df(&mut df, &ori_key, &new_key)?;
        }
        /* preprocess data df */
        df = remap_contact(df)?;
        df = split_support(df)?;

        /* get support df */
        let gait_df = cal_gait(&df)?;
        // let gait_df = gait_df.with_row_count("Id", None)?;
        let middle = gait_df.height() / 2;
        let range = gait_df.height() * percent / 100;
        let start = middle - range / 2;
        let gait_df = gait_df.slice(start as i64, range);
        let range_value = format!(
            "{}-{}",
            gait_df
                .column("start")?
                .head(Some(1))
                .f64()?
                .get(0)
                .unwrap(),
            gait_df.column("end")?.tail(Some(1)).f64()?.get(0).unwrap()
        );

        /* read/write only header */
        extract_header(&file, &saved_path);
        /* header to dataframe */
        let mut header_df = CsvReader::from_path(&saved_path)?.finish()?;
        /* write range to selection column */
        header_df = header_df
            .lazy()
            .with_column(lit(range_value).alias("selection"))
            .drop_columns(["last_name", "first_name"])
            .collect()?;
        /* save modidied header to csv */
        save_csv(&mut header_df, &save_dir.display().to_string(), &filename);
        append_df2header(&mut df, &save_dir.display().to_string(), &filename);
    }
    Ok(())
}
