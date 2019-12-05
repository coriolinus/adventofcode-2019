use std::fs::File;
use std::io::prelude::*;
use std::io::BufReader;
use std::path::Path;
use std::str::FromStr;

pub mod day01;
pub mod day02;
pub mod day03;
pub mod day04;
pub mod geometry;
pub mod intcode;

pub trait Exercise {
    fn part1(&self, path: &Path);
    fn part2(&self, path: &Path);
}

pub fn dispatch(day: u8, path: &Path, part1: bool, part2: bool) {
    if !path.exists() {
        eprintln!("input file at {} not found", path.to_string_lossy());
        return;
    }
    let exercise: Option<Box<dyn Exercise>> = match day {
        1 => Some(Box::new(day01::Day01 {})),
        2 => Some(Box::new(day02::Day02 {})),
        3 => Some(Box::new(day03::Day03 {})),
        4 => Some(Box::new(day04::Day {})),
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

/// adaptor which plugs into parse, splitting comma-separated items from the line
///
/// This can be flattened or consumed by line, as required
pub struct CommaSep<T>(Vec<T>);

impl<T> FromStr for CommaSep<T>
where
    T: FromStr,
{
    type Err = <T as FromStr>::Err;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        s.split(',')
            .map(str::parse)
            .collect::<Result<Vec<_>, _>>()
            .map(|ts| CommaSep(ts))
    }
}

impl<T> IntoIterator for CommaSep<T> {
    type Item = T;
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}
