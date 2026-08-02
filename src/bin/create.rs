use anyhow::{Context, Result};
use indicatif::ProgressIterator;
use rand::RngExt;
use rand::distr::{Distribution, Uniform};
use rand::seq::IndexedRandom;
use rayon::iter::ParallelIterator;
use rayon::str::ParallelString;
use rustc_hash::FxHashSet;
use std::env::args;
use std::fs::{File, read_to_string};
use std::io::{BufWriter, Write};
const COLDEST: i32 = -999;
const HOTTEST: i32 = 999;
const STATIONS: &str = "data/weather_stations.csv";
const MEASUREMENTS: &str = "data/measurements.txt";
const MAX_ROWS: u32 = 1_000_000_000;
fn get_rows() -> Result<u32> {
    let rows = match args().nth(1) {
        Some(s) => s.parse::<u32>()?,
        None => MAX_ROWS,
    };
    Ok(rows)
}
fn main() -> Result<()> {
    let start = std::time::Instant::now();
    let rows = get_rows()?;
    let src = read_to_string(STATIONS)?;
    let mut dst = BufWriter::with_capacity(64 * 1024usize.pow(2), File::create(MEASUREMENTS)?);
    let uni = Uniform::new(COLDEST, HOTTEST)?;
    let rng = &mut rand::rng();
    let stations: Vec<&str> = src
        .par_lines()
        .map(|f| f.split_once(';').map(|(name, _)| name).unwrap_or_default())
        .collect::<FxHashSet<_>>()
        .into_iter()
        .collect();
    let station_names_10k: [_; 10_000] = stations.sample_array(rng).context("couldnt sample")?;
    let nums: Vec<(usize, f32)> = (0..rows)
        .into_iter()
        .map(|_| (rng.random_range(0..10_000), uni.sample(rng) as f32 / 10.))
        .collect();
    let mut buff = ryu::Buffer::new();
    for (idx, num) in nums.into_iter().progress() {
        dst.write_all(station_names_10k[idx].as_bytes())?;
        dst.write_all(buff.format_finite(num).as_bytes())?;
        dst.write_all(b"\n")?;
    }
    dst.flush()?;
    println!("took {:.3?}", start.elapsed());
    Ok(())
}
