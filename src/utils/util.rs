use polars::prelude::*;
use std::io::prelude::*;
use std::io::{LineWriter, Write};

/// get remap column name csv
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

/// extract header to smaller file
pub fn extract_header(path: &str, output: &str) {
    let raw_file = std::fs::File::open(&path).expect("Can't open raw file");
    let reader_raw = std::io::BufReader::new(raw_file);
    let header_file =
        std::fs::File::create(output).expect("Can't create header_file");
    let mut writer_header = LineWriter::new(header_file);
    reader_raw.lines().take(2).for_each(|l| {
        let mut l = l.unwrap();
        l.push('\n');
        writer_header
            .write_all(l.as_bytes())
            .expect("write line to header");
    });
}

/// write df into new csv
pub fn save_csv<'a>(mut df: &mut DataFrame, path: &'a str) -> &'a str {
    let mut file = std::fs::File::create(path).unwrap();

    CsvWriter::new(&mut file)
        .has_header(true)
        .with_delimiter(b',')
        .finish(&mut df)
        .unwrap();
    path
}

/// append df to header.csv
pub fn append_df2header<'a>(
    mut df: &mut DataFrame,
    file_path: &'a str,
) -> &'a str {
    let mut file = std::fs::OpenOptions::new()
        .append(true)
        .open(file_path)
        .unwrap();

    writeln!(file, "").expect("Err add blank line between header and data");

    CsvWriter::new(&mut file)
        .has_header(true)
        .with_delimiter(b',')
        .finish(&mut df)
        .unwrap();
    file_path
}

/// get range from header df
pub fn get_range(df: &DataFrame) -> String {
    match df.column("selection") {
        Ok(s) => match s.utf8().unwrap().get(0) {
            Some(range) => range.to_string(),
            None => "\"\"".to_string(),
        },
        Err(..) => "\"\"".to_string(),
    }
}
