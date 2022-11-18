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

#[derive(Default)]
pub struct RawData {
    pub x: Vec<f64>,
    pub y: HashMap<
        Position,
        HashMap<Variable, (Vec<f64>, (f64, f64, f64, f64, f64))>,
    >,
    pub l_contact: Vec<i64>,
    pub r_contact: Vec<i64>,
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
                file_lists.push(DataInfo {
                    path: file.file_name().to_str().unwrap().to_string(),
                    raw: RawData::parse_file(file.path()).unwrap(),
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
    pub fn parse_file<P: AsRef<std::path::Path>>(path: P) -> Result<Self> {
        let df = CsvReader::from_path(path.as_ref())?
            .with_skip_rows(3)
            .finish()?;

        let x = df
            .column("time")?
            .f64()?
            .into_no_null_iter()
            .collect::<Vec<f64>>();

        let mut y = HashMap::new();
        for p in Position::iterator() {
            let mut variables = HashMap::new();
            for v in Variable::iterator() {
                let col_name = Variable::to_name_string(v, p);
                let data = df
                    .column(&col_name)?
                    .f64()?
                    .into_no_null_iter()
                    .collect::<Vec<f64>>();
                let quantile = get_quantile(&data, &x, vec![vec![7.75, 8.87]])?;
                variables.entry(v.clone()).or_insert((data, quantile));
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

        // calculate gait
        // get lt/rt/db
        let df = df
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
            .collect()?;

        let df = df
            .lazy()
            // Gait Start
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
            // LT Start/end
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
            // RT Start/end
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

        println!(
            "DB s: {}",
            df.clone().lazy().select([col("DB_S")]).collect()?
        );

        let a = df
            .clone()
            .lazy()
            .filter(col("DB_S").eq(1).or(col("DB_E").eq(1)))
            .select(vec![col("time"), col("DB_S"), col("DB_E")])
            .with_column(col("time").shift(1).alias("drop_first_row"))
            .drop_nulls(None)
            .drop_columns(["drop_first_row"])
            .with_column(col("time").shift(1).alias("time_shift"))
            .with_column((col("time") - col("time_shift")).alias("gap"))
            .select(vec![col("gap")])
            .select([
                col("gap").min().suffix("_min"),
                col("gap")
                    .quantile(0.25, QuantileInterpolOptions::Nearest)
                    .suffix("_Q1"),
                col("gap").median().suffix("_median"),
                col("gap")
                    .quantile(0.75, QuantileInterpolOptions::Nearest)
                    .suffix("_Q3"),
                col("gap").max().suffix("_max"),
            ])
            .collect()?;
        let arr = a.to_ndarray::<Float64Type>()?;
        let v = arr.row(0).to_vec();

        Ok(Self {
            x,
            y,
            l_contact,
            r_contact,
        })
    }
}

pub fn get_quantile(
    data: &Vec<f64>,
    time: &Vec<f64>,
    ranges: Vec<Vec<f32>>,
) -> Result<(f64, f64, f64, f64, f64)> {
    // in gait min / max mean
    // also in valid range
    let mut df = df!(
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
    let mut df = df!(
        "min" => min_vec,
        "max" => max_vec,
    )?;

    df = df
        .lazy()
        .select([
            all().exclude(["time"]).min().suffix("_min"),
            all()
                .exclude(["time"])
                .quantile(0.25, QuantileInterpolOptions::Nearest)
                .suffix("_Q1"),
            all().exclude(["time"]).median().suffix("_median"),
            all()
                .exclude(["time"])
                .quantile(0.75, QuantileInterpolOptions::Nearest)
                .suffix("_Q3"),
            all().exclude(["time"]).max().suffix("_max"),
        ])
        .collect()?;

    let v = df.to_ndarray::<Float64Type>()?.row(0).to_vec();
    Ok((v[0], v[1], v[2], v[3], v[4]))
}
