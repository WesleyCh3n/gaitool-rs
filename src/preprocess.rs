// use polars::functions::hor_concat_df;
use polars::prelude::*;

pub const LT_CONTACT: &str = "Noraxon MyoMotion-Segments-Foot LT-Contact";
pub const RT_CONTACT: &str = "Noraxon MyoMotion-Segments-Foot RT-Contact";
pub const DB_SUP: &str = "double_support";
pub const SG_SUP: &str = "single_support";
pub const L_SG_SUP: &str = "LT_single_support";
pub const R_SG_SUP: &str = "RT_single_support";

pub fn get_keys(path: &str) -> Result<(Vec<String>, Vec<String>)> {
    let dict = CsvReader::from_path(path)?.finish()?;
    let ori_key =
        dict["Original"]
            .utf8()?
            .into_iter()
            .fold(Vec::new(), |mut v, k| {
                v.push(k.unwrap().to_string());
                v
            });
    let new_key =
        dict["New"]
            .utf8()?
            .into_iter()
            .fold(Vec::new(), |mut v, k| {
                v.push(k.unwrap().to_string());
                v
            });
    return Ok((ori_key, new_key));
}

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
        .drop_columns([LT_CONTACT, RT_CONTACT])
        .collect()?;
    Ok(df)
}

pub fn cal_gait(mut df: DataFrame) -> Result<DataFrame> {
    // Opt 1: hor_concat_df
    /* let new_df = hor_concat_df(&[
        df.select(["time"])?,
        df.select([DB_SUP])?.shift(1),
        df.select([DB_SUP])?.shift(1),
    ])?; */

    // Opt 2: hstack
    /* let new_df = df.select(["time"])?.hstack(&[
        df[DB_SUP].shift(1).rename("first").to_owned(),
        df[DB_SUP].shift(-1).rename("second").to_owned(),
    ])?; */

    df = df
        .select(["time", DB_SUP])?
        .lazy()
        .with_column(col(DB_SUP).shift(1).alias("first"))
        .with_column(col(DB_SUP).shift(-1).alias("second"))
        .drop_columns([DB_SUP])
        .with_columns(vec![
            when(not(col("first")).and(col("second")))
                .then(lit(true))
                .otherwise(lit(false))
                .alias("start"),
            when(col("first").and(not(col("second"))))
                .then(lit(true))
                .otherwise(lit(false))
                .alias("end"),
        ])
        .collect()?;

    println!("{}", df);
    Ok(df)
}
