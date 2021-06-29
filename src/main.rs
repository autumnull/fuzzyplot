use num::complex::Complex;
use std::env;
use std::f64::consts::TAU;
use image::{RgbImage, Rgb};

const GRAPH_BOUNDS: Bounds = Bounds((-2.0, -2.0), (2.0, 2.0));
const THICK_CONST: f64 = (GRAPH_BOUNDS.1.0 - GRAPH_BOUNDS.0.0)*(GRAPH_BOUNDS.1.1 - GRAPH_BOUNDS.0.1);
const ACCURACY_CONST: f64 = (1 << 7) as f64;
const AXIS_CONST: f64 = 0.0001;
const GRID_CONST: f64 = 0.1;
const GRID_SIZE: f64 = 1.0;
const DRAW_AXES: bool = true;
const PROP_EQU: bool = true;

#[derive(Debug)]
struct Bounds (
    (f64, f64),
    (f64, f64),
);

#[allow(unused_variables)]
fn f_l(x: Complex<f64>, y: Complex<f64>, r: f64, t: f64) -> Complex<f64> {
    (x.powi(2) + y.powi(2) - 1.0).powi(3)
}

#[allow(unused_variables)]
fn f_r(x: Complex<f64>, y: Complex<f64>, r: f64, t: f64) -> Complex<f64>{
    x.powi(2)*y.powi(3)
}

fn diff(x: Complex<f64>, y: Complex<f64>) -> f64 {
    let r = (x.re.powi(2) + y.re.powi(2)).sqrt();
    let theta = y.re.atan2(x.re) % TAU;
    let lhs = f_l(x, y, r, theta);
    let rhs = f_r(x, y, r, theta);
    let d = if PROP_EQU {
        ((lhs - rhs)/(lhs + rhs)).norm()
    } else {
        (lhs-rhs).norm()
    };
    d.powi(-2) / ACCURACY_CONST * THICK_CONST
}

fn grid_diff(x: f64, y: f64) -> f64 {
    let dx = (x - GRID_SIZE/2.0).rem_euclid(GRID_SIZE) - GRID_SIZE/2.0;
    let dy = (y - GRID_SIZE/2.0).rem_euclid(GRID_SIZE) - GRID_SIZE/2.0;
    (dx.powi(-2) + dy.powi(-2)) * THICK_CONST * AXIS_CONST * GRID_CONST
}

fn axis_diff(x: f64, y: f64) -> f64 {
    (x.powi(-2) + y.powi(-2)) * THICK_CONST * AXIS_CONST
}

fn parse_args(args: &Vec<String>) -> Result<(&str, u32, u32), &str> {
    let n_args = args.len();
    if n_args == 2 {
        Ok((&args[1], 800u32, 800u32))
    } else if n_args == 3 {
        if let Ok(size) = args[2].parse::<u32>() {
            Ok((&args[1], size, size))
        } else {
            Err("size must be an integer")
        }
    } else if n_args == 4 {
        if let (Ok(width), Ok(height)) = (args[2].parse::<u32>(), args[3].parse::<u32>()) {
            Ok((&args[1], width, height))
        } else {
            Err("dimensions must be integers")
        }
    } else {
        Err("Usage: fuzzyplot <filename> [size [height]]")
    }
}

fn transform(p: (f64,f64), c0: &Bounds, c1: &Bounds) -> (f64, f64) {
    (
        (p.0 - c0.0.0)/(c0.1.0 - c0.0.0)*(c1.1.0 - c1.0.0) + c1.0.0,
        (p.1 - c0.0.1)/(c0.1.1 - c0.0.1)*(c1.1.1 - c1.0.1) + c1.0.1,
    )
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let (filename, width, height) = match parse_args(&args) {
        Ok((f, w, h)) => (f, w, h),
        Err(s) => {
            println!("{}", s);
            return;
        },
    };
    let mut img = RgbImage::new(width, height);

    let graph_corners = GRAPH_BOUNDS;
    let img_corners = Bounds((0.0, height as f64), (width as f64, 0.0));
    
    for x in 0..width {
        for y in 0..height {
            let (graph_x, graph_y) = transform((x as f64, y as f64), &img_corners, &graph_corners);
            let diff: u8 = diff(Complex::new(graph_x, 0.0), Complex::new(graph_y, 0.0)) as u8;
            let mut color = [255, 255-diff, 255-diff];
            if DRAW_AXES {
                let axisness = axis_diff(graph_x, graph_y) + grid_diff(graph_x, graph_y);
                for channel in 0..3 {
                    color[channel] -= (axisness as u8).min(color[channel]);
                }
            };
            img.put_pixel(x, y, Rgb(color));
        }
    }
    img.save(filename).unwrap();
}
