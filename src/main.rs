use crate::fuzzyplot::*;
use crate::fuzzyplot::cli::Cli;
use crate::fuzzyplot::diff::*;
use crate::fuzzyplot::setup::*;

use anyhow::{Context, Result};
use image::{RgbImage, Rgb};
use indicatif::ProgressIterator;
use structopt::StructOpt;
use rug::ops::Pow;

mod fuzzyplot;

fn main() -> Result<()> {
    let mut args = Cli::from_args();
    parse_args(&mut args)?;
    
    let (width, height) = (args.size[0], args.size[1]);
    
    let img_rect = Rect{
        x: 0.0,
        y: 0.0,
        w: width as f64,
        h: height as f64,
    };
    let aspect_ratio = (width / height) as f64;
    let graph_rect_r = 2.0.pow(-args.zoom);
    let graph_rect = Rect{
        x: -graph_rect_r * aspect_ratio,
        y: -graph_rect_r,
        w: graph_rect_r * 2.0 * aspect_ratio,
        h: graph_rect_r * 2.0,
    };
    let graph_pixel_r = graph_rect.w / img_rect.w / 2.0;
    
    let params = Params {
        plain_diff: args.plain_diff,
        thickness: graph_rect.w * graph_rect.h,
    };
    
    let plots = make_plots(&args.equations)?;
    
    let mut img = RgbImage::new(width, height);
    
    println!("Generating image...");
    
    let pb = indicatif::ProgressBar::new((height * width) as u64);
    pb.set_style(indicatif::ProgressStyle::default_bar()
        .template("{prefix:>10.green} [{wide_bar}] {pos:>6}/{len:6} ({eta:>3})")
        .progress_chars("=> "));
    pb.set_prefix("Plotting");
    pb.set_draw_rate(3);
    
    // TODO multithreading
    for (x, y, pixel) in img.enumerate_pixels_mut().progress_with(pb) {
        let img_point = Point{x: x as f64, y: (height-1 - y) as f64};
        let graph_point = img_rect.map_point(&img_point, &graph_rect);
        
        *pixel = Rgb([255, 255, 255]);
        if ! args.no_axes {
            let axisness = axis_diff(&graph_point, &params)
                + grid_diff(&graph_point, graph_pixel_r);
            for channel in 0..3 {
                pixel[channel] -= (axisness as u8).min(pixel[channel]);
            }
        };
        let contexts = make_contexts(graph_point, &args.t_range);
        for plot in plots.iter() {
            let mut max_diff: u8 = 0;
            for context in contexts.iter() {
                max_diff = max_diff.max(diff(
                    &context,
                    &plot,
                    &params) as u8);
            }
            for channel in 0..3 {
                if (plot.color >> channel) & 1 == 1 {
                    pixel[channel] -= max_diff.min(pixel[channel]);
                }
            }
        }
    }
    
    img.save(&args.outfile)
        .with_context(
            || format!("Couldn't save file '{}'", args.outfile.display())
        )?;
    println!("Done!");
    Ok(())
}
