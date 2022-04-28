use polars::functions::hor_concat_df;
use polars::prelude::*;
use serde_json::json;
use std::path::{Path, PathBuf};

use crate::utils::preprocess::*;
use crate::utils::util::*;

pub fn exporter(
    file: PathBuf,
    save_dir: PathBuf,
    ranges: Vec<(u32, u32)>,
) -> Result<()> {
    /* extract file name */
    let filename = file
        .file_name()
        .expect("Err get input file name")
        .to_str()
        .unwrap()
        .to_string();
    /* file name without suffix */
    let outfile = Path::new(&filename)
        .file_stem()
        .expect("Err get input file stem")
        .to_str()
        .unwrap()
        .to_string();

    let mut df = CsvReader::from_path(file)?.finish()?;

    let gait_df = cal_gait(&df)?;
    /* calculate every gap */
    let gait_ldf = gait_df
        .clone()
        .lazy()
        .with_column((col("end") - col("start")).alias("gait mean"));
    let ls_ldf = cal_x_support(&df, L_SG_SUP)?
        .lazy()
        .with_column((col("end") - col("start")).alias("ls mean"));
    let rs_ldf = cal_x_support(&df, R_SG_SUP)?
        .lazy()
        .with_column((col("end") - col("start")).alias("rs mean"));
    let db_ldf = cal_x_support(&df, DB_SUP)?
        .lazy()
        .with_column((col("end") - col("start")).alias("db mean"));
    df = df
        .lazy()
        .drop_columns([DB_SUP, L_SG_SUP, R_SG_SUP, SG_SUP])
        .collect()?;

    let mut vec_ranges: Vec<(f64, f64)> = vec![]; // for calculate valid data
    let mut str_ranges: Vec<String> = vec![]; // for output selection
    let mut gait_ldfs = vec![];
    let mut ls_ldfs = vec![];
    let mut rs_ldfs = vec![];
    let mut db_ldfs = vec![];
    for (start, end) in ranges {
        let gait_slice = gait_df.slice(start as i64, (end - start) as usize);
        gait_slice["start"]
            .f64()?
            .into_iter()
            .zip(gait_slice["end"].f64()?.into_iter())
            .for_each(|(s, e)| {
                vec_ranges.push((s.unwrap(), e.unwrap()));
            });

        let t_start = gait_df["start"].f64()?.get(start as usize).unwrap();
        let t_end = gait_df["start"].f64()?.get(end as usize).unwrap();
        str_ranges.push(format!("{}-{}", t_start, t_end));
        /* add valid ranges in gait/ls/rs/db between time start/end */
        let expr = col("start")
            .gt_eq(lit(t_start))
            .and(col("start").lt(lit(t_end)));
        let sel_col = &[col("^*mean$")];
        gait_ldfs.push(gait_ldf.clone().filter(expr.clone()).select(sel_col));
        ls_ldfs.push(ls_ldf.clone().filter(expr.clone()).select(sel_col));
        rs_ldfs.push(rs_ldf.clone().filter(expr.clone()).select(sel_col));
        db_ldfs.push(db_ldf.clone().filter(expr.clone()).select(sel_col));
    }
    let gt_mean = concat(gait_ldfs, true)?.mean().collect()?;
    let ls_mean = concat(ls_ldfs, true)?.mean().collect()?;
    let rs_mean = concat(rs_ldfs, true)?.mean().collect()?;
    let db_mean = concat(db_ldfs, true)?.mean().collect()?;

    /* iter valid step get max/min amoung all col in data */
    let lazy_dfs = vec_ranges.iter().fold(Vec::new(), |mut v, (start, end)| {
        v.push(
            df.clone()
                .lazy()
                .filter(
                    col("time")
                        .gt_eq(lit(*start))
                        .and(col("time").lt(lit(*end))),
                )
                .with_column(all().exclude(&["time"]).max().suffix("_max"))
                .with_column(all().exclude(&["time"]).min().suffix("_min")),
        );
        v
    });
    let data_df = concat(lazy_dfs, true)?
        .mean()
        .drop_columns(&["time"])
        .collect()?;

    /* basic info column */
    let info_df = df![
        "filename" => &[filename.clone()],
        "selection" => &[str_ranges.join(" ")],
    ]?;

    /* concat all column */
    let mut result_df =
        hor_concat_df(&[info_df, gt_mean, ls_mean, rs_mean, db_mean, data_df])?;

    /* stdout result api */
    let resp_filter_api = json!({
        "ExportFile": save_csv(&mut result_df, &save_dir.display().to_string(), &format!("{}-result.csv", outfile)),
    })
    .to_string();
    println!("{}", resp_filter_api);
    Ok(())
}
