use structopt::StructOpt;
use num::complex::Complex;
use std::f64::consts::TAU;
use image::{RgbImage, Rgb};
use anyhow::{Context, Result};

// TODO add GRAPH_BOUNDS, GRID_SIZE to CLI
const GRAPH_BOUNDS: Rect = Rect{x: -2.0, y: -2.0, w: 4.0, h: 4.0};
const THICK_CONST: f64 = GRAPH_BOUNDS.w * GRAPH_BOUNDS.h;
const ACCURACY_CONST: f64 = (1 << 7) as f64;
const AXIS_CONST: f64 = 0.0001;
const GRID_CONST: f64 = 0.1;
const GRID_SIZE: f64 = 1.0;

/// outputs a fuzzy-plotted graph image of a given equation
#[derive(StructOpt)]
struct Cli {
    /// don't draw axes
    #[structopt(short = "A", long = "axisless")]
    no_axes: bool,
    /// evaluate plain difference, not proportional to magnitude
    #[structopt(short, long = "plain")]
    plain_diff: bool,
    /// filename of the new image. must be .png or .jp(e)g
    #[structopt(short, long, parse(from_os_str), default_value="graph.png")]
    outfile: std::path::PathBuf,
    /// image width
    #[structopt(default_value = "800")]
    width: u32,
    /// image height
    #[structopt(default_value = "width", parse(try_from_str = parse_height))]
    height: u32,
}

fn parse_height(input: &str) -> Result<u32, std::num::ParseIntError> {
    if input == "width" {
        Ok(0)
    } else {
        std::str::FromStr::from_str(input)
    }
}

#[derive(Debug)]
struct Point {
    x: f64,
    y: f64,
}

#[derive(Debug)]
struct Rect {
    x: f64,
    y: f64,
    w: f64,
    h: f64,
}

impl Rect {
    fn map_point(&self, p: &Point, c1: &Rect) -> Point {
        Point {
            x: (p.x - &self.x) / &self.w * c1.w + c1.x,
            y: (p.y - &self.y) / &self.h * c1.h + c1.y,
        }
    }
}

// TODO make into parsed equations
#[allow(unused_variables)]
fn f_l(x: Complex<f64>, y: Complex<f64>, r: f64, t: f64) -> Complex<f64> {
    (x.powi(2) + y.powi(2) - 1.0).powi(3)
}

#[allow(unused_variables)]
fn f_r(x: Complex<f64>, y: Complex<f64>, r: f64, t: f64) -> Complex<f64>{
    x.powi(2)*y.powi(3)
}

fn diff(p: &Point, plain_diff: bool) -> f64 {
    let x = Complex::new(p.x, 0.0);
    let y = Complex::new(p.y, 0.0);
    let r = (x.re.powi(2) + y.re.powi(2)).sqrt();
    let theta = y.re.atan2(x.re) % TAU;
    let lhs = f_l(x, y, r, theta);
    let rhs = f_r(x, y, r, theta);
    let d = if plain_diff {
        (lhs-rhs).norm()
    } else {
        ((lhs - rhs)/(lhs + rhs)).norm()
    };
    d.powi(-2) / ACCURACY_CONST * THICK_CONST
}

fn grid_diff(p: &Point) -> f64 {
    let dx = (p.x - GRID_SIZE/2.0).rem_euclid(GRID_SIZE) - GRID_SIZE/2.0;
    let dy = (p.y - GRID_SIZE/2.0).rem_euclid(GRID_SIZE) - GRID_SIZE/2.0;
    (dx.powi(-2) + dy.powi(-2)) * THICK_CONST * AXIS_CONST * GRID_CONST
}

fn axis_diff(p: &Point) -> f64 {
    (p.x.powi(-2) + p.y.powi(-2)) * THICK_CONST * AXIS_CONST
}

fn main() -> Result<()> {
    let args = Cli::from_args();
    let (width, height) = if args.height == 0 {
            (args.width, args.width)
        } else {
            (args.width, args.height)
        };
    
    // check valid image format before calculation  
    image::ImageFormat::from_path(args.outfile.as_path())
        .with_context(|| format!("Unrecognized file extension for image"))?;
    
    let mut img = RgbImage::new(width, height);
    
    let graph_rect = GRAPH_BOUNDS;
    let img_rect = Rect{
        x: 0.0,
        y: 0.0,
        w: width as f64,
        h: height as f64,
    };
    
    println!("generating image...");
    for x in 0..width {
        for y in 0..height {
            let img_point = Point{x: x as f64, y: y as f64};
            let graph_point = img_rect.map_point(&img_point, &graph_rect);
            let diff = diff(&graph_point, args.plain_diff) as u8;
            // TODO: make color immutable ?
            let mut color = Rgb([255, 255-diff, 255-diff]);
            if !args.no_axes {
                let axisness = axis_diff(&graph_point) + grid_diff(&graph_point);
                for channel in 0..3 {
                    color[channel] -= (axisness as u8).min(color[channel]);
                }
            };
            img.put_pixel(x, height-1 - y, color);
        }
    }
    
    img.save(&args.outfile)
        .with_context(
            || format!("Couldn't save file '{}'", args.outfile.display())
        )?;
    println!("done!");
    Ok(())
}
