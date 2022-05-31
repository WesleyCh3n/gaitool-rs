use crate::utils::util::*;

use polars::prelude::*;
use std::collections::HashMap;
use std::fs::{self, create_dir_all, remove_dir_all};
use std::path::PathBuf;

pub fn check(file_dir: PathBuf) -> Result<()> {
    let save_dir = "./tmp/";
    create_dir_all(save_dir)?;
    let paths = fs::read_dir(&file_dir)?;
    let mut cnt = HashMap::new();
    for file in paths {
        let file = file?;
        let file = file.path();
        let file = file.display().to_string();
        /* read file */
        let filename = get_file_name(&file);
        let file_stem = get_file_stem(&file);
        let saved_path = join_path(&save_dir, &filename.as_str());

        let parts = file_stem.split("_").collect::<Vec<&str>>();
        if parts.len() != 2 {
            println!("Can't parse file name: {}. Skipped!", filename);
            continue;
        }
        // {record_datetime}_{user_id}-{assistant_user_id}-{location}-{posture_id}-[{reason_id},{reason_id}]-{order}.csv
        let mut name_vec = parts[0].split("-").collect::<Vec<&str>>();
        name_vec.append(&mut parts[1].split("-").collect::<Vec<&str>>());

        if name_vec.len() < 11 {
            println!("Can't parse file name: {}. Skipped!", filename);
            continue;
        }
        if name_vec[7] == "1" {
            let count = cnt.entry(name_vec[5].to_string() + "-1").or_insert(0);
            *count += 1;
        } else if name_vec[7] == "2" {
            let count = cnt.entry(name_vec[5].to_string() + "-2").or_insert(0);
            *count += 1;
        }

        /* read/write only header */
        extract_header(&file, &saved_path);
        /* header to dataframe */
        let header_df = CsvReader::from_path(&saved_path)?
            .with_ignore_parser_errors(true)
            .finish()?;
        /* check last_name first_name selection */
        let mut checks = vec![];
        if let Ok(_) = header_df.column("last_name") {
            checks.push("found: last_name");
        }
        if let Ok(_) = header_df.column("first_name") {
            checks.push("found: first_name");
        }
        if let Err(_) = header_df.column("selection") {
            checks.push("not found: selection");
        }
        if !checks.is_empty() {
            println!("{:<50 } {:?}", &file, checks);
        }
    }

    remove_dir_all(save_dir)?;
    println!("{:#?}", cnt);

    Ok(())
}
