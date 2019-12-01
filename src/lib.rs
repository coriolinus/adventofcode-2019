use std::fs::File;
use std::io::prelude::*;
use std::io::BufReader;
use std::path::Path;
use std::str::FromStr;

pub mod day01;

pub trait Exercise {
    fn part1(&self, path: &Path);
    fn part2(&self, path: &Path);
}

pub fn dispatch(day: u8, path: &Path, part1: bool, part2: bool) {
    if !path.exists() {
        eprintln!("input file at {} not found", path.to_string_lossy());
        return;
    }
    let exercise = match day {
        1 => Some(day01::Day01 {}),
        _ => None,
    };
    match exercise {
        None => {
            eprintln!("exercise {} is not available", day);
        }
        Some(exercise) => {
            if part1 {
                exercise.part1(path);
            }
            if part2 {
                exercise.part2(path);
            }
        }
    }
}

pub fn parse<T>(path: &Path) -> std::io::Result<impl Iterator<Item = T>>
where
    T: FromStr,
{
    let file = File::open(path)?;
    let mut reader = BufReader::new(file);
    let mut buf = String::new();
    Ok(std::iter::from_fn(move || {
        buf.clear();
        reader
            .read_line(&mut buf)
            .map_err(|_| ())
            .and_then(|_| T::from_str(buf.trim()).map_err(|_| ()))
            .ok()
    })
    .fuse())
}
