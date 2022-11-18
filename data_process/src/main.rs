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

    let df_db = df
        .clone()
        .lazy()
        .select([col("time"), col("DB_S")])
        .filter(col("DB_S").eq(1))
        .collect()?;
    let db_start_vec = df_db
        .column("time")?
        .f64()?
        .into_no_null_iter()
        .collect::<Vec<f64>>();
    println!("DB s: {:?}", db_start_vec);
    let gait_vec = db_start_vec
        .into_iter()
        .step_by(2)
        .map(|v| v)
        .collect::<Vec<f64>>();
    println!("gait_vec: {:?}", gait_vec);
    let ranges = gait_vec
        .windows(2)
        .map(|s| s.to_vec())
        .collect::<Vec<Vec<f64>>>();
    println!("ranges: {:?}", ranges);

    for range in ranges {
        let step = raw_df
            .clone()
            .lazy()
            .filter(col("time").gt_eq(range[0]).and(col("time").lt(range[1])))
            .select([all().min().suffix("_min"), all().max().suffix("_max")])
            .collect()?;
        println!("range: {:?}", range);
        println!("Step: {}", step);
        break;
    }

    Ok(())
}
