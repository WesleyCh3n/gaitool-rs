use crate::config::{Position, Variable};
use polars::{io::csv::CsvReader, prelude::SerReader};
use std::collections::HashMap;

pub struct RawData {
    pub x: Vec<f64>,
    pub y: HashMap<Position, HashMap<Variable, Vec<f64>>>,
}

impl RawData {
    pub fn new<P: AsRef<std::path::Path>>(path: P) -> Self {
        Self::get_data(path).unwrap()
        // Self {
        //     x: vec![],
        //     y: HashMap::new(),
        // }
    }

    pub fn get_data<P: AsRef<std::path::Path>>(
        path: P,
    ) -> Result<Self, polars::error::PolarsError> {
        let df = CsvReader::from_path(path.as_ref())?
            .with_skip_rows(3)
            .finish()?;

        let x = df
            .column("time")?
            .f64()?
            .into_no_null_iter()
            .collect::<Vec<f64>>();
        println!("{:?}", df);
        println!("x len: {:?}", x.len());

        let mut y: HashMap<Position, HashMap<Variable, Vec<f64>>> =
            HashMap::new();
        for p in Position::iterator() {
            let mut variables: HashMap<Variable, Vec<f64>> = HashMap::new();
            for v in Variable::iterator() {
                variables.entry(v.clone()).or_insert(
                    df.column(Variable::to_name_string(v, p).as_str())?
                        .f64()?
                        .into_no_null_iter()
                        .collect::<Vec<f64>>(),
                );
            }
            y.entry(p.clone()).or_insert(variables);
        }
        println!("keys: {:?}", y.keys());
        // insert gait time
        Ok(Self { x, y })
    }
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
