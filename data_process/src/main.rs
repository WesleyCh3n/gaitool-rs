use polars::prelude::*;

fn main() -> Result<()> {
    let raw_df = CsvReader::from_path(
        "C:\\Users\\Wesley\\Downloads\\資料\\output\\走路\\2022-11-02-17-00_101-8-1-1-[1]-1.csv",
    )?
    .with_skip_rows(3)
    .finish()?;
    println!("{}", raw_df);
    let df =
        raw_df
            .clone()
            .lazy()
            .select(vec![
                col("time"),
                when(
                    col("Noraxon MyoMotion-Segments-Foot LT-Contact")
                        .eq(lit::<i32>(1000)),
                )
                .then(lit(true))
                .otherwise(lit(false))
                .alias("LT"),
                when(
                    col("Noraxon MyoMotion-Segments-Foot RT-Contact")
                        .eq(lit::<i32>(1000)),
                )
                .then(lit(true))
                .otherwise(lit(false))
                .alias("RT"),
            ])
            .with_column(col("LT").and(col("RT")).alias("DB"))
            .with_column(not(col("DB")).alias("SG"))
            .with_columns(vec![
                col("LT").and(col("SG")).alias("LS"),
                col("RT").and(col("SG")).alias("RS"),
            ])
            .collect()?;

    let df = df
        .lazy()
        // Gait Start
        .with_column(col("DB").shift(1).alias("first"))
        .with_column(col("DB").alias("second"))
        .with_columns(vec![
            when(not(col("first")).and(col("second")))
                .then(lit::<i32>(1))
                .otherwise(lit::<i32>(0))
                .alias("DB_S"),
            when(col("first").and(not(col("second"))))
                .then(lit::<i32>(1))
                .otherwise(lit::<i32>(0))
                .alias("DB_E"),
        ])
        .drop_columns(["first", "second"])
        // LT Start/end
        .with_column(col("LT").shift(1).alias("first"))
        .with_column(col("LT").alias("second"))
        .with_columns(vec![
            when(not(col("first")).and(col("second")))
                .then(lit::<i32>(1))
                .otherwise(lit::<i32>(0))
                .alias("LT_S"),
            when(col("first").and(not(col("second"))))
                .then(lit::<i32>(1))
                .otherwise(lit::<i32>(0))
                .alias("LT_E"),
        ])
        .drop_columns(["first", "second"])
        // RT Start/end
        .with_column(col("RT").shift(1).alias("first"))
        .with_column(col("RT").alias("second"))
        .with_columns(vec![
            when(not(col("first")).and(col("second")))
                .then(lit::<i32>(1))
                .otherwise(lit::<i32>(0))
                .alias("RT_S"),
            when(col("first").and(not(col("second"))))
                .then(lit::<i32>(1))
                .otherwise(lit::<i32>(0))
                .alias("RT_E"),
        ])
        .drop_columns(["first", "second"])
        .drop_nulls(None)
        .collect()?;
    println!("df: {}", df);

    let gait_ranges = df
        .clone()
        .lazy()
        .select([col("time"), col("DB_S")])
        .filter(col("DB_S").eq(1))
        .collect()?
        // get db start time to vec
        .column("time")?
        .f64()?
        .into_no_null_iter()
        .collect::<Vec<f64>>()
        // get every 2 db
        .into_iter()
        .step_by(2)
        .map(|v| v)
        .collect::<Vec<f64>>()
        // create start end
        .windows(2)
        .map(|s| s.to_vec())
        .collect::<Vec<Vec<f64>>>();
    println!("ranges: {:?}", gait_ranges);

    let selections = vec![vec![8.87, 17.55], vec![29.04, 37.54]];
    let gait_ranges = gait_ranges
        .into_iter()
        .filter(|r| {
            // r in one of the selection
            let mut is_valid = false;
            for sel in selections.iter() {
                if sel[0] <= r[0] && r[1] <= sel[1] {
                    is_valid = true;
                    break;
                }
            }
            is_valid
        })
        .collect::<Vec<Vec<f64>>>();
    println!("filter ranges: {:?}", gait_ranges);

    let db_times = get_support_range(df.clone(), "DB", &selections)?
        .into_iter()
        .map(|v| v[1] - v[0])
        .collect::<Vec<f64>>();
    let lt_times = get_support_range(df.clone(), "LT", &selections)?
        .into_iter()
        .map(|v| v[1] - v[0])
        .collect::<Vec<f64>>();
    let rt_times = get_support_range(df.clone(), "RT", &selections)?
        .into_iter()
        .map(|v| v[1] - v[0])
        .collect::<Vec<f64>>();
    // println!("{:?}", db_times);
    // println!("{:?}", lt_times);
    // println!("{:?}", rt_times);

    let data = raw_df
        .column("L Accel Sensor X (mG)")?
        .f64()?
        .into_no_null_iter()
        .collect::<Vec<f64>>();
    let x = raw_df
        .column("time")?
        .f64()?
        .into_no_null_iter()
        .collect::<Vec<f64>>();
    let quantile = get_min_max_quantile(&data, &x, &gait_ranges)?;
    println!("quantile: {:?}", quantile);
    Ok(())
}

