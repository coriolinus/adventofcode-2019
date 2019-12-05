use crate::Exercise;
use std::fmt;
use std::path::Path;

pub struct Day;

pub const LOW: u32 = 234_208;
pub const HIGH: u32 = 765_869;
pub const LEN: usize = 6;

impl Exercise for Day {
    fn part1(&self, _: &Path) {
        println!(
            "possible pw count: {}",
            Password::new(LOW, HIGH).iter().count()
        );
    }

    fn part2(&self, _: &Path) {
        println!(
            "possible pw count (no 3-runs): {}",
            Password::new(LOW, HIGH)
                .iter()
                .filter(|pw| no_3_runs(*pw))
                .count()
        );
    }
}

// the name here isn't perfectly accurate: 3 runs are legal, as long as there
// is at least one run of exactly 2
fn no_3_runs(v: u32) -> bool {
    let digits = Password::digits(v);
    #[cfg(test)]
    dbg!(digits);
    for idx in 1..LEN {
        #[cfg(test)]
        {
            dbg!(idx);
            dbg!(digits[idx]);
            dbg!(digits[idx - 1]);
            if idx >= 2 {
                dbg!(digits[idx - 2]);
            }
            if idx < LEN - 1 {
                dbg!(digits[idx + 1]);
            }
        }
        if digits[idx] == digits[idx - 1] {
            // this is a run of at least 2, but is it exactly 2?
            let mut exact2 = true;
            if idx >= 2 {
                exact2 &= digits[idx - 2] != digits[idx];
            }
            if idx < LEN - 1 {
                exact2 &= digits[idx + 1] != digits[idx];
            }
            if exact2 {
                return true;
            }
        }
    }
    false
}

// this internal type stores the digits in reverse order: the 0 idx has the low
// digit
pub type Digits = [u8; LEN];

pub struct Password {
    high: u32,
    digits: Digits,
}

impl Password {
    pub fn new(start: u32, high: u32) -> Password {
        let mut p = Password {
            high,
            digits: Self::digits(start),
        };
        if !Self::is_legal_digits(p.digits) {
            // we can't just use next, because it assumes that the digits are already valid
            // so let's run a quick full scan and ensure we have the lowest initial value
            let mut pinidx = None;
            for idx in (1..LEN).rev() {
                if pinidx.is_none() && p.digits[idx - 1] < p.digits[idx] {
                    pinidx = Some(idx);
                }
                if let Some(pidx) = pinidx {
                    p.digits[idx - 1] = p.digits[pidx];
                }
            }
        }
        p
    }

    pub fn digits(mut n: u32) -> Digits {
        let mut digits = [0; LEN];
        for digit in digits.iter_mut() {
            *digit = (n % 10) as u8;
            n /= 10;
        }
        debug_assert_eq!(n, 0); // otherwise, input too large
        digits
    }

    fn is_legal_digits(digits: Digits) -> bool {
        // because digits are stored backwards, each digit must be
        // greater than or equal to its successor
        digits.windows(2).all(|w| w[0] >= w[1])
            && digits.windows(2).any(|w| w[0] == w[1])
            && digits.iter().all(|d| *d < 10)
    }

    pub fn is_legal(n: u32) -> bool {
        Self::is_legal_digits(Self::digits(n))
    }

    pub fn value(&self) -> u32 {
        self.digits
            .iter()
            .enumerate()
            // idx (usize) will always fit into u32, because it will never be
            // greater than LEN, so we can skip any bounds checking and use as.
            // d is a u8, so always fits into u32.
            .map(|(idx, d)| 10_u32.pow(idx as u32) * *d as u32)
            .sum()
    }

