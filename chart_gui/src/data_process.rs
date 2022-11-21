use crate::config::{Position, Variable};
use polars::{io::csv::CsvReader, prelude::*};
use std::{
    collections::HashMap,
    path::Path,
    sync::{
        atomic::{AtomicBool, Ordering},
        mpsc::Sender,
    },
    thread,
};

#[derive(Default, Debug)]
pub struct Quantile<T: Copy> {
    min: T,
    q1: T,
    mid: T,
    q3: T,
    max: T,
}

impl<T: Copy> Quantile<T> {
    pub fn new(nums: (T, T, T, T, T)) -> Self {
        Self {
            min: nums.0,
            q1: nums.1,
            mid: nums.2,
            q3: nums.3,
            max: nums.4,
        }
    }
    pub fn to_tuple(&self) -> (T, T, T, T, T) {
        let Self {
            min,
            q1,
            mid,
            q3,
            max,
        } = self;
        (*min, *q1, *mid, *q3, *max)
    }
    pub fn max(&self) -> T {
        self.max
    }
    pub fn min(&self) -> T {
        self.min
    }
}

#[derive(Default)]
pub struct RawData {
    pub x: Vec<f64>,
    pub y: HashMap<
        Position,
        HashMap<Variable, (Vec<f64>, Quantile<f64>, Quantile<f64>)>,
    >,
    pub l_contact: Vec<i64>,
    pub r_contact: Vec<i64>,
    pub db: Quantile<f64>,
    pub lt: Quantile<f64>,
    pub rt: Quantile<f64>,
}

pub struct DataInfo {
    pub path: String,
    pub raw: RawData,
}

pub enum Message {
    Nothing,
    Running(f32, String),
    Abort(String),
    Done(Vec<DataInfo>),
}

pub struct Manager {
    external_sender: Sender<Message>,
    must_stop: Arc<AtomicBool>,
}

impl Manager {
    pub fn new(external_sender: Sender<Message>) -> Self {
        Self {
            external_sender,
            must_stop: Arc::new(AtomicBool::new(false)),
        }
    }

    pub fn stop(&mut self) {
        self.must_stop.store(true, Ordering::Relaxed)
    }

    pub fn clear_message(&mut self) {
        self.external_sender.send(Message::Nothing).unwrap();
    }

    pub fn start_get_data<P: AsRef<Path> + std::marker::Send + 'static>(
        &self,
        input_dir: P,
    ) {
        let external_sender = self.external_sender.clone();
        let must_stop = self.must_stop.clone();
        must_stop.store(false, Ordering::Relaxed);
        thread::spawn(move || {
            let mut file_lists = Vec::new();
            let files: Vec<std::fs::DirEntry> = std::fs::read_dir(input_dir)
                .unwrap()
                .map(|e| e.unwrap())
                .collect();
            for (i, file) in files.iter().enumerate() {
                if must_stop.load(Ordering::Relaxed) {
                    external_sender
                        .send(Message::Abort("Stopped!".into()))
                        .unwrap();
                    return;
                }
                external_sender
                    .send(Message::Running(
                        (i as f32 + 1.) / files.len() as f32,
                        format!("{}", file.file_name().into_string().unwrap()),
                    ))
                    .unwrap();
                let info = Self::extract_info(file.path());
                if info[0].len() != 12 || info[1].len() != 12 {
                    external_sender
                        .send(Message::Abort("info not 12 len".to_owned()))
                        .unwrap();
                    break;
                }
                if info[0][5] != String::from("exported with version") {
                    external_sender
                        .send(Message::Abort("no version info".to_owned()))
                        .unwrap();
                    break;
                }
                let selection = info[1][11]
                    .split(" ")
                    .collect::<Vec<&str>>()
                    .iter()
                    .map(|s| {
                        s.split("-")
                            .collect::<Vec<&str>>()
                            .iter()
                            .map(|n| n.parse::<f64>().unwrap())
                            .collect::<Vec<f64>>()
                    })
                    .collect::<Vec<Vec<f64>>>();
                file_lists.push(DataInfo {
                    path: file.file_name().to_str().unwrap().to_string(),
                    raw: RawData::parse_file(file.path(), selection).unwrap(),
                })
            }
            external_sender.send(Message::Done(file_lists)).unwrap();
        });
    }

    pub fn extract_info<P: AsRef<std::path::Path>>(
        path: P,
    ) -> Vec<Vec<String>> {
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
    }
}

