use memmap2::Mmap;
use rayon::prelude::*;
use rustc_hash::FxHashMap;
use std::{fs::File, io::Write};
const FILE: &str = "data/measurements.txt";
struct City {
    count: u32,
    min: i32,
    max: i32,
    sum: i32,
}
impl City {
    fn new(temp: i32) -> Self {
        City {
            count: 1,
            min: temp,
            max: temp,
            sum: temp,
        }
    }
    fn update(&mut self, temp: i32) {
        self.count += 1;
        self.min = self.min.min(temp);
        self.max = self.max.max(temp);
        self.sum += temp;
    }
    fn merge(&mut self, other: &City) {
        self.count += other.count;
        self.min = self.min.min(other.min);
        self.max = self.max.max(other.max);
        self.sum += other.sum;
    }
}
fn parse_temp(temp: &[u8]) -> anyhow::Result<i32> {
    let temp: f32 = lexical_core::parse(temp)?;
    Ok((10. * temp) as i32)
}

fn main() -> anyhow::Result<()> {
    let start = std::time::Instant::now();
    let mut out = std::io::stdout().lock();
    let file = File::open(FILE)?;
    let mmap = unsafe { Mmap::map(&file)? };
    let data: &[u8] = &mmap;
    let map: FxHashMap<&[u8], City> = data
        .par_split(|&b| b == b'\n')
        .fold(
            || FxHashMap::<&[u8], City>::with_capacity_and_hasher(10_000, Default::default()),
            |mut map, line| {
                let mut parts = line.split(|&b| b == b';');
                if let (Some(name), Some(temp)) = (parts.next(), parts.next()) {
                    let temp = parse_temp(temp).unwrap_or(0);
                    map.entry(name)
                        .and_modify(|f| f.update(temp))
                        .or_insert(City::new(temp));
                }
                map
            },
        )
        .reduce(
            || FxHashMap::with_capacity_and_hasher(10_000, Default::default()),
            |mut sum_map, map| {
                for (k, v) in map {
                    sum_map.entry(k).and_modify(|f| f.merge(&v)).or_insert(v);
                }
                sum_map
            },
        );
    let mut v: Vec<_> = map.into_iter().collect();
    v.sort_unstable_by(|a, b| a.0.cmp(b.0));
    let mut min_buff = ryu::Buffer::new();
    let mut sum_buff = ryu::Buffer::new();
    let mut max_buff = ryu::Buffer::new();
    let mut line = Vec::with_capacity(64);
    for (k, v) in v {
        let min = v.min as f32 / 10.;
        let sum = v.sum as f32 / v.count as f32 / 10.;
        let max = v.max as f32 / 10.;
        line.clear();
        line.extend_from_slice(min_buff.format_finite(min).as_bytes());
        line.push(b' ');
        line.extend_from_slice(sum_buff.format_finite(sum).as_bytes());
        line.push(b' ');
        line.extend_from_slice(max_buff.format_finite(max).as_bytes());
        line.push(b'\n');
        out.write_all(k)?;
        out.write_all(&line)?;
    }
    out.flush()?;
    println!("took {:.3}s", start.elapsed().as_secs_f32());
    Ok(())
}
