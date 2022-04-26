use crate::utils::preprocess::*;
use crate::utils::util::*;

use polars::prelude::*;
use serde_json::json;
use std::fs::create_dir_all;
use std::path::Path;

pub fn filter(file: String, save_dir: String) -> Result<()> {
    create_dir_all(&save_dir)?;
    let df_path = Path::new(&save_dir).join(Path::new(&file));
    let gait_path = Path::new(&save_dir).join(Path::new("gait.csv"));
    let ls_path = Path::new(&save_dir).join(Path::new("ls.csv"));
    let rs_path = Path::new(&save_dir).join(Path::new("rs.csv"));
    let db_path = Path::new(&save_dir).join(Path::new("db.csv"));

    use std::time::Instant;
    let now = Instant::now();
    extract_header(&file, df_path.to_str().unwrap());
    let mut header_df =
        CsvReader::from_path(df_path.to_str().unwrap())?.finish()?;
    let elapsed = now.elapsed();
    println!("Elapsed: {:.2?}", elapsed);

    let range = get_range(&header_df);
    save_csv(&mut header_df, df_path.to_str().unwrap());

    let (ori_key, new_key) = get_keys("./name.csv")?;
    let now = Instant::now();
    let mut df = CsvReader::from_path(file)?
        .with_skip_rows(3)
        .with_columns(Some(ori_key.clone()))
        .finish()?;
    let elapsed = now.elapsed();
    println!("Elapsed: {:.2?}", elapsed);

    rename_df(&mut df, &ori_key, &new_key)?;
    df = remap_contact(df)?;
    df = split_support(df)?;

    let mut gait_df = cal_gait(&df)?;
    let mut ls_df = cal_step_support(&df, L_SG_SUP)?;
    let mut rs_df = cal_step_support(&df, R_SG_SUP)?;
    let mut db_df = cal_step_support(&df, DB_SUP)?;

    df = df
        .lazy()
        .drop_columns([DB_SUP, L_SG_SUP, R_SG_SUP, SG_SUP])
        .collect()?;

    let resp_filter_api = json!({
            "FltrFile": {
                "rslt": append_df2header(&mut df, df_path.to_str().unwrap()),
                "cyGt": save_csv(&mut gait_df, gait_path.to_str().unwrap()),
                "cyLt": save_csv(&mut ls_df, ls_path.to_str().unwrap()),
                "cyRt": save_csv(&mut rs_df, rs_path.to_str().unwrap()),
                "cyDb": save_csv(&mut db_df, db_path.to_str().unwrap()),
            },
            "Range": range,
    });
    println!("{}", resp_filter_api.to_string());

    Ok(())
}
