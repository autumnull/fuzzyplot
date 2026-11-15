use crate::fuzzyplot::{
    cli::Cli,
    expr::{Compiler, Mode},
    Plot, Point, Rect,
};
use anyhow::{anyhow, Context, Result};
use num_complex::Complex64;
use std::f64::consts::TAU;

#[derive(Debug, Clone, Copy)]
pub struct Params {
    pub size: (u32, u32),
    pub t_range: (i32, i32),
    pub img_rect: Rect,
    pub graph_rect: Rect,
    pub graph_pixel_r: f64,
    pub sharpness: f64,
    pub mode: Mode,
    pub poles: bool,
    pub draw_axes: bool,
    pub grid_size: f64,
}

impl Params {
    pub fn from_args(args: &Cli) -> Result<Params> {
        // check valid file extension before proceeding
        image::ImageFormat::from_path(args.outfile.as_path())
            .with_context(|| "Unrecognized file extension for image")?;

        let size = if args.dimensions.len() == 2 {
            if args.dimensions[0] < 1 || args.dimensions[1] < 1 {
                return Err(anyhow!(
                    "Image dimensions must be greater than zero"
                ));
            } else {
                (args.dimensions[0], args.dimensions[1])
            }
        } else {
            (800, 800)
        };
        let focus = if args.focus.len() == 2 {
            (args.focus[0], args.focus[1])
        } else {
            (0.0, 0.0)
        };
        let t_range = if args.t_range.len() == 2 {
            (args.t_range[0], args.t_range[1] + 1)
        } else {
            (0, 1)
        };

        let img_rect = Rect {
            x: 0.0,
            y: 0.0,
            w: size.0 as f64,
            h: size.1 as f64,
        };
        let (x_ratio, y_ratio) = if size.0 < size.1 {
            (1.0, (size.1 / size.0) as f64)
        } else {
            ((size.0 / size.1) as f64, 1.0)
        };
        let graph_rect_r = 2.0_f64.powf(-args.zoom);
        let graph_rect = Rect {
            x: focus.0 - graph_rect_r * x_ratio,
            y: focus.1 - graph_rect_r * y_ratio,
            w: graph_rect_r * 2.0 * x_ratio,
            h: graph_rect_r * 2.0 * y_ratio,
        };
        let graph_pixel_r = graph_rect.w / img_rect.w / 2.0;
        let mode = if args.quotient {
            Mode::Quotient
        } else if args.plain_diff {
            Mode::Plain
        } else {
            Mode::Distance
        };
        let sharpness = if mode == Mode::Distance {
            2.0_f64.powf(args.sharpness)
        } else {
            2.0_f64.powf(args.sharpness + 8.0)
        };

        Ok(Params {
            size,
            t_range,
            img_rect,
            graph_rect,
            graph_pixel_r,
            sharpness,
            mode,
            poles: !args.no_poles,
            draw_axes: !args.no_axes,
            grid_size: args.grid_size,
        })
    }
}

pub fn make_contexts(
    p: &Point,
    t_range: (i32, i32),
    contexts: &mut Vec<[Complex64; 4]>,
) {
    let x = Complex64::new(p.x, 0.0);
    let y = Complex64::new(p.y, 0.0);
    let r = (p.x.powi(2) + p.y.powi(2)).sqrt();
    let t = p.y.atan2(p.x);

    contexts.clear();
    for i in t_range.0..t_range.1 {
        // accomodate negative r for opposite angle
        contexts.push([
            x,
            y,
            Complex64::new(r, 0.0),
            Complex64::new(t + TAU * (i as f64), 0.0),
        ]);
        // if t is negative, turn in positive direction, and vice versa
        let half_turn = t.signum() * -TAU / 2.0;
        contexts.push([
            x,
            y,
            Complex64::new(-r, -0.0),
            Complex64::new(t + half_turn + TAU * (i as f64), 0.0),
        ]);
    }
}

pub fn context_count(plot: &Plot, contexts: usize) -> usize {
    let needed = if plot.program.uses_t {
        contexts
    } else if plot.program.uses_r {
        2
    } else {
        1
    };
    needed.min(contexts)
}

fn hue(degrees: f64) -> [f64; 3] {
    let h = degrees.rem_euclid(360.0) / 60.0;
    let x = 1.0 - (h % 2.0 - 1.0).abs();
    match h as u32 {
        0 => [1.0, x, 0.0],
        1 => [x, 1.0, 0.0],
        2 => [0.0, 1.0, x],
        3 => [0.0, x, 1.0],
        4 => [x, 0.0, 1.0],
        _ => [1.0, 0.0, x],
    }
}

pub fn make_plots(
    equations: &Vec<String>,
    params: &Params,
) -> Result<Vec<Plot>> {
    let mut plots: Vec<Plot> = Vec::new();
    for (i, equ) in equations.iter().enumerate() {
        let mut compiler =
            Compiler::new(params.sharpness, params.mode, params.poles);
        let (zeros, poles) = compiler
            .expression(equ)
            .context("Failed to parse equation")?;

        let angle = 180.0 * i as f64 / equations.len() as f64;
        let plot = Plot {
            program: compiler.finish(),
            zeros,
            poles,
            color: hue(angle),
        };
        plots.push(plot);
    }
    Ok(plots)
}
