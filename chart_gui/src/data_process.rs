use crate::config::{Position, Variable};
use polars::{io::csv::CsvReader, prelude::*};
use std::collections::HashMap;

pub struct RawData {
    pub x: Vec<f64>,
    pub y: HashMap<
        Position,
        HashMap<Variable, (Vec<f64>, f64, f64, f64, f64, f64)>,
    >,
    pub l_contact: Vec<i64>,
    pub r_contact: Vec<i64>,
}

impl RawData {
    pub fn new<P: AsRef<std::path::Path>>(path: P) -> Self {
        Self::get_data(path).unwrap()
    }

    pub fn get_data<P: AsRef<std::path::Path>>(path: P) -> Result<Self> {
        let df = CsvReader::from_path(path.as_ref())?
            .with_skip_rows(3)
            .finish()?;

        let x = df
            .column("time")?
            .f64()?
            .into_no_null_iter()
            .collect::<Vec<f64>>();
        let df_quantile = df
            .clone()
            .lazy()
            .select([
                all().min().suffix("_min"),
                all()
                    .quantile(0.25, QuantileInterpolOptions::Nearest)
                    .suffix("_Q1"),
                all().median().suffix("_median"),
                all()
                    .quantile(0.75, QuantileInterpolOptions::Nearest)
                    .suffix("_Q3"),
                all().max().suffix("_max"),
            ])
            .collect()?;

        let mut y = HashMap::new();
        for p in Position::iterator() {
            let mut variables = HashMap::new();
            for v in Variable::iterator() {
                let col_name = Variable::to_name_string(v, p);
                variables.entry(v.clone()).or_insert((
                    df.column(&col_name)?
                        .f64()?
                        .into_no_null_iter()
                        .collect::<Vec<f64>>(),
                    df_quantile
                        .column(&format!("{}_min", col_name))?
                        .f64()?
                        .get(0)
                        .unwrap(),
                    df_quantile
                        .column(&format!("{}_Q1", col_name))?
                        .f64()?
                        .get(0)
                        .unwrap(),
                    df_quantile
                        .column(&format!("{}_median", col_name))?
                        .f64()?
                        .get(0)
                        .unwrap(),
                    df_quantile
                        .column(&format!("{}_Q3", col_name))?
                        .f64()?
                        .get(0)
                        .unwrap(),
                    df_quantile
                        .column(&format!("{}_max", col_name))?
                        .f64()?
                        .get(0)
                        .unwrap(),
                ));
            }
            y.entry(p.clone()).or_insert(variables);
        }
        let l_contact = df
            .column("Noraxon MyoMotion-Segments-Foot LT-Contact")?
            .i64()?
            .into_no_null_iter()
            .collect::<Vec<i64>>();
        let r_contact = df
            .column("Noraxon MyoMotion-Segments-Foot RT-Contact")?
            .i64()?
            .into_no_null_iter()
            .collect::<Vec<i64>>();
        Ok(Self {
            x,
            y,
            l_contact,
            r_contact,
        })
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
