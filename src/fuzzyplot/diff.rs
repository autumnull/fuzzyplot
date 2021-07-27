use crate::fuzzyplot::{setup::Params, Plot, Point};
use itertools::Itertools;
use rug::Complex;

const ACCURACY_CONST: f64 = (1 << 16) as f64;
const AXIS_CONST: f64 = 0.0001;
const GRID_SIZE: f64 = 1.0;

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
    min_d.powi(-2) / ACCURACY_CONST * params.thickness
}

pub fn axis_diff(p: &Point, params: &Params) -> f64 {
    (p.x.powi(-2) + p.y.powi(-2)) * params.thickness * AXIS_CONST
}

pub fn grid_diff(p: &Point, pixel_r: f64) -> f64 {
    if (p.x + pixel_r).rem_euclid(GRID_SIZE).abs() < 2.0 * pixel_r
        || (p.y + pixel_r).rem_euclid(GRID_SIZE).abs() < 2.0 * pixel_r
    {
        100.0
    } else {
        0.0
    }
}
