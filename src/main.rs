use aoc2019::dispatch;
use chrono::{Datelike, Utc};
use std::path::PathBuf;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(name = "aoc2019", about = "advent of code 2019")]
struct Opt {
    /// input file
    #[structopt(parse(from_os_str))]
    input: PathBuf,

    /// day (default: today's date)
    #[structopt(short, long)]
    day: Option<u8>,

    /// skip part 1
    #[structopt(long = "no-part1")]
    no_part1: bool,

    /// run part 2
    #[structopt(long)]
    part2: bool,
}

fn main() {
    let opt = Opt::from_args();
    dispatch(
        opt.day.unwrap_or_else(|| Utc::now().day() as u8),
        &opt.input,
        !opt.no_part1,
        opt.part2,
    )
}
