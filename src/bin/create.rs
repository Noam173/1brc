use anyhow::{Context, Result};
use rand::distr::{Distribution, Uniform};
use rand::seq::IndexedRandom;
use std::env::args;
use std::fs::File;
use std::io::{BufRead, BufReader, BufWriter, Write};
pub const STATIONS: &str = "data/weather_stations.csv";
const MEASUREMENTS: &str = "data/measurements.txt";
fn get_rows() -> Result<u32> {
    let rows = match args().nth(1) {
        Some(s) => s.parse::<u32>()?,
        None => 1_000_000_000,
    };
    Ok(rows)
}
fn build_weather_station_name_list() -> Result<Vec<String>> {
    let mut station_name: Vec<String> = Vec::new();
    let file = BufReader::new(File::open(STATIONS)?);
    for station in file.lines() {
        let station = station?;
        if !station.contains("#") {
            let name = station.split(";").next().context("contanes no ';'")?;
            station_name.push(name.into());
        }
    }
    station_name.dedup();
    Ok(station_name)
}
pub fn build_test_data() -> Result<()> {
    let weather_station_names: Vec<String> = build_weather_station_name_list()?;
    let start = std::time::Instant::now();
    let n_rows = get_rows()?;
    let rng = &mut rand::rng();
    let mut file = BufWriter::new(File::create(MEASUREMENTS)?);
    let coldest_temp: f32 = -99.9;
    let hottest_temp: f32 = 99.9;
    let uni = Uniform::new(coldest_temp, hottest_temp)?;
    let station_names_10k: Vec<_> = weather_station_names
        .sample(rng, 10_000)
        .map(|f| {
            let rng = &mut rand::rng();
            let temp = uni.sample(rng);
            format!("{};{:.1}", f, temp)
        })
        .collect();
    for _ in 0..n_rows {
        let line = station_names_10k.choose(rng).context("idk")?;
        file.write_all(line.as_bytes())?;
        file.write_all(b"\n")?;
    }
    file.flush()?;
    println!("took {:?}", start.elapsed());
    Ok(())
}
fn main() -> Result<()> {
    build_test_data()?;
    Ok(())
}