    /// find the next valid password according to the rules
    ///
    /// updates the internal state
    pub fn next(&mut self) {
        // start with a simple digit-wise incr
        for idx in 0..LEN {
            self.digits[idx] += 1;
            if self.digits[idx] > 9 {
                // handle rollover
                for next_idx in (idx + 1)..LEN {
                    if self.digits[next_idx] < 9 {
                        self.digits[idx] = self.digits[next_idx] + 1;
                        break;
                    }
                }
                // now test again: if we're still over 9, we have to roll over,
                // because it was nines to the end
                if self.digits[idx] > 9 {
                    self.digits[idx] = 0;
                }
            } else {
                break;
            }
        }

        // now that we've incremented the value, we still have to ensure that
        // the second property--at least one repeated digit pair exists--is
        // preserved.
        if !Self::is_legal_digits(self.digits) {
            // given that we always want the lowest subsequent value which has
            // a repeated digit pair, we only have to consider the final two
            // digits: we know that the second digit isn't 9, so we can just
            // set both low-order digits to the next value
            debug_assert_ne!(self.digits[1], 9);
            self.digits[1] += 1;
            self.digits[0] = self.digits[1];
        }

        #[cfg(test)]
        dbg!(self.value());
        debug_assert!(Self::is_legal_digits(self.digits));
    }

    pub fn iter(&mut self) -> Iter<'_> {
        Iter { pw: self }
    }
}

pub struct Iter<'a> {
    pw: &'a mut Password,
}

impl<'a> Iterator for Iter<'a> {
    type Item = u32;
    fn next(&mut self) -> Option<Self::Item> {
        let v = self.pw.value();
        // overflow always ends iteration
        if v > self.pw.high || v == 0 {
            None
        } else {
            self.pw.next();
            Some(v)
        }
    }
}

impl std::iter::FusedIterator for Iter<'_> {}

impl fmt::Display for Password {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for d in self.digits.iter().rev() {
            write!(f, "{}", d)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_legal_1s() {
        assert!(Password::is_legal(111111));
    }

    #[test]
    fn test_legal_four_1s() {
        assert!(Password::is_legal(001111));
    }

    #[test]
    fn test_illegal_decreasing() {
        assert!(!Password::is_legal(223450));
    }

    #[test]
    fn test_illegal_no_double() {
        assert!(!Password::is_legal(123789));
    }

    #[test]
    #[should_panic]
    #[cfg(debug_assertions)]
    fn test_max_range() {
        Password::is_legal(1234567);
    }

    #[test]
    fn test_values() {
        for value in [111111, 001111, 223457].iter().cloned() {
            assert_eq!(Password::new(value, 0).value(), value);
        }
    }

    fn next_case(have: u32, expect_next: u32) {
        let mut pw = Password::new(have, 0);
        pw.next();
        assert_eq!(pw.value(), expect_next);
    }

    #[test]
    fn text_next_ones() {
        next_case(111111, 111112);
    }

    #[test]
    fn text_next_nines() {
        next_case(999999, 000000);
    }

    #[test]
    fn test_next_rollover() {
        next_case(899999, 999999);
    }

    #[test]
    fn test_next_459() {
        next_case(111459, 111466);
    }

    #[test]
    fn text_next_499() {
        next_case(111499, 111555);
    }

    #[test]
    fn test_next_999() {
        next_case(123999, 124444);
    }

    #[test]
    fn test_next_123455() {
        next_case(123455, 123466);
    }

    #[test]
    fn test_next_123777() {
        next_case(123777, 123778);
    }

    #[test]
    fn test_next_123568() {
        next_case(123568, 123577);
    }

    #[test]
    fn text_next_123489() {
        next_case(123489, 123499);
    }

    #[test]
    fn text_exhaustive() {
        let mut pw = Password::new(000001, 999999);
        for value in pw.iter() {
            assert!(Password::is_legal(value));
        }
        // reset trailing digit to enable more counting
        pw.digits[0] = 1;
        assert!(pw.iter().take(2).count() > 0);
    }

    #[test]
    fn test_no_3_runs_examples() {
        assert!(no_3_runs(&112233));
        assert!(!no_3_runs(&123444));
        assert!(no_3_runs(&111122));
    }
}