fn get_support_range(
    df: DataFrame, // support dataframe which has DB/LT/RT start/end
    pos: &str,
    selections: &Vec<Vec<f64>>,
) -> Result<Vec<Vec<f64>>> {
    let ranges = df
        .lazy()
        .select([
            col("time"),
            col(format!("{pos}_S").as_str()),
            col(format!("{pos}_E").as_str()),
        ])
        .filter(
            col(format!("{pos}_S").as_str())
                .eq(1)
                .or(col(format!("{pos}_E").as_str()).eq(1)),
        )
        .collect()?
        .column("time")?
        .f64()?
        .into_no_null_iter()
        .collect::<Vec<f64>>();
    assert_eq!(ranges.len() % 2, 0);

    let ranges = (&ranges[1..ranges.len() - 1])
        .to_vec()
        .windows(2)
        .map(|s| s.to_vec())
        .step_by(2)
        .filter(|r| {
            // r in one of the selection
            for sel in selections.iter() {
                if sel[0] <= r[0] && r[1] <= sel[1] {
                    return true;
                }
            }
            false
        })
        .collect::<Vec<Vec<f64>>>();
    Ok(ranges)
}

pub fn get_min_max_quantile(
    data: &Vec<f64>,
    time: &Vec<f64>,
    ranges: &Vec<Vec<f64>>,
) -> Result<(f64, f64, f64, f64, f64)> {
    // in gait min / max mean
    // also in valid range
    let mut df = df!(
        "data" => data,
        "time" => time,
    )?;

    // get min in every step
    let mut min_vec = Vec::new();
    let mut max_vec = Vec::new();
    for range in ranges {
        let min_df = df
            .clone()
            .lazy()
            .filter(col("time").gt_eq(range[0]).and(col("time").lt(range[1])))
            .select([
                col("data").min().suffix("_min"),
                col("data").max().suffix("_max"),
            ])
            .collect()?;
        let arr = min_df.to_ndarray::<Float64Type>()?.row(0).to_vec();
        min_vec.push(arr[0]);
        max_vec.push(arr[1]);
    }
    let quantile = get_quantile(&min_vec)?;
    Ok(quantile)
}

fn get_quantile(data: &Vec<f64>) -> Result<(f64, f64, f64, f64, f64)> {
    let mut df = df!("data" => data)?;

    df = df
        .lazy()
        .select([
            all().min().suffix("_min"),
            all()
                .quantile(0.25, QuantileInterpolOptions::Nearest)
                .suffix("_Q1"),
            all().median().suffix("_median"),
            all()
                .quantile(0.75, QuantileInterpolOptions::Nearest)
                .suffix("_Q3"),
            all().max().suffix("_max"),
        ])
        .collect()?;
    println!("df: {}", df);

    let v = df.to_ndarray::<Float64Type>()?.row(0).to_vec();
    Ok((v[0], v[1], v[2], v[3], v[4]))
}