impl RawData {
    pub fn parse_file<P: AsRef<std::path::Path>>(
        path: P,
        selections: Vec<Vec<f64>>,
    ) -> Result<Self> {
        let raw_df = CsvReader::from_path(path.as_ref())?
            .with_skip_rows(3)
            .finish()?;

        let contact_df = raw_df
            .clone()
            .lazy()
            .select(vec![
                col("time"),
                when(
                    col("Noraxon MyoMotion-Segments-Foot LT-Contact")
                        .eq(lit::<i32>(1000)),
                )
                .then(lit(true))
                .otherwise(lit(false))
                .alias("LT"),
                when(
                    col("Noraxon MyoMotion-Segments-Foot RT-Contact")
                        .eq(lit::<i32>(1000)),
                )
                .then(lit(true))
                .otherwise(lit(false))
                .alias("RT"),
            ])
            .with_column(col("LT").and(col("RT")).alias("DB"))
            .with_column(not(col("DB")).alias("SG"))
            .with_columns(vec![
                col("LT").and(col("SG")).alias("LS"),
                col("RT").and(col("SG")).alias("RS"),
            ])
            .collect()?;

        let contact_df = contact_df
            .lazy()
            // DB start/end
            .with_column(col("DB").shift(1).alias("first"))
            .with_column(col("DB").alias("second"))
            .with_columns(vec![
                when(not(col("first")).and(col("second")))
                    .then(lit::<i32>(1))
                    .otherwise(lit::<i32>(0))
                    .alias("DB_S"),
                when(col("first").and(not(col("second"))))
                    .then(lit::<i32>(1))
                    .otherwise(lit::<i32>(0))
                    .alias("DB_E"),
            ])
            .drop_columns(["first", "second"])
            // LT start/end
            .with_column(col("LT").shift(1).alias("first"))
            .with_column(col("LT").alias("second"))
            .with_columns(vec![
                when(not(col("first")).and(col("second")))
                    .then(lit::<i32>(1))
                    .otherwise(lit::<i32>(0))
                    .alias("LT_S"),
                when(col("first").and(not(col("second"))))
                    .then(lit::<i32>(1))
                    .otherwise(lit::<i32>(0))
                    .alias("LT_E"),
            ])
            .drop_columns(["first", "second"])
            // RT start/end
            .with_column(col("RT").shift(1).alias("first"))
            .with_column(col("RT").alias("second"))
            .with_columns(vec![
                when(not(col("first")).and(col("second")))
                    .then(lit::<i32>(1))
                    .otherwise(lit::<i32>(0))
                    .alias("RT_S"),
                when(col("first").and(not(col("second"))))
                    .then(lit::<i32>(1))
                    .otherwise(lit::<i32>(0))
                    .alias("RT_E"),
            ])
            .drop_columns(["first", "second"])
            .drop_nulls(None)
            .collect()?;

        let gait_ranges = contact_df
            .clone()
            .lazy()
            .select([col("time"), col("DB_S")])
            .filter(col("DB_S").eq(1))
            .collect()?
            // get db start time to vec
            .column("time")?
            .f64()?
            .into_no_null_iter()
            // get every 2 db
            .step_by(2)
            .collect::<Vec<f64>>()
            // create start end
            .windows(2)
            .map(|s| s.to_vec())
            // valid range from selction
            .filter(|r| {
                // r in one of the selection
                for sel in selections.iter() {
                    if sel[0] <= r[0] && r[1] <= sel[1] {
                        return true;
                    }
                }
                false
            })
            .collect::<Vec<Vec<f64>>>();
        let db_gaps = get_support_range(contact_df.clone(), "DB", &selections)?
            .into_iter()
            .map(|v| v[1] - v[0])
            .collect::<Vec<f64>>();
        let lt_gaps = get_support_range(contact_df.clone(), "LT", &selections)?
            .into_iter()
            .map(|v| v[1] - v[0])
            .collect::<Vec<f64>>();
        let rt_gaps = get_support_range(contact_df.clone(), "RT", &selections)?
            .into_iter()
            .map(|v| v[1] - v[0])
            .collect::<Vec<f64>>();

        let x = raw_df
            .column("time")?
            .f64()?
            .into_no_null_iter()
            .collect::<Vec<f64>>();

        let mut y = HashMap::new();
        for p in Position::iterator() {
            let mut variables = HashMap::new();
            for v in Variable::iterator() {
                let col_name = Variable::to_name_string(v, p);
                let data = raw_df
                    .column(&col_name)?
                    .f64()?
                    .into_no_null_iter()
                    .collect::<Vec<f64>>();
                let quantiles = get_min_max_quantile(&data, &x, &gait_ranges)?;

                variables.entry(v.clone()).or_insert((
                    data,
                    quantiles.0,
                    quantiles.1,
                ));
            }
            y.entry(p.clone()).or_insert(variables);
        }

        Ok(Self {
            x,
            y,
            l_contact: raw_df
                .column("Noraxon MyoMotion-Segments-Foot LT-Contact")?
                .i64()?
                .into_no_null_iter()
                .collect::<Vec<i64>>(),
            r_contact: raw_df
                .column("Noraxon MyoMotion-Segments-Foot RT-Contact")?
                .i64()?
                .into_no_null_iter()
                .collect::<Vec<i64>>(),
            db: get_quantile(&db_gaps)?,
            lt: get_quantile(&lt_gaps)?,
            rt: get_quantile(&rt_gaps)?,
        })
    }
}

