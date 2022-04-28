use polars::prelude::*;
use serde_json::json;
use std::fs::create_dir_all;
use std::path::PathBuf;

use crate::utils::util::*;

pub fn concater(files: Vec<PathBuf>, save_dir: PathBuf) -> Result<()> {
    create_dir_all(&save_dir)?;

    /* read all files ioto LazyFrame */
    let ldfs = files.iter().fold(Vec::new(), |mut v, file| {
        v.push(
            LazyCsvReader::new(file.as_path().display().to_string())
                .finish()
                .expect(
                    format!(
                        "Read {} failed",
                        file.as_path().display().to_string()
                    )
                    .as_str(),
                ),
        );
        v
    });

    /* concat all */
    let mut concat_df = concat(ldfs, true)?.collect()?;

    let resp_filter_api = json!({"ConcatFile": save_csv(
        &mut concat_df,
        &save_dir.display().to_string(),
        "concat.csv",
    )})
    .to_string();
    println!("{}", resp_filter_api);
    Ok(())
}
