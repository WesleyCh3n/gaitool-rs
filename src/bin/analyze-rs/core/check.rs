use crate::utils::util::*;

use polars::prelude::*;
use std::fs::{self, create_dir_all, remove_dir_all};
use std::path::{Path, PathBuf};
use std::collections::HashMap;

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
        let filename = get_file_name(Path::new(&file));
        let file_stem = get_file_stem(Path::new(&file));
        let saved_path = join_path(Path::new(&save_dir), Path::new(&filename));

        let parts = file_stem.split("_").collect::<Vec<&str>>();
        if parts.len() != 2 {
            println!("Can't parse file name: {}", filename);
            continue;
        }
        let mut name_vec = parts[0].split("-").collect::<Vec<&str>>();
        name_vec.append(&mut parts[1].split("-").collect::<Vec<&str>>());

        println!("{:?}", name_vec);
        if name_vec.len() < 11 {
            println!("Can't parse file name: {}", filename);
            continue;
        }
        let count = cnt.entry(name_vec[5].to_string()).or_insert(0);
        *count += 1;

        /* read/write only header */
        extract_header(&file, &saved_path);
        /* header to dataframe */
        let header_df = CsvReader::from_path(&saved_path)?.finish()?;
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
