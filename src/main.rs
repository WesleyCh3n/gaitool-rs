mod preprocess;

use polars::prelude::*;

use preprocess::*;

fn save_csv(mut df: DataFrame, path: &str) {
    let mut file = std::fs::File::create(path).unwrap();

    CsvWriter::new(&mut file)
        .has_header(true)
        .with_delimiter(b',')
        .finish(&mut df).unwrap();
}

fn filter(file: String) -> Result<()> {
    let (ori_key, new_key) = get_keys("./name.csv")?;
    let mut df = CsvReader::from_path(file)?
        .with_skip_rows(3)
        .with_columns(Some(ori_key.clone()))
        .finish()?;

    rename_df(&mut df, &ori_key, &new_key)?;

    df = remap_contact(df)?;
    println!("{}", df.select(["time"])?);
    // println!("{}", df.select([LT_CONTACT, RT_CONTACT])?);

    df = split_support(df)?;
    // println!("{}", df.select([DB_SUP, SG_SUP, L_SG_SUP, R_SG_SUP])?);

    let _gait_df = cal_gait(&df)?;
    println!("{}", _gait_df);

    let _ls_df = cal_step_support(&df, L_SG_SUP)?;
    println!("{}", _ls_df);
    let _rs_df = cal_step_support(&df, R_SG_SUP)?;
    println!("{}", _rs_df);
    let _db_df = cal_step_support(&df, DB_SUP)?;
    println!("{}", _db_df);

    save_csv(_gait_df, "gait.csv");
    save_csv(_db_df, "db.csv");

    Ok(())
}

fn main() {
    filter("./v3.18.44-en-sample.csv".to_string()).unwrap();
}
