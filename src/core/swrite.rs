use crate::utils::preprocess::*;
use crate::utils::util::*;

use polars::prelude::*;
use serde_json::json;
use std::fs::create_dir_all;
use std::path::{Path, PathBuf};

pub fn swrite(
    file: PathBuf,
    save_dir: PathBuf,
    range_value: String,
) -> Result<()> {
    create_dir_all(&save_dir)?;
    /* output file path */
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

    /* read/write only header */
    extract_header(&file.display().to_string(), &saved_path);
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

    /* get remap column csv */
    let (ori_key, new_key) = get_keys("./assets/name.csv")?;
    let mut df = CsvReader::from_path(file)?
        .with_skip_rows(3)
        .with_columns(Some(ori_key.clone())) // read only selected column
        .finish()?;

    /* preprocess data df */
    rename_df(&mut df, &ori_key, &new_key)?;

    /* stdout result api */
    let resp_filter_api = json!({
        "CleanFile": append_df2header(&mut df, &save_dir.display().to_string(), &filename),
    })
    .to_string();
    println!("{}", resp_filter_api);

    Ok(())
}
