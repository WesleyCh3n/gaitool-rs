use crate::utils::preprocess::*;
use crate::utils::util::*;

use polars::prelude::*;
use std::fs;
use std::path::{Path, PathBuf};

pub fn split(file_dir: PathBuf, save_dir: PathBuf) -> Result<()> {
    let paths = fs::read_dir(file_dir)?;
    for path in paths {
        /* read file */
        let file = path?.path();
        /* TODO: check type */
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
        let mut gait_df = cal_gait(&df)?;
        println!("{}", gait_df);

        /* read/write only header */
        extract_header(&file.display().to_string(), &saved_path);
        /* header to dataframe */
        let mut header_df = CsvReader::from_path(&saved_path)?.finish()?;
        /* write range to selection column */
        header_df = header_df
            .lazy()
            // .with_column(lit(range_value).alias("selection"))
            .drop_columns(["last_name", "first_name"])
            .collect()?;
        /* save modidied header to csv */
        save_csv(&mut header_df, &save_dir.display().to_string(), &filename);
        append_df2header(&mut df, &save_dir.display().to_string(), &filename);
    }
    Ok(())
}
