const FILE: &str = "./measurements.txt";
use memmap2::Mmap;
use rayon::prelude::*;
use rustc_hash::FxHashMap;
use std::{fs::File, io::Write};
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
    for (k, v) in v {
        out.write_all(k)?;
        writeln!(
            out,
            " {:.1}, {:.1}, {:.1}",
            v.min as f32 / 10.0,
            v.sum as f32 / v.count as f32 / 10.0,
            v.max as f32 / 10.0
        )?;
    }
    out.flush()?;
    Ok(())
}
