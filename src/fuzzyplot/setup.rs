use crate::fuzzyplot::{cli::Cli, Plot, Point, Rect};
use anyhow::{anyhow, Context, Result};
use rug::Complex;
use std::f64::consts::TAU;

#[derive(Debug)]
pub struct Params {
    pub size: (u32, u32),
    pub t_range: (i32, i32),
    pub img_rect: Rect,
    pub graph_rect: Rect,
    pub graph_pixel_r: f64,
    pub thickness: f64,
    pub plain_diff: bool,
}

impl Params {
    pub fn from_args(args: &Cli) -> Result<Params> {
        // check valid file extension before proceeding
        image::ImageFormat::from_path(args.outfile.as_path()).with_context(
            || format!("Unrecognized file extension for image"),
        )?;

        let size = if args.size.len() == 2 {
            (args.size[0], args.size[1])
        } else {
            (800, 800)
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
        let aspect_ratio = (size.0 / size.1) as f64;
        let graph_rect_r = 2.0_f64.powf(-args.zoom);
        let graph_rect = Rect {
            x: -graph_rect_r * aspect_ratio,
            y: -graph_rect_r,
            w: graph_rect_r * 2.0 * aspect_ratio,
            h: graph_rect_r * 2.0,
        };
        let graph_pixel_r = graph_rect.w / img_rect.w / 2.0;
        let thickness = graph_rect.w * graph_rect.h;

        Ok(Params {
            size,
            t_range,
            img_rect,
            graph_rect,
            graph_pixel_r,
            thickness,
            plain_diff: args.plain_diff,
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
            context.set_var("x", x.clone());
            context.set_var("y", y.clone());
            context.set_var("r", r_coeff * r.clone());
            if r_coeff == 1 {
                context.set_var("t", t.clone() + TAU * (i as f64));
            } else {
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

        // TODO handle errors more nicely
        let lhs_expr =
            mexprp::Expression::parse_ctx(lhs, init_context.clone()).unwrap();
        let rhs_expr =
            mexprp::Expression::parse_ctx(rhs, init_context.clone()).unwrap();

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
