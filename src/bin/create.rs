use anyhow::{Context, Result};
use memmap2::Mmap;
use rand::distr::{Distribution, Uniform};
use rand::seq::IndexedRandom;
use rayon::prelude::*;
use rustc_hash::FxHashSet as HashSet;
use std::env::args;
use std::fs::File;
use std::io::{BufWriter, Write};
const COLDEST: f32 = -99.9;
const HOTTEST: f32 = 99.9;
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
    let src = File::open(STATIONS)?;
    let mut dst = BufWriter::new(File::create(MEASUREMENTS)?);
    let mmap = unsafe { Mmap::map(&src) }?;
    let data: &[u8] = &mmap;
    let uni = Uniform::new(COLDEST, HOTTEST)?;
    let rng = &mut rand::rng();
    let stations: Vec<&[u8]> = data
        .par_split(|&b| b == b'\n')
        .map(|f| f.split_inclusive(|&b| b == b';').next().unwrap_or_default())
        .collect::<HashSet<_>>()
        .into_iter()
        .collect();
    let station_names_10k: [&[u8]; 10_000] =
        stations.sample_array(rng).context("couldnt sample")?;
    let nums: Vec<f32> = (0..rows)
        .into_par_iter()
        .map_init(
            || rand::rng(),
            |rng, _| (10. * uni.sample(rng)).round() / 10.,
        )
        .collect();
    let mut buff = ryu::Buffer::new();
    for num in nums.into_iter() {
        let station = station_names_10k.choose(rng).context("couldnt choose")?;
        dst.write_all(station)?;
        dst.write_all(buff.format_finite(num).as_bytes())?;
        dst.write_all(b"\n")?;
    }
    dst.flush()?;
    println!("took {:.3}s", start.elapsed().as_secs_f32());
    Ok(())
}
