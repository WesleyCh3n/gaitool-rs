mod preprocess;

use polars::prelude::*;

use preprocess::*;

fn filter(file: String) -> Result<()> {
    let (ori_key, new_key) = get_keys("./name.csv")?;
    let mut df = CsvReader::from_path(file)?
        .with_skip_rows(3)
        .with_columns(Some(ori_key.clone()))
        .finish()?;

    rename_df(&mut df, &ori_key, &new_key)?;

    df = remap_contact(df)?;
    println!("{}", df.select([LT_CONTACT, RT_CONTACT])?);

    df = split_support(df)?;
    println!("{}", df.select([DB_SUP, SG_SUP, L_SG_SUP, R_SG_SUP])?);

    let gait_df = cal_gait(&df)?;
    println!("{}", gait_df);

    /* let mut file = std::fs::File::create("sample.csv")?;
    CsvWriter::new(&mut file)
        .has_header(true)
        .with_delimiter(b',')
        .finish(&mut gait_df)?; */

    Ok(())
}

fn main() {
    filter("./v3.18.44-en-sample.csv".to_string()).unwrap();
}
