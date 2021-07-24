use rug::Complex;

pub mod cli;
pub mod diff;
pub mod setup;

// TODO add GRID_SIZE to CLI
const ACCURACY_CONST: f64 = (1 << 16) as f64;
const AXIS_CONST: f64 = 0.0001;
const GRID_SIZE: f64 = 1.0;

type Expr = mexprp::Expression<Complex>;

#[derive(Debug)]
pub struct Plot {
    pub lhs_expr: Expr,
    pub rhs_expr: Expr,
    pub color: u8
}

#[derive(Debug)]
pub struct Params {
    pub plain_diff: bool,
    pub thickness: f64,
}

#[derive(Debug)]
pub struct Point {
    pub x: f64,
    pub y: f64,
}

#[derive(Debug)]
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
