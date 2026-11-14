use crate::fuzzyplot::{cli::Cli, diff, setup, setup::Params, Point};
use anyhow::{Context, Result};
use clap::Parser;
use image::RgbImage;
use std::sync::Mutex;
use std::thread;

mod fuzzyplot;

const TILE: usize = 32;

fn main() -> Result<()> {
    let args = Cli::parse();

    let params = Params::from_args(&args)?;
    let plots = setup::make_plots(&args.equations, &params)?;
    let (width, height) = params.size;
    let halo = plots
        .iter()
        .map(|plot| plot.program.halo)
        .max()
        .unwrap_or(0);
    let tile_width = TILE + 2 * halo;
    let lanes = tile_width * tile_width;

    let layers: Vec<Vec<_>> = plots
        .iter()
        .map(|plot| {
            let poles = plot.poles.clone().map(|p| (p, plot.pole_color));
            std::iter::once((plot.zeros.clone(), plot.zero_color))
                .chain(poles)
                .collect()
        })
        .collect();

    let mut img = RgbImage::new(width, height);
    let row_len = width as usize * 3;
    let strips = Mutex::new(img.chunks_mut(TILE * row_len).enumerate());
    let threads = thread::available_parallelism().map_or(1, |n| n.get());

    thread::scope(|scope| {
        for _thread in 0..threads {
            scope.spawn(|| {
                let mut scratches: Vec<_> = plots
                    .iter()
                    .map(|plot| plot.program.scratch(tile_width))
                    .collect();
                let mut contexts = vec![Vec::new(); lanes];
                let mut core = Vec::new();
                loop {
                    let next = strips.lock().unwrap().next();
                    let (index, strip) = if let Some(next) = next {
                        next
                    } else {
                        break;
                    };
                    let top = index * TILE;
                    let rows = strip.len() / row_len;
                    for left in (0..width as usize).step_by(TILE) {
                        let graph_point = |lane: usize| {
                            let x = (left + lane % tile_width) as f64;
                            let y = (top + lane / tile_width) as f64;
                            let img_point = Point {
                                x: x - halo as f64,
                                y: (height - 1) as f64 - (y - halo as f64),
                            };
                            params
                                .img_rect
                                .map_point(&img_point, &params.graph_rect)
                        };
                        for (lane, contexts) in contexts.iter_mut().enumerate()
                        {
                            setup::make_contexts(
                                &graph_point(lane),
                                params.t_range,
                                contexts,
                            );
                        }

                        core.clear();
                        for row in 0..rows {
                            for col in 0..TILE.min(width as usize - left) {
                                let lane =
                                    (row + halo) * tile_width + col + halo;
                                let offset =
                                    (row * width as usize + left + col) * 3;
                                core.push((lane, offset));
                            }
                        }
                        for &(lane, offset) in &core {
                            let pixel = &mut strip[offset..offset + 3];
                            pixel.fill(255);
                            if params.draw_axes {
                                let point = graph_point(lane);
                                let axisness = diff::axis_diff(&point, &params)
                                    .max(diff::grid_diff(&point, &params))
                                    * 255.0;
                                for channel in pixel.iter_mut() {
                                    *channel -= (axisness as u8).min(*channel);
                                }
                            }
                        }

                        for ((plot, scratch), layers) in
                            plots.iter().zip(&mut scratches).zip(&layers)
                        {
                            let count =
                                setup::context_count(plot, contexts[0].len());
                            let mut intensities =
                                vec![0.0_f64; core.len() * layers.len()];
                            for i in 0..count {
                                for (lane, contexts) in
                                    contexts.iter().enumerate()
                                {
                                    plot.program.set_vars(
                                        scratch,
                                        lane,
                                        contexts[i],
                                    );
                                }
                                plot.program.eval(scratch);
                                for (&(lane, _), intensities) in core
                                    .iter()
                                    .zip(intensities.chunks_mut(layers.len()))
                                {
                                    for ((range, _), intensity) in
                                        layers.iter().zip(intensities)
                                    {
                                        for value in plot.program.results(
                                            scratch,
                                            range.clone(),
                                            lane,
                                        ) {
                                            *intensity = intensity.max(
                                                value.re.clamp(0.0, 1.0)
                                                    * 255.0,
                                            );
                                        }
                                    }
                                }
                            }
                            for (&(_, offset), intensities) in core
                                .iter()
                                .zip(intensities.chunks(layers.len()))
                            {
                                let pixel = &mut strip[offset..offset + 3];
                                for ((_, color), intensity) in
                                    layers.iter().zip(intensities)
                                {
                                    for (value, filter) in
                                        pixel.iter_mut().zip(color)
                                    {
                                        let amount = intensity * (1.0 - filter);
                                        *value -= (amount as u8).min(*value);
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
