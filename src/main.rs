use crate::fuzzyplot::{cli::Cli, diff, setup, setup::Params, Point};
use anyhow::{Context, Result};
use image::{Rgb, RgbImage};
use indicatif::ProgressIterator;
use structopt::StructOpt;

mod fuzzyplot;

fn main() -> Result<()> {
    let args = Cli::from_args();

    let params = Params::from_args(&args)?;
    let (width, height) = params.size;

    let plots = setup::make_plots(&args.equations)?;

    let mut img = RgbImage::new(width, height);

    println!("Generating image...");

    let pb = indicatif::ProgressBar::new((width * height) as u64);
    pb.set_style(
        indicatif::ProgressStyle::default_bar()
            .template(
                "{prefix:>10.green} [{wide_bar}] {pos:>6}/{len:6} ({eta:>3})",
            )
            .progress_chars("=> "),
    );
    pb.set_prefix("Plotting");
    pb.set_draw_rate(3);

    // TODO multithreading
    for (x, y, pixel) in img.enumerate_pixels_mut().progress_with(pb) {
        let img_point = Point {
            x: x as f64,
            y: (height - 1 - y) as f64,
        };
        let graph_point =
            params.img_rect.map_point(&img_point, &params.graph_rect);

        *pixel = Rgb([255, 255, 255]);
        if !args.no_axes {
            let axisness = diff::axis_diff(&graph_point, &params)
                + diff::grid_diff(&graph_point, params.graph_pixel_r);
            for channel in 0..3 {
                pixel[channel] -= (axisness as u8).min(pixel[channel]);
            }
        };
        let contexts = setup::make_contexts(graph_point, params.t_range);
        for plot in plots.iter() {
            let mut max_diff: u8 = 0;
            for context in contexts.iter() {
                max_diff =
                    max_diff.max(diff::diff(&context, &plot, &params) as u8);
            }
            for channel in 0..3 {
                if (plot.color >> channel) & 1 == 0 {
                    pixel[channel] -= max_diff.min(pixel[channel]);
                }
            }
        }
    }

    img.save(&args.outfile).with_context(|| {
        format!("Couldn't save file '{}'", args.outfile.display())
    })?;
    println!("Done!");
    Ok(())
}
