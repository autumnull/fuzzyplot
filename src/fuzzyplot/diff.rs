use crate::fuzzyplot::{setup::Params, Point};

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
