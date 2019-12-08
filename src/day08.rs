use crate::{parse, Exercise};
use std::path::Path;
use std::str::FromStr;

pub struct Day;

pub const WIDTH: usize = 25;
pub const HEIGHT: usize = 6;

impl Exercise for Day {
    fn part1(&self, path: &Path) {
        let images: Vec<SpaceImageFormat> = parse::<SpaceImageFormat>(path).unwrap().collect();
        println!("qty images: {}", images.len());
        for (iidx, image) in images.iter().enumerate() {
            let (_, fewest_0_idx) = image
                .layers
                .iter()
                .enumerate()
                .map(|(lidx, layer)| {
                    let zero_count: usize = layer
                        .iter()
                        .map(|row| row.iter().filter(|v| **v == 0).count())
                        .sum();
                    (zero_count, lidx)
                })
                .min()
                .unwrap();
            let ones: usize = image.layers[fewest_0_idx]
                .iter()
                .map(|row| row.iter().filter(|v| **v == 1).count())
                .sum();
            let twos: usize = image.layers[fewest_0_idx]
                .iter()
                .map(|row| row.iter().filter(|v| **v == 2).count())
                .sum();
            println!(
                "image {}: layer {}: n1 * n2 = {}",
                iidx,
                fewest_0_idx,
                ones * twos
            );
        }
    }

    fn part2(&self, path: &Path) {
        let images: Vec<SpaceImageFormat> = parse::<SpaceImageFormat>(path).unwrap().collect();
        for (iidx, image) in images.iter().enumerate() {
            let mut render = [[0_u8; WIDTH]; HEIGHT];
            for row in 0..HEIGHT {
                for col in 0..WIDTH {
                    for layer in 0..(image.layers.len()) {
                        if image.layers[layer][row][col] != 2 {
                            render[row][col] = image.layers[layer][row][col];
                            break;
                        }
                    }
                }
            }
            println!("image {}:", iidx);
            for row in &render {
                for col in row {
                    print!(
                        "{}",
                        match col {
                            0 => " ",
                            1 => "*",
                            _ => "?",
                        }
                    );
                }
                println!();
            }
        }
    }
}

pub type Layer = [[u8; WIDTH]; HEIGHT];
pub struct SpaceImageFormat {
    layers: Vec<Layer>,
}

impl FromStr for SpaceImageFormat {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.trim();
        if s.len() == 0 {
            return Err("no data in image");
        }
        if s.len() % (WIDTH * HEIGHT) != 0 {
            return Err("bad input length");
        }
        let nlayers = s.len() / (WIDTH * HEIGHT);
        let mut layers: Vec<Layer> = vec![Layer::default(); nlayers];

        let b = s.as_bytes();

        for layer in 0..nlayers {
            for row in 0..HEIGHT {
                for col in 0..WIDTH {
                    let idx = col + (row * WIDTH) + (layer * WIDTH * HEIGHT);
                    // dbg!(layer, row, col, idx);
                    layers[layer][row][col] = (b[idx] as char)
                        .to_digit(10)
                        .ok_or("could no parse digit")?
                        as u8;
                }
            }
        }

        Ok(SpaceImageFormat { layers })
    }
}
