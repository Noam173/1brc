use anyhow::{Context, Result};
use memmap2::Mmap;
use rand::RngExt;
use rand::seq::IndexedRandom;
use rayon::iter::{IntoParallelIterator, ParallelIterator};
use rayon::slice::ParallelSlice;
use std::env::args;
use std::fs::File;
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
    let rows = 0..get_rows()?;
    let map = unsafe { Mmap::map(&File::open(STATIONS)?) }?;
    let mut out = BufWriter::with_capacity(8 * 1024usize.pow(2), File::create(MEASUREMENTS)?);
    let rng = &mut rand::rng();
    let stations: [&[u8]; 10_000] = parse_map(&map)
        .sample_array(rng)
        .context("couldnt sample")?;
    let nums = index_num(rows);
    let mut buff = ryu::Buffer::new();
    for (idx, num) in nums {
        out.write_all(unsafe { stations.get_unchecked(idx) })?;
        out.write_all(buff.format_finite(num).as_bytes())?;
        out.write_all(b"\n")?;
    }
    out.flush()?;
    Ok(())
}
fn index_num(rows: impl IntoParallelIterator) -> Vec<(usize, f32)> {
    rows.into_par_iter()
        .map_init(rand::rng, |rng, _| {
            (
                rng.random_range(0..10_000),
                rng.random_range(COLDEST..=HOTTEST) as f32 / 10.,
            )
        })
        .collect()
}
fn parse_map(map: &[u8]) -> Vec<&[u8]> {
    map.par_split(|b| *b == b'\n')
        .map(|f| f.split_inclusive(|b| *b == b';').next().unwrap_or_default())
        .collect()
}