fn get_support_range(
    contact_df: DataFrame, // support dataframe which has DB/LT/RT start/end
    pos: &str,
    selections: &Vec<Vec<f64>>,
) -> Result<Vec<Vec<f64>>> {
    let ranges = contact_df
        .lazy()
        .select([
            col("time"),
            col(format!("{pos}_S").as_str()),
            col(format!("{pos}_E").as_str()),
        ])
        .filter(
            col(format!("{pos}_S").as_str())
                .eq(1)
                .or(col(format!("{pos}_E").as_str()).eq(1)),
        )
        .collect()?
        .column("time")?
        .f64()?
        .into_no_null_iter()
        .collect::<Vec<f64>>();
    assert_eq!(ranges.len() % 2, 0);

    let ranges = (&ranges[1..ranges.len() - 1])
        .to_vec()
        .windows(2)
        .map(|s| s.to_vec())
        .step_by(2)
        .filter(|r| {
            // r in one of the selection
            for sel in selections.iter() {
                if sel[0] <= r[0] && r[1] <= sel[1] {
                    return true;
                }
            }
            false
        })
        .collect::<Vec<Vec<f64>>>();
    Ok(ranges)
}

pub fn get_min_max_quantile(
    data: &Vec<f64>,
    time: &Vec<f64>,
    ranges: &Vec<Vec<f64>>,
) -> Result<(Quantile<f64>, Quantile<f64>)> {
    // in gait min / max mean
    // also in valid range
    let df = df!(
        "data" => data,
        "time" => time,
    )?;

    // get min in every step
    let mut min_vec = Vec::new();
    let mut max_vec = Vec::new();
    for range in ranges {
        let min_df = df
            .clone()
            .lazy()
            .filter(col("time").gt_eq(range[0]).and(col("time").lt(range[1])))
            .select([
                col("data").min().suffix("_min"),
                col("data").max().suffix("_max"),
            ])
            .collect()?;
        let arr = min_df.to_ndarray::<Float64Type>()?.row(0).to_vec();
        min_vec.push(arr[0]);
        max_vec.push(arr[1]);
    }
    let min_q = get_quantile(&min_vec)?;
    let max_q = get_quantile(&max_vec)?;
    Ok((min_q, max_q))
}

fn get_quantile(data: &Vec<f64>) -> Result<Quantile<f64>> {
    let mut df = df!("data" => data)?;

    df = df
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
    let v = df.to_ndarray::<Float64Type>()?.row(0).to_vec();
    Ok(Quantile::new((v[0], v[1], v[2], v[3], v[4])))
}
