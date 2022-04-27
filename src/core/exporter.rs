use polars::prelude::*;

use crate::utils::preprocess::*;

pub fn exporter(
    file: String,
    save_dir: String,
    ranges: Vec<(u32, u32)>,
) -> Result<()> {
    let df = CsvReader::from_path(file)?.finish()?;

    let gait_df = cal_gait(&df)?.with_row_count("Id", None)?;
    let _ls_df = cal_x_support(&df, L_SG_SUP)?;
    let _rs_df = cal_x_support(&df, R_SG_SUP)?;
    let _db_df = cal_x_support(&df, DB_SUP)?;

    // println!("{}", gait_df);
    // println!("{}", ls_df);
    // println!("{}", rs_df);
    // println!("{}", db_df);
    for (s, e) in ranges {
        let tmp_df = gait_df.clone()
            .lazy()
            .filter(col("Id").gt_eq(lit(s)).and(col("Id").lt(lit(e))))
            .collect()?;
        // gait_df.slice(s as i64, e as usize)
        println!("{}", tmp_df);
    }
    Ok(())
}
