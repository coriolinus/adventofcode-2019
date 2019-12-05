use std::ops::Sub;
use std::str::FromStr;

pub fn follow(traces: &[Trace]) -> Vec<Line> {
    let mut cursor = Point::new(0, 0);
    let mut out = Vec::with_capacity(traces.len());
    for trace in traces {
        let prev = cursor;
        use Direction::*;
        let (val, mul) = match trace.direction {
            Right => (&mut cursor.x, 1),
            Left => (&mut cursor.x, -1),
            Up => (&mut cursor.y, 1),
            Down => (&mut cursor.y, -1),
        };
        *val += trace.distance * mul;
        out.push(Line::new(prev, cursor));
    }
    out
}

// https://stackoverflow.com/a/1968345/504550
pub fn intersect(a: &Line, b: &Line) -> Option<Point> {
    let p0 = a.from;
    let p1 = a.to;
    let p2 = b.from;
    let p3 = b.to;

    let s1_x = (p1.x - p0.x) as f32;
    let s1_y = (p1.y - p0.y) as f32;
    let s2_x = (p3.x - p2.x) as f32;
    let s2_y = (p3.y - p2.y) as f32;

    let s =
        (-s1_y * (p0.x - p2.x) as f32 + s1_x * (p0.y - p2.y) as f32) / (-s2_x * s1_y + s1_x * s2_y);
    let t =
        (s2_x * (p0.y - p2.y) as f32 - s2_y * (p0.x - p2.x) as f32) / (-s2_x * s1_y + s1_x * s2_y);

    if s >= 0.0 && s <= 1.0 && t >= 0.0 && t <= 1.0 {
        // round the results so errors line up nicely
        Some(Point::new(
            p0.x + (t * s1_x).round() as i32,
            p0.y + (t * s1_y).round() as i32,
        ))
    } else {
        None
    }
}

// bentley-ottman algorithm: http://geomalgorithms.com/a09-_intersect-3.html
// TODO: maybe finish later?
// pub fn intersections(a: &[Line], b: &[Line]) -> Vec<Point> {
//     let mut eq = Vec::with_capacity(a.len() + b.len());
//     eq.extend(a.iter().map(|l| Event {
//         line: *l,
//         color: Color::Red,
//     }));
//     eq.extend(b.iter().map(|l| Event {
//         line: *l,
//         color: Color::Black,
//     }));
//     eq.sort_unstable_by_key(|e| e.line);
//     unimplemented!()
// }

pub fn intersections_naive(ap: &[Line], bp: &[Line]) -> Vec<Point> {
    let mut isects = Vec::new();
    for a in ap {
        for b in bp {
            if let Some(isect) = intersect(a, b) {
                isects.push(isect);
            }
        }
    }
    isects
}

#[derive(Debug, Clone, Copy)]
pub enum Direction {
    Right,
    Left,
    Up,
    Down,
}

#[derive(Debug, Clone, Copy)]
pub struct Trace {
    direction: Direction,
    distance: i32,
}

impl FromStr for Trace {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.len() < 2 {
            return Err("len < 2".into());
        }
        let direction = match s.as_bytes()[0] {
            b'R' | b'r' => Direction::Right,
            b'L' | b'l' => Direction::Left,
            b'U' | b'u' => Direction::Up,
            b'D' | b'd' => Direction::Down,
            unknown => return Err(format!("unknown direction: {}", unknown as char)),
        };
        let distance = s[1..]
            .parse()
            .map_err(|e: std::num::ParseIntError| e.to_string())?;
        Ok(Trace {
            direction,
            distance,
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Point {
    pub x: i32,
    pub y: i32,
}

impl Point {
    pub fn new(x: i32, y: i32) -> Point {
        Point { x, y }
    }

    // on my machine, passing self by copy and reference are equally sized,
    // and passing by copy breaks the cleanest usage of this function in Iterator::map,
    // so I'm going to retain the reference behavior. I expect the compiler to 
    // inline this function anyway.
    #[allow(clippy::trivially_copy_pass_by_ref)]
    pub fn manhattan(&self) -> i32 {
        self.x.abs() + self.y.abs()
    }
}

impl Sub for Point {
    type Output = Point;

    fn sub(self, other: Point) -> Point {
        Point {
            x: self.x - other.x,
            y: self.y - other.y,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Line {
    pub from: Point,
    pub to: Point,
}

impl Line {
    pub fn new(from: Point, to: Point) -> Line {
        Line { from, to }
    }

    pub fn manhattan_len(&self) -> i32 {
        (self.to - self.from).manhattan()
    }
}
