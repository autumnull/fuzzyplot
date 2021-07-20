use anyhow::{anyhow, Context, Result};
use image::{RgbImage, Rgb};
use indicatif::ProgressIterator;
use rug::Complex;
use rug::ops::Pow;
use std::f64::consts::TAU;
use structopt::StructOpt;

// TODO add GRAPH_BOUNDS, GRID_SIZE to CLI
const ACCURACY_CONST: f64 = (1 << 16) as f64;
const AXIS_CONST: f64 = 0.0001;
const GRID_CONST: f64 = 0.1;
const GRID_SIZE: f64 = 1.0;

fn parse_height(input: &str) -> Result<u32, std::num::ParseIntError> {
    if input == "width" {
        Ok(0)
    } else {
        std::str::FromStr::from_str(input)
    }
}

/// outputs a fuzzy-plotted graph image of a given equation
#[derive(StructOpt)]
#[structopt(setting(clap::AppSettings::AllowNegativeNumbers))]
struct Cli {
    /// don't draw axes
    #[structopt(short = "A", long = "axisless")]
    no_axes: bool,
    /// evaluate plain difference, not proportional to magnitude
    #[structopt(short, long = "plain")]
    plain_diff: bool,
    /// equation to plot
    #[structopt()]
    equ_strings: Vec<String>,
    /// filename of the new image. must be .png or .jp(e)g
    #[structopt(short, long, parse(from_os_str), default_value="graph.png")]
    outfile: std::path::PathBuf,
    /// zoom level  
    #[structopt(short, long, default_value="-1")]
    zoom: f64,
    /// image width
    #[structopt(short, long, default_value = "800")]
    width: u32,
    /// image height
    #[structopt(short, long, default_value = "width", parse(try_from_str = parse_height))]
    height: u32,
}

type Expr = mexprp::Expression<Complex>;

#[derive(Debug)]
struct Plot {
    lhs_expr: Expr,
    rhs_expr: Expr,
    color: u8
}

#[derive(Debug)]
struct Params {
    plain_diff: bool,
    thickness: f64,
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

fn diff(p: &Point, plot: &Plot, params: &Params) -> f64 {
    let x = Complex::with_val(53, (p.x, 0.0));
    let y = Complex::with_val(53, (p.y, 0.0));
    let r = Complex::with_val(53, ((p.x.powi(2) + p.y.powi(2)).sqrt(), 0.0));
    let t = Complex::with_val(53, (p.y.atan2(p.x) % TAU, 0.0));
    let mut context = mexprp::Context::new();
    context.set_var("x", x);
    context.set_var("y", y);
    context.set_var("r", r);
    context.set_var("t", t);
    let lhs = plot.lhs_expr.eval_ctx(&context).unwrap().to_vec()[0].clone();
    let rhs = plot.rhs_expr.eval_ctx(&context).unwrap().to_vec()[0].clone();
    let d = if params.plain_diff {
        (lhs-rhs).norm().real().to_f64()
    } else {
        let top = Complex::with_val(53, &lhs-&rhs);
        (top / (lhs + rhs)).norm().real().to_f64()
    };
    d.pow(-2) / ACCURACY_CONST * params.thickness
}

// TODO ditch these and draw the lines using image library
fn axis_diff(p: &Point, params: &Params) -> f64 {
    (p.x.powi(-2) + p.y.powi(-2)) * params.thickness * AXIS_CONST
}

fn grid_diff(p: &Point, params: &Params) -> f64 {
    let dx = (p.x - GRID_SIZE/2.0).rem_euclid(GRID_SIZE) - GRID_SIZE/2.0;
    let dy = (p.y - GRID_SIZE/2.0).rem_euclid(GRID_SIZE) - GRID_SIZE/2.0;
    (dx.powi(-2) + dy.powi(-2)) * params.thickness * AXIS_CONST * GRID_CONST
}

fn make_context() -> mexprp::Context<Complex> {
    // set to only return one sqrt() result
    let mut context = mexprp::Context::new();
    context.cfg = mexprp::Config {
        implicit_multiplication: true,
        precision: 53,
        sqrt_both: false,
    };
    // initialise variables to 0.
    let init = Complex::with_val(53, (0.0, 0.0));
    context.set_var("x", init.clone());
    context.set_var("y", init.clone());
    context.set_var("r", init.clone());
    context.set_var("t", init);
    context
}

fn main() -> Result<()> {
    let args = Cli::from_args();
    
    // check valid image format before proceeding
    image::ImageFormat::from_path(args.outfile.as_path())
    .with_context(|| format!("Unrecognized file extension for image"))?;
    
    if args.equ_strings.len() > 3 {
        return Err(anyhow!("Maximum of 3 equations allowed"));
    };
    
    let (width, height) = if args.height == 0 {
        (args.width, args.width)
    } else {
        (args.width, args.height)
    };
    
    let img_rect = Rect{
        x: 0.0,
        y: 0.0,
        w: width as f64,
        h: height as f64,
    };
    let graph_rect_r = 2.0.pow(-args.zoom);
    let graph_rect = Rect{
        x: -graph_rect_r,
        y: -graph_rect_r,
        w: graph_rect_r * 2.0,
        h: graph_rect_r * 2.0,
    };
    let params = Params {
        plain_diff: args.plain_diff,
        thickness: graph_rect.w * graph_rect.h,
    };
    
    let context = make_context();
    
    let mut plots: Vec<Plot> = Vec::new();
    for (i, equ) in args.equ_strings.iter().enumerate() {
        // separate the left and right sides of the equation
        let split_equ = equ.split("=").collect::<Vec<&str>>();
        let (lhs, rhs) = if split_equ.len() == 2 {
            (split_equ[0], split_equ[1])
        } else {
            return Err(anyhow!("Equation should have 1 '=' sign"));
        };
        // TODO handle errors more nicely
        let lhs_expr = mexprp::Expression::parse_ctx(lhs, context.clone())
            .unwrap();
        let rhs_expr = mexprp::Expression::parse_ctx(rhs, context.clone())
            .unwrap();
        // set color to red if only 1 equation, otherwise CMY
        let color = if args.equ_strings.len() == 1 {6} else {1 << i as u8};
        let plot = Plot{lhs_expr, rhs_expr, color};
        plots.push(plot);
    }
    
    let mut img = RgbImage::new(width, height);
    
    println!("generating image...");
    for (x, y, pixel) in img.enumerate_pixels_mut().progress() {
        let img_point = Point{x: x as f64, y: (height-1 - y) as f64};
        let graph_point = img_rect.map_point(&img_point, &graph_rect);
        
        *pixel = Rgb([255, 255, 255]);
        if !args.no_axes {
            let axisness = axis_diff(&graph_point, &params)
            + grid_diff(&graph_point, &params);
            for channel in 0..3 {
                pixel[channel] -= (axisness as u8).min(pixel[channel]);
            }
        };
        
        for plot in plots.iter() {
            let diff = diff(
                &graph_point,
                &plot,
                &params) as u8;
            for channel in 0..3 {
                if (plot.color >> channel) & 0b1 == 0b1 {
                    pixel[channel] -= diff.min(pixel[channel]);
                }
            }
        }
    }
    
    img.save(&args.outfile)
        .with_context(
            || format!("Couldn't save file '{}'", args.outfile.display())
        )?;
    println!("done!");
    Ok(())
}
