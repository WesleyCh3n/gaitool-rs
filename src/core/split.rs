use crate::utils::preprocess::*;
use crate::utils::util::*;

use polars::prelude::*;
use std::borrow::Cow;
use std::fs::create_dir_all;
use std::path::{Path, PathBuf};

pub fn split(
    file: &PathBuf,
    save_dir: &PathBuf,
    percent: usize,
    remap_csv: &PathBuf,
    mut c: Option<Box<dyn FnMut(&String) -> ()>>,
) -> Result<()> {
    create_dir_all(&save_dir)?;
    let (ori_key, new_key) = get_keys(remap_csv.to_str().unwrap())
        .unwrap_or_else(|e| panic!("{:?} {}", remap_csv, e));
    let file = file.display().to_string();
    if let Some(ref mut c) = c {
        c(&file);
    }
    /* read file */
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
    let name_vec = filename.split("-").collect::<Vec<&str>>();
    if name_vec.len() < 10 {
        return Err(PolarsError::InvalidOperation(Cow::Borrowed(
            "Parse name failed",
        )));
    }
    match load_csv(&file, &ori_key, &new_key) {
        Ok(mut df) => {
            /* preprocess data df */
            let mut export_df = df.clone();
            df = remap_contact(df)?;
            df = split_support(df)?;

            /* get support df */
            let gait_df = cal_gait(&df)?;
            let range_value = if name_vec[6] == "2" {
                let range_df = get_select_df(
                    &gait_df,
                    gait_df.height(),
                    gait_df.height() / 2,
                    percent,
                );
                get_range_string(&range_df)
            } else if name_vec[6] == "1" {
                let half = gait_df.height() / 2;
                let range_df_1 =
                    get_select_df(&gait_df, half, half / 2, percent);
                let range_df_2 = get_select_df(
                    &gait_df,
                    half,
                    half + if gait_df.height() % 2 == 0 { 1 } else { 0 }
                        + (half / 2),
                    percent,
                );
                format!(
                    "{} {}",
                    get_range_string(&range_df_1),
                    get_range_string(&range_df_2)
                )
            } else {
                "".to_string()
            };

            extract_header(&file, &saved_path);
            let mut header_df = CsvReader::from_path(&saved_path)?.finish()?;
            header_df = header_df
                .lazy()
                .with_column(lit(range_value).alias("selection"))
                .drop_columns(["last_name", "first_name"])
                .collect()?;
            save_csv(
                &mut header_df,
                &save_dir.display().to_string(),
                &filename,
            );
            append_df2header(
                &mut export_df,
                &save_dir.display().to_string(),
                &filename,
            );
            println!("{}: Success", file);
            return Ok(());
        }
        Err(e) => {
            println!("{}: {}", file, e);
            return Err(e);
        }
    }
}

fn load_csv<P, K>(filename: P, ori_key: K, new_key: K) -> Result<DataFrame>
where
    P: AsRef<Path>,
    K: AsRef<Vec<String>>,
{
    let mut df = CsvReader::from_path(filename.as_ref())?
        .with_skip_rows(3)
        .finish()?;
    /* preprocess data df */
    if df.width() > new_key.as_ref().len() {
        df = df.select(ori_key.as_ref())?; // select original key
        rename_df(&mut df, ori_key.as_ref(), new_key.as_ref())?;
    }
    Ok(df)
}

fn get_select_df(
    df: &DataFrame,
    length: usize,
    middle: usize,
    percent: usize,
) -> DataFrame {
    let range = length * percent / 100;
    let start = middle - range / 2;
    df.slice(start as i64, range)
}

fn get_range_string(df: &DataFrame) -> String {
    format!(
        "{}-{}",
        df.column("start")
            .unwrap()
            .head(Some(1))
            .f64()
            .unwrap()
            .get(0)
            .unwrap(),
        df.column("end")
            .unwrap()
            .tail(Some(1))
            .f64()
            .unwrap()
            .get(0)
            .unwrap()
    )
}
