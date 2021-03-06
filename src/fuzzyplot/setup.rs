use crate::fuzzyplot::{cli::Cli, Plot, Point, Rect};
use anyhow::{anyhow, Context, Result};
use failure::Fail;
use rug::Complex;
use std::f64::consts::TAU;

#[derive(Debug, Clone, Copy)]
pub struct Params {
    pub size: (u32, u32),
    pub t_range: (i32, i32),
    pub img_rect: Rect,
    pub graph_rect: Rect,
    pub graph_pixel_r: f64,
    pub sharpness: f64,
    pub plain_diff: bool,
    pub draw_axes: bool,
    pub grid_size: f64,
}

impl Params {
    pub fn from_args(args: &Cli) -> Result<Params> {
        // check valid file extension before proceeding
        image::ImageFormat::from_path(args.outfile.as_path())
            .with_context(|| "Unrecognized file extension for image")?;

        // we do a lil baby test to check that the equation is valid
        let plots_test = make_plots(&args.equations)?;
        let (lhs_result, rhs_result) =
            (plots_test[0].lhs_expr.eval(), plots_test[0].rhs_expr.eval());
        if let Err(e) = lhs_result.and(rhs_result) {
            return Err(e.compat()).context("Failed to parse equation");
        }

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
        let center = if args.focus.len() == 2 {
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
            x: center.0 - graph_rect_r * x_ratio,
            y: center.1 - graph_rect_r * y_ratio,
            w: graph_rect_r * 2.0 * x_ratio,
            h: graph_rect_r * 2.0 * y_ratio,
        };
        let graph_pixel_r = graph_rect.w / img_rect.w / 2.0;
        let sharpness = 2.0_f64.powf(args.sharpness + 8.0);

        Ok(Params {
            size,
            t_range,
            img_rect,
            graph_rect,
            graph_pixel_r,
            sharpness,
            plain_diff: args.plain_diff,
            draw_axes: !args.no_axes,
            grid_size: args.grid_size,
        })
    }
}

pub fn make_contexts(
    p: Point,
    t_range: (i32, i32),
) -> Vec<mexprp::Context<Complex>> {
    let x = Complex::with_val(53, (p.x, 0.0));
    let y = Complex::with_val(53, (p.y, 0.0));
    let r = Complex::with_val(53, ((p.x.powi(2) + p.y.powi(2)).sqrt(), 0.0));
    let t = Complex::with_val(53, (p.y.atan2(p.x), 0.0));

    let mut contexts: Vec<mexprp::Context<Complex>> = Vec::new();
    for i in t_range.0..t_range.1 {
        // accomodate negative r for opposite angle
        for r_coeff in [1, -1] {
            let mut context = mexprp::Context::new();
            context.set_var("tau", Complex::with_val(53, (TAU, 0.0))); // yes
            context.set_var("x", x.clone());
            context.set_var("y", y.clone());
            context.set_var("r", r_coeff * r.clone());
            if r_coeff == 1 {
                context.set_var("t", t.clone() + TAU * (i as f64));
            } else {
                // if t is negative, turn in positive direction, and vice versa
                let half_turn = t.real().to_f64().signum() * -TAU / 2.0;
                context.set_var("t", t.clone() + half_turn + TAU * (i as f64));
            }
            contexts.push(context);
        }
    }
    contexts
}

pub fn make_plots(equations: &Vec<String>) -> Result<Vec<Plot>> {
    let init_context =
        make_contexts(Point { x: 0.0, y: 0.0 }, (0, 1))[0].clone();
    let mut plots: Vec<Plot> = Vec::new();
    for (i, equ) in equations.iter().enumerate() {
        // separate the left and right sides of the equation
        let split_equ = equ.split("=").collect::<Vec<&str>>();
        let (lhs, rhs) = if split_equ.len() == 2 {
            (split_equ[0], split_equ[1])
        } else {
            return Err(anyhow!("Equations should have 1 '=' sign"));
        };

        let lhs_expr =
            match mexprp::Expression::parse_ctx(lhs, init_context.clone()) {
                Ok(expr) => expr,
                Err(e) => {
                    return Err(e.compat()).context("Failed to parse equation")
                }
            };
        let rhs_expr =
            match mexprp::Expression::parse_ctx(rhs, init_context.clone()) {
                Ok(expr) => expr,
                Err(e) => {
                    return Err(e.compat()).context("Failed to parse equation")
                }
            };

        // set color to red if only 1 equation, otherwise CMY
        let color = if equations.len() == 1 {
            0b001
        } else {
            0b111 ^ (1 << i)
        };
        let plot = Plot {
            lhs_expr,
            rhs_expr,
            color,
        };
        plots.push(plot);
    }
    Ok(plots)
}
