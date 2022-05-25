use crate::utils::util::*;

use polars::prelude::*;
use std::fs::{self, create_dir_all};
use std::path::{Path, PathBuf};

pub fn check(file_dir: PathBuf) -> Result<()> {
    let save_dir = "./tmp/";
    create_dir_all(save_dir)?;
    let paths = fs::read_dir(&file_dir)?;
    for file in paths {
        let file = file?;
        let file = file.path();
        let file = file.display().to_string();
        /* read file */
        let filename = Path::new(&file)
            .file_name()
            .expect("Err get input file stem")
            .to_str()
            .unwrap()
            .to_string();
        let saved_path = Path::new(&save_dir)
            .join(Path::new(&filename))
            .to_str()
            .unwrap()
            .to_string();

        let name_vec = filename.split("-").collect::<Vec<&str>>();
        if name_vec.len() < 10 {
            continue;
        }
        /* if name_vec[6] == "1" {
            continue;
        } */

        /* read/write only header */
        extract_header(&file, &saved_path);
        /* header to dataframe */
        let header_df = CsvReader::from_path(&saved_path)?.finish()?;
        /* check last_name first_name selection */
        let mut results = vec![];
        match header_df.column("last_name") {
            Ok(_) => {
                results.push("found: last_name");
            }
            Err(_) => (),
        }
        match header_df.column("first_name") {
            Ok(_) => {
                results.push("found: first_name");
            }
            Err(_) => (),
        }
        match header_df.column("selection") {
            Ok(_) => (),
            Err(_) => {
                results.push("not found: selection");
            }
        }
        if results.len() != 0 {
            println!("{:<50 } {:?}", &file, results);
        }
    }
    Ok(())
}
