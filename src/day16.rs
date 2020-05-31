use crate::Exercise;
use std::path::Path;

pub struct Day;

fn read_input(path: &Path) -> Vec<i8> {
    as_i8(std::fs::read_to_string(path).unwrap())
}

// this is silly and too-low-level: it makes a lot of assumptions about its input
// it's fast, though
fn as_i8(data: String) -> Vec<i8> {
    let mut data = data.into_bytes();
    data.truncate(
        data.len()
            - data
                .iter()
                .rev()
                .take_while(|c| c.is_ascii_whitespace())
                .count(),
    );
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

fn calculate_element<P>(input: &[i8], pattern: P) -> i8
where
    P: Iterator<Item = i8>,
{
    (input
        .iter()
        .zip(pattern)
        .map(|(i, p)| (*i * p) as i64)
        .sum::<i64>()
        .abs()
        % 10)
        as i8
}

const BASE_PATTERN: [i8; 4] = [0, 1, 0, -1];

fn pattern_for(idx: usize) -> impl Iterator<Item = i8> + Clone {
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

fn first_n(data: &[i8], n: usize) -> String {
    let mut out = String::with_capacity(n);
    for d in data.iter().take(n) {
        out.push_str(&d.to_string());
    }
    out
}

impl Exercise for Day {
    fn part1(&self, path: &Path) {
        let mut data = read_input(path);
        for _ in 0..100 {
            data = phase(&data);
        }
        println!("after 100 phases: {}", first_n(&data, 8));
    }

    fn part2(&self, _path: &Path) {
        unimplemented!()
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
