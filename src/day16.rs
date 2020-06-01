use crate::Exercise;
use rayon::prelude::*;
use std::path::Path;

pub struct Day;

fn read_input(path: &Path) -> Vec<i8> {
    let mut data = std::fs::read_to_string(path).unwrap();
    data.truncate(
        data.len()
            - data
                .as_bytes()
                .iter()
                .rev()
                .take_while(|c| c.is_ascii_whitespace())
                .count(),
    );

    as_i8(data)
}

// this is silly and too-low-level: it makes a lot of assumptions about its input
// it's fast, though
fn as_i8(data: String) -> Vec<i8> {
    let mut data = data.into_bytes();
    for byte in data.iter_mut() {
        // convert each byte from its ascii representation into a u8
        *byte -= b'0';
    }

    // this is probably fine: a vector of u8 must also be valid as the same vector of i8
    let data_i8 =
        unsafe { Vec::from_raw_parts(data.as_mut_ptr() as *mut i8, data.len(), data.capacity()) };
    // now that we have a new vec over this data, we have to forget the old one so we don't deallocate it.
    std::mem::forget(data);
    data_i8
}

fn show(data: &[i8]) -> String {
    let mut out = String::with_capacity(data.len());
    for d in data.iter() {
        out.push((*d as u8 + b'0') as char);
    }
    out
}

fn calculate_element<P>(input: &[i8], pattern: P) -> i8
where
    P: Iterator<Item = i8> + Send,
{
    (input
        .iter()
        .zip(pattern)
        .par_bridge()
        .filter(|(_, p)| *p != 0)
        .map(|(i, p)| (*i * p) as i64)
        .sum::<i64>()
        .abs()
        % 10) as i8
}

const BASE_PATTERN: [i8; 4] = [0, 1, 0, -1];

fn pattern_for(idx: usize) -> impl Iterator<Item = i8> {
    BASE_PATTERN
        .iter()
        .map(move |e| std::iter::repeat(*e).take(idx + 1))
        .flatten()
        .cycle()
        .skip(1)
}

fn phase(input: &[i8]) -> Vec<i8> {
    let mut out = Vec::with_capacity(input.len());
    for idx in 0..input.len() {
        out.push(calculate_element(input, pattern_for(idx)));
    }
    out
}

impl Exercise for Day {
    fn part1(&self, path: &Path) {
        let mut data = read_input(path);
        for _ in 0..100 {
            data = phase(&data);
        }
        println!("after 100 phases: {}", show(&data[..8]));
    }

    fn part2(&self, path: &Path) {
        let mut data = read_input(path);
        let message_offset: usize = show(&data[..7]).parse().unwrap();

        // I'm not even going to worry about the cost of allocation: we allocate 6.5mb 100 times,
        // which I suspect will be a very small portion of the total runtime.
        // It would be great if this were auto-vectorized, but I worry that since
        // the pattern is an iterator, the compiler might not be smart enough
        // to do that. Right now, I'm not feeling smart enough to do that, so
        // I'm just going to turn this on in release mode and see if I can get
        // a result in reasonable time. In principle, it's just a matter of processing
        // about 650 Mb of data; even single-threaded, that shouldn't be too
        // terrible; I'd expect a runtime of under half an hour.
        data = data
            .iter()
            .copied()
            .cycle()
            .take(data.len() * 10000)
            .collect();

        for idx in 0..100 {
            if idx % 10 == 0 {
                println!("processed {} phases...", idx);
            }
            data = phase(&data);
        }
        println!("after 100 phases: {}", show(&data[..8]));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn min_pattern_for(idx: usize) -> Vec<i8> {
        let want_items = BASE_PATTERN.len() * (idx + 1);
        pattern_for(idx).take(want_items).collect()
    }

    #[test]
    fn pattern_3() {
        let expect = vec![0, 0, 1, 1, 1, 0, 0, 0, -1, -1, -1, 0];
        assert_eq!(min_pattern_for(2), expect);
    }

    #[test]
    fn example_phase_1() {
        let input: Vec<i8> = vec![1, 2, 3, 4, 5, 6, 7, 8];
        let expect: Vec<i8> = vec![4, 8, 2, 2, 6, 1, 5, 8];

        for (idx, e) in expect.into_iter().enumerate() {
            let pattern = pattern_for(idx);
            let got = calculate_element(&input, pattern);
            println!(
                "{}: pattern {:?} => {} ({})",
                idx,
                min_pattern_for(idx),
                got,
                e
            );
            assert_eq!(got, e);
        }
    }

    #[test]
    fn example() {
        let mut data = as_i8("12345678".into());
        let expects: [String; 4] = [
            "48226158".into(),
            "34040438".into(),
            "03415518".into(),
            "01029498".into(),
        ];
        for expect in expects.iter().cloned() {
            data = phase(&data);
            let want = as_i8(expect);
            assert_eq!(data, want);
        }
    }
}
