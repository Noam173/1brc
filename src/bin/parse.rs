use anyhow::Result;
use memmap2::Mmap;
use rayon::{
    iter::{IntoParallelRefIterator, ParallelIterator},
    slice::ParallelSlice,
};
use rustc_hash::FxHashMap;
use ryu::Buffer;
use std::{
    fs::File,
    io::{Write, stdout},
};
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
fn parse_temp(temp: &[u8]) -> Result<i32> {
    let temp: f32 = lexical_core::parse(temp)?;
    Ok((10. * temp) as i32)
}

fn main() -> Result<()> {
    let mut out = stdout().lock();
    let file = File::open(FILE)?;
    let map = unsafe { Mmap::map(&file)? };
    let map: FxHashMap<&[u8], City> = parse_map(&map);
    let mut v: Vec<_> = map.into_iter().collect();
    v.sort_unstable_by(|a, b| a.0.cmp(b.0));
    let v: Vec<_> = v
        .par_iter()
        .map(|(k, v)| {
            (
                *k,
                v.min as f32 / 10.,
                v.sum as f32 / v.count as f32 / 10.,
                v.max as f32 / 10.,
            )
        })
        .collect();
    stdout_write(v, &mut out)?;
    out.flush()?;
    Ok(())
}
fn stdout_write(v: Vec<(&[u8], f32, f32, f32)>, out: &mut impl Write) -> Result<()> {
    let mut buf = Buffer::new();
    let mut line: Vec<_> = Vec::with_capacity(20);
    v.into_iter()
        .try_for_each(|(name, min, sum, max)| -> Result<()> {
            line.extend_from_slice(name);
            line.extend_from_slice(buf.format_finite(min).as_bytes());
            line.push(b'/');
            line.extend_from_slice(buf.format_finite(sum).as_bytes());
            line.push(b'/');
            line.extend_from_slice(buf.format_finite(max).as_bytes());
            line.push(b'\n');
            out.write_all(&line)?;
            Ok(())
        })?;
    Ok(())
}
fn parse_map(map: &[u8]) -> FxHashMap<&[u8], City> {
    map.par_split(|b| *b == b'\n')
        .fold(
            || FxHashMap::<&[u8], City>::with_capacity_and_hasher(10_000, Default::default()),
            |mut map, line| {
                let mut parts = line.split(|b| *b == b';');
                if let (Some(name), Some(temp)) = (parts.next(), parts.next()) {
                    let temp = parse_temp(temp).unwrap_or_default();
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
        )
}
