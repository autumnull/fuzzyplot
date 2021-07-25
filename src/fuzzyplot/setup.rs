use crate::fuzzyplot::cli::Cli;
use crate::fuzzyplot::{Plot, Point, Rect};
use anyhow::{anyhow, Context, Result};
use rug::Complex;
use std::f64::consts::TAU;

pub fn parse_args(args: &mut Cli) -> Result<()> {
    // check valid image format before proceeding
    image::ImageFormat::from_path(args.outfile.as_path())
        .with_context(|| format!("Unrecognized file extension for image"))?;

    if args.size.len() != 2 {
        args.size = vec![800, 800];
    }
    if args.t_range.len() != 2 {
        args.t_range = vec![0, 0];
    }
    Ok(())
}

pub fn make_rects(width: u32, height: u32, zoom: f64) -> (Rect, Rect) {
    let img_rect = Rect {
        x: 0.0,
        y: 0.0,
        w: width as f64,
        h: height as f64,
    };
    let aspect_ratio = (width / height) as f64;
    let graph_rect_r = 2.0_f64.powf(-zoom);
    let graph_rect = Rect {
        x: -graph_rect_r * aspect_ratio,
        y: -graph_rect_r,
        w: graph_rect_r * 2.0 * aspect_ratio,
        h: graph_rect_r * 2.0,
    };
    (img_rect, graph_rect)
}

pub fn make_contexts(
    p: Point,
    t_range: &Vec<i32>,
) -> Vec<mexprp::Context<Complex>> {
    let x = Complex::with_val(53, (p.x, 0.0));
    let y = Complex::with_val(53, (p.y, 0.0));
    let r = Complex::with_val(53, ((p.x.powi(2) + p.y.powi(2)).sqrt(), 0.0));
    let t = Complex::with_val(53, (p.y.atan2(p.x), 0.0));

    let mut contexts: Vec<mexprp::Context<Complex>> = Vec::new();
    for i in (t_range[0] * 2)..((t_range[1] + 1) * 2) {
        let mut context = mexprp::Context::new();
        context.set_var("x", x.clone());
        context.set_var("y", y.clone());
        if i % 2 == 0 {
            context.set_var("t", t.clone() + TAU * (i.div_euclid(2) as f64));
            context.set_var("r", r.clone());
        } else {
            let half_circle =
                if *t.real() < 0 { TAU / 2.0 } else { -TAU / 2.0 };
            context.set_var(
                "t",
                t.clone() + half_circle + TAU * (i.div_euclid(2) as f64),
            );
            context.set_var("r", -r.clone());
        }
        contexts.push(context);
    }
    contexts
}

pub fn make_plots(equations: &Vec<String>) -> Result<Vec<Plot>> {
    let init_context =
        make_contexts(Point { x: 0.0, y: 0.0 }, &vec![0, 1])[0].clone();
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
            6
        } else {
            1 << i as u8
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
