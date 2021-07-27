use crate::fuzzyplot::{setup::Params, Plot, Point};
use itertools::Itertools;
use rug::Complex;

pub fn diff(
    context: &mexprp::Context<Complex>,
    plot: &Plot,
    params: &Params,
) -> f64 {
    let lhs_results = plot.lhs_expr.eval_ctx(context).unwrap().to_vec();
    let rhs_results = plot.rhs_expr.eval_ctx(context).unwrap().to_vec();
    let mut min_d = f64::INFINITY;
    for (lhs, rhs) in lhs_results.iter().cartesian_product(rhs_results.iter()) {
        min_d = min_d.min(if params.plain_diff {
            let top = Complex::with_val(53, lhs - rhs);
            top.norm().real().to_f64()
        } else {
            let top = Complex::with_val(53, lhs - rhs);
            let bottom = Complex::with_val(53, lhs + rhs);
            (top / bottom).norm().real().to_f64()
        });
    }
    ((min_d * params.sharpness).powi(2) + 1.0).powi(-1)
}

pub fn axis_diff(p: &Point, params: &Params) -> f64 {
    let pix_r = params.graph_pixel_r;
    if p.x.abs() < pix_r || p.y.abs() < pix_r {
        1.0
    } else {
        0.0
    }
}

pub fn grid_diff(p: &Point, params: &Params) -> f64 {
    let pix_r = params.graph_pixel_r;
    if (p.x + pix_r).rem_euclid(params.grid_size).abs() < 2.0 * pix_r
        || (p.y + pix_r).rem_euclid(params.grid_size).abs() < 2.0 * pix_r
    {
        0.4
    } else {
        0.0
    }
}
