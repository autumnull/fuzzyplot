use crate::fuzzyplot::{
    cli::Cli, diff, expr::LANES, setup, setup::Params, Point,
};
use anyhow::{Context, Result};
use clap::Parser;
use image::{Rgb, RgbImage};
use std::sync::Mutex;
use std::thread;

mod fuzzyplot;

fn main() -> Result<()> {
    let args = Cli::parse();

    let params = Params::from_args(&args)?;
    let plots = setup::make_plots(&args.equations, &params)?;
    let (width, height) = params.size;

    let mut img = RgbImage::new(width, height);
    let rows = Mutex::new(img.enumerate_rows_mut());
    let threads = thread::available_parallelism().map_or(1, |n| n.get());

    thread::scope(|scope| {
        for _thread in 0..threads {
            scope.spawn(|| {
                let mut buffers: Vec<_> =
                    plots.iter().map(|plot| plot.program.buffer()).collect();
                let mut contexts = vec![Vec::new(); LANES];
                loop {
                    let next = rows.lock().unwrap().next();
                    let mut row: Vec<_> = if let Some((_, row)) = next {
                        row.collect()
                    } else {
                        break;
                    };
                    for chunk in row.chunks_mut(LANES) {
                        for ((x, y, pixel), contexts) in
                            chunk.iter_mut().zip(&mut contexts)
                        {
                            let img_point = Point {
                                x: *x as f64,
                                y: (height - 1 - *y) as f64,
                            };
                            let graph_point = params
                                .img_rect
                                .map_point(&img_point, &params.graph_rect);

                            **pixel = Rgb([255, 255, 255]);
                            if params.draw_axes {
                                let axisness =
                                    diff::axis_diff(&graph_point, &params).max(
                                        diff::grid_diff(&graph_point, &params),
                                    ) * 255.0;
                                for channel in 0..3 {
                                    pixel[channel] -=
                                        (axisness as u8).min(pixel[channel]);
                                }
                            };
                            setup::make_contexts(
                                &graph_point,
                                params.t_range,
                                contexts,
                            );
                        }
                        for (plot, buffer) in plots.iter().zip(&mut buffers) {
                            let count =
                                setup::context_count(plot, contexts[0].len());
                            let mut max_diffs = [0.0_f64; LANES];
                            for i in 0..count {
                                for (lane, contexts) in
                                    contexts[..chunk.len()].iter().enumerate()
                                {
                                    plot.program.set_vars(
                                        buffer,
                                        lane,
                                        contexts[i],
                                    );
                                }
                                plot.program.eval(buffer);
                                for (lane, max_diff) in max_diffs[..chunk.len()]
                                    .iter_mut()
                                    .enumerate()
                                {
                                    for value in plot.program.results(
                                        buffer,
                                        plot.result.clone(),
                                        lane,
                                    ) {
                                        *max_diff = max_diff.max(
                                            value.re.clamp(0.0, 1.0) * 255.0,
                                        );
                                    }
                                }
                            }
                            for ((_, _, pixel), max_diff) in
                                chunk.iter_mut().zip(&max_diffs)
                            {
                                for channel in 0..3 {
                                    if (plot.color >> channel) & 1 == 0 {
                                        pixel[channel] -= (*max_diff as u8)
                                            .min(pixel[channel]);
                                    }
                                }
                            }
                        }
                    }
                }
            });
        }
    });

    img.save(&args.outfile).with_context(|| {
        format!("Couldn't save file '{}'", args.outfile.display())
    })?;
    Ok(())
}
