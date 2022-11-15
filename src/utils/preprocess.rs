// use polars::functions::hor_concat_df;
use polars::prelude::*;

pub const LT_CONTACT: &str = "Noraxon MyoMotion-Segments-Foot LT-Contact";
pub const RT_CONTACT: &str = "Noraxon MyoMotion-Segments-Foot RT-Contact";
pub const DB_SUP: &str = "double_support";
pub const SG_SUP: &str = "single_support";
pub const L_SG_SUP: &str = "LT_single_support";
pub const R_SG_SUP: &str = "RT_single_support";

pub fn rename_df<'a>(
    df: &'a mut DataFrame,
    origs: &Vec<String>,
    news: &Vec<String>,
) -> Result<()> {
    for (o, n) in origs.into_iter().zip(news.into_iter()) {
        df.rename(&o, &n)?;
    }
    Ok(())
}

pub fn remap_contact(mut df: DataFrame) -> Result<DataFrame> {
    df = df
        .lazy()
        .with_columns(vec![
            when(col(LT_CONTACT).eq(lit::<i32>(1000)))
                .then(lit(true))
                .otherwise(lit(false))
                .alias(LT_CONTACT),
            when(col(RT_CONTACT).eq(lit::<i32>(1000)))
                .then(lit(true))
                .otherwise(lit(false))
                .alias(RT_CONTACT),
        ])
        .collect()?;
    Ok(df)
}

pub fn split_support(mut df: DataFrame) -> Result<DataFrame> {
    df = df
        .lazy()
        .with_column(col(LT_CONTACT).and(col(RT_CONTACT)).alias(DB_SUP))
        .with_column(not(col(DB_SUP)).alias(SG_SUP))
        .with_columns(vec![
            col(LT_CONTACT).and(col(SG_SUP)).alias(L_SG_SUP),
            col(RT_CONTACT).and(col(SG_SUP)).alias(R_SG_SUP),
        ])
        // .drop_columns([LT_CONTACT, RT_CONTACT])
        .collect()?;
    Ok(df)
}

pub fn cal_gait(df: &DataFrame) -> Result<DataFrame> {
    // Opt 1: hor_concat_df
    /* let new_df = hor_concat_df(&[
    df.select(["time"])?,
    df.select([DB_SUP])?.shift(1),
    df.select([DB_SUP])?.shift(-1),
    ])?; */

    // Opt 2: hstack
    /* let new_df = df.select(["time"])?.hstack(&[
    df[DB_SUP].shift(1).rename("first").to_owned(),
    df[DB_SUP].shift(-1).rename("second").to_owned(),
    ])?; */

    // shift double_support 1 foward as first
    // use first == 0 & second == 1 as starting index
    let time_df = &df
        .select(["time", DB_SUP])?
        .lazy()
        .with_column(col(DB_SUP).shift(1).alias("first"))
        .with_column(col(DB_SUP).alias("second"))
        .with_columns(vec![when(not(col("first")).and(col("second")))
            .then(lit::<i32>(1))
            .otherwise(lit::<i32>(0))
            .alias("start")])
        .drop_nulls(None)
        .drop_columns([DB_SUP, "first", "second"])
        .collect()?;

    // create start time every two step
    let mut s_vec = time_df
        .filter(&time_df.column("start")?.equal(1)?)?
        .select(["time"])?
        .column("time")?
        .f64()?
        .into_iter()
        .step_by(2)
        .fold(Vec::new(), |mut v, t| {
            v.push(t.unwrap());
            v
        });

    // insert first/last time
    s_vec.insert(0, 0f64);
    s_vec.insert(
        s_vec.len(),
        time_df.tail(Some(1)).column("time")?.f64()?.get(0).unwrap(),
    );

    // gait_vec
    // start: 0 ~ last start
    // end: first start ~ last end
    Ok(df!("start" => &s_vec[..s_vec.len()-1], "end" => &s_vec[1..])?)
}

pub fn cal_x_support(df: &DataFrame, sup_type: &str) -> Result<DataFrame> {
    // shift double_support 1 foward as first
    // use first == 0 & second == 1 as starting index
    // use first == 1 & second == 0 as ending index
    let time_df = &df
        .select(["time", sup_type])?
        .lazy()
        .with_column(col(sup_type).shift(1).alias("first"))
        .with_column(col(sup_type).alias("second"))
        .with_columns(vec![
            when(not(col("first")).and(col("second")))
                .then(lit::<i32>(1))
                .otherwise(lit::<i32>(0))
                .alias("start"),
            when(col("first").and(not(col("second"))))
                .then(lit::<i32>(1))
                .otherwise(lit::<i32>(0))
                .alias("end"),
        ])
        .drop_nulls(None)
        .drop_columns([sup_type, "first", "second"])
        .collect()?;

    let s_vec = time_df
        .filter(&time_df.column("start")?.equal(1)?)?
        .select(["time"])?
        .column("time")?
        .f64()?
        .into_iter()
        .fold(Vec::new(), |mut v, t| {
            v.push(t.unwrap());
            v
        });

    let e_vec = time_df
        .filter(&time_df.column("end")?.equal(1)?)?
        .select(["time"])?
        .column("time")?
        .f64()?
        .into_iter()
        .fold(Vec::new(), |mut v, t| {
            v.push(t.unwrap());
            v
        });

    if sup_type == DB_SUP {
        return Ok(
            df!("start" => &s_vec[..s_vec.len()-1], "end" => &e_vec[1..])?,
        );
    }
    Ok(df!("start" => s_vec, "end" => e_vec)?)
}

pub fn extract_info<P: AsRef<std::path::Path>>(path: P) -> Vec<Vec<String>> {
    let raw_file = std::fs::File::open(path).expect("Can't open raw file");
    let reader_raw = std::io::BufReader::new(raw_file);
    let mut rdr = csv::ReaderBuilder::new()
        .has_headers(false)
        .from_reader(reader_raw);
    let info: Vec<Vec<String>> = rdr
        .records()
        .take(2)
        .map(|row| {
            let r = row.expect("a valid csv entry");
            let mut v = Vec::new();
            for i in 0..r.len() {
                let range = r.range(i).expect("a range");
                let value = &r.as_slice()[range];
                v.push(value.to_string().clone());
            }
            v
        })
        .collect();
    info
    // if info[0].len() != 12 || info[1].len() != 12 {
    //     return false;
    // }
    // return true;
}
