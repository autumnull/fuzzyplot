use std::ops::Range;

pub mod cli;
pub mod diff;
pub mod expr;
pub mod setup;

#[derive(Debug)]
pub struct Plot {
    pub program: expr::Program,
    pub zeros: Range<usize>,
    pub poles: Option<Range<usize>>,
    pub zero_color: [f64; 3],
    pub pole_color: [f64; 3],
}

#[derive(Debug)]
pub struct Point {
    pub x: f64,
    pub y: f64,
}

#[derive(Debug, Clone, Copy)]
pub struct Rect {
    pub x: f64,
    pub y: f64,
    pub w: f64,
    pub h: f64,
}

impl Rect {
    pub fn map_point(&self, p: &Point, c1: &Rect) -> Point {
        Point {
            x: (p.x - &self.x) / &self.w * c1.w + c1.x,
            y: (p.y - &self.y) / &self.h * c1.h + c1.y,
        }
    }
}
