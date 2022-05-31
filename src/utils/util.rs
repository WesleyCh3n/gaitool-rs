use polars::prelude::{
    CsvReader, CsvWriter, DataFrame, Result, SerReader, SerWriter,
    TakeRandomUtf8,
};
use serde_json::{json, Value};
use std::io::prelude::*;
use std::io::{LineWriter, Write};
use std::path::Path;

/// get remap column name csv
pub fn get_keys(path: &str) -> Result<(Vec<String>, Vec<String>)> {
    let dict = CsvReader::from_path(path)?.finish()?;
    let ori_key = dict["Original"].utf8()?.into_iter().fold(
        Vec::with_capacity(dict.height()),
        |mut v, k| {
            v.push(k.unwrap().to_string());
            v
        },
    );
    let new_key = dict["New"].utf8()?.into_iter().fold(
        Vec::with_capacity(dict.height()),
        |mut v, k| {
            v.push(k.unwrap().to_string());
            v
        },
    );
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
pub fn save_csv<'a>(
    mut df: &mut DataFrame,
    save_dir: &'a str,
    filename: &'a str,
) -> &'a str {
    let file_path = Path::new(&save_dir).join(Path::new(&filename));
    let mut file = std::fs::File::create(file_path).unwrap();

    CsvWriter::new(&mut file)
        .has_header(true)
        .with_delimiter(b',')
        .finish(&mut df)
        .unwrap();
    filename
}

/// append df to header.csv
pub fn append_df2header<'a>(
    mut df: &mut DataFrame,
    save_dir: &'a str,
    filename: &'a str,
) -> &'a str {
    let file_path = Path::new(&save_dir).join(Path::new(&filename));
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
    filename
}

/// get range from header df
pub fn get_range(df: &DataFrame) -> Vec<Value> {
    match df.column("selection") {
        Ok(s) => match s.utf8().unwrap().get(0) {
            Some(ranges) => {
                let ranges = ranges
                    .split(' ')
                    .map(str::to_string)
                    .collect::<Vec<String>>();
                ranges.iter().fold(
                    Vec::with_capacity(ranges.len()),
                    |mut v, r| {
                        let range = r
                            .split('-')
                            .take(2)
                            .map(str::to_string)
                            .collect::<Vec<String>>();
                        v.push(json!({
                            "Start": range[0].parse::<f64>().unwrap(),
                            "End": range[1].parse::<f64>().unwrap()
                        }));
                        v
                    },
                )
            }
            None => {
                vec![]
            }
        },
        Err(..) => vec![],
    }
}

pub fn get_file_name<P: AsRef<Path>>(input_path: P) -> String {
    input_path
        .as_ref()
        .file_name()
        .unwrap_or_else(|| {
            panic!(
                "Couldn't get file name from {}",
                input_path.as_ref().display()
            )
        })
        .to_str()
        .unwrap_or_else(|| {
            panic!(
                "Couldn't parse file name to str from {}",
                input_path.as_ref().display()
            )
        })
        .to_string()
}

pub fn get_file_stem<P: AsRef<Path>>(input_path: P) -> String {
    input_path
        .as_ref()
        .file_stem()
        .unwrap_or_else(|| {
            panic!(
                "Couldn't get file stem from {}",
                input_path.as_ref().display()
            )
        })
        .to_str()
        .unwrap_or_else(|| {
            panic!(
                "Couldn't parse file stem to str from {}",
                input_path.as_ref().display()
            )
        })
        .to_string()
}

pub fn join_path<P: AsRef<Path>>(path: P, input: P) -> String {
    path.as_ref()
        .join(input.as_ref())
        .to_str()
        .unwrap_or_else(|| {
            panic!(
                "Couldn't join path from {} to {}",
                path.as_ref().display(),
                input.as_ref().display()
            )
        })
        .to_string()
}
