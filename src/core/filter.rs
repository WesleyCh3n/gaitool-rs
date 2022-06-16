use crate::utils::preprocess::*;
use crate::utils::util::*;

use polars::prelude::*;
use serde_json::{json, Value};
use std::fs::create_dir_all;
use std::path::{Path, PathBuf};

pub fn filter(
    file: PathBuf,
    save_dir: PathBuf,
    remap_csv: PathBuf,
    filter_csv: PathBuf,
) -> Result<Value> {
    create_dir_all(&save_dir)?;
    /* output file path */
    let filename = file
        .file_name()
        .expect("Err get input file stem")
        .to_str()
        .unwrap()
        .to_string();
    let saved_path = save_dir
        .join(Path::new(&filename))
        .to_str()
        .unwrap()
        .to_string();

    /* read/write only header */
    extract_header(&file.display().to_string(), &saved_path);
    /* header to dataframe */
    let header_df = CsvReader::from_path(&saved_path)?.finish()?;
    /* get selection range from header */
    let sel_range = get_range(&header_df);

    /* get remap column csv */
    // let (ori_key, new_key) = get_keys("./assets/filter.csv")?;
    let dict = CsvReader::from_path(filter_csv.to_str().unwrap())
        .unwrap_or_else(|e| panic!("{:?} {}", filter_csv, e))
        .finish()
        .unwrap_or_else(|e| panic!("{:?} {}", filter_csv, e));
    let filter_key = dict["New"].utf8()?.into_iter().fold(
        Vec::with_capacity(dict.height()),
        |mut v, k| {
            v.push(k.unwrap().to_string());
            v
        },
    );
    let (raw_key, new_key) = get_keys(remap_csv.to_str().unwrap())
        .unwrap_or_else(|e| panic!("{:?} {}", remap_csv, e));
    let mut df = CsvReader::from_path(file)?.with_skip_rows(3).finish()?;

    // if key is not selected, means not remaped
    if df.width() > new_key.len() {
        df = df.select(&raw_key)?; // select original key
        rename_df(&mut df, &raw_key, &new_key)?;
    }
    // select for web
    df = df.select(&filter_key)?;

    // pre-calculation
    df = df
        .lazy()
        .with_columns(vec![
            // convert mG to SI m^2/s
            col("^.*mG.*$") * lit::<f64>(9.80665) * lit::<f64>(0.001),
            // X axis - Gravity Accel
            col("^.*X.*mG.*$") * lit::<f64>(9.80665) * lit::<f64>(0.001)
                - lit::<f64>(9.80665),
        ])
        .collect()?;

    /* preprocess data df */
    df = remap_contact(df)?;
    df = split_support(df)?;

    /* get support df */
    let mut gait_df = cal_gait(&df)?;
    let mut ls_df = cal_x_support(&df, L_SG_SUP)?;
    let mut rs_df = cal_x_support(&df, R_SG_SUP)?;
    let mut db_df = cal_x_support(&df, DB_SUP)?;

    /* stdout result api */
    let resp_filter_api = json!({
            "FltrFile": {
                "rslt": save_csv(&mut df, &save_dir.display().to_string(), &filename),
                "cyGt": save_csv(&mut gait_df, &save_dir.display().to_string(), "gait.csv"),
                "cyLt": save_csv(&mut ls_df, &save_dir.display().to_string(), "ls.csv"),
                "cyRt": save_csv(&mut rs_df, &save_dir.display().to_string(), "rs.csv"),
                "cyDb": save_csv(&mut db_df, &save_dir.display().to_string(), "db.csv"),
            },
            "Range": sel_range,
    });

    Ok(resp_filter_api)
}
