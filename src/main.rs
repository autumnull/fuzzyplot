use crate::fuzzyplot::{cli::Cli, diff, setup, setup::Params, Point};
use anyhow::{Context, Result};
use image::{Rgb, RgbImage};
use indicatif::ProgressIterator;
use std::{sync::mpsc, thread};
use structopt::StructOpt;

mod fuzzyplot;

struct Tag {
    x: u32,
    y: u32,
    result: Option<Rgb<u8>>,
    quit: bool,
}

fn main() -> Result<()> {
    let args = Cli::from_args();

    let params = Params::from_args(&args)?;
    let (width, height) = params.size;

    let mut handles = Vec::new();

    let (return_tx, return_rx) = mpsc::channel();
    
    let mut thread_channels = Vec::new();
    for i in 0..50 {
        let (tx, rx) = mpsc::channel();
        thread_channels.push(tx);
        let response = return_tx.clone();
        let equations = args.equations.clone();
        handles.push(thread::spawn(move || {
            println!("thread {} reporting for duty", i);
            loop {
                let mut item: Tag = rx.recv().unwrap();
                if item.quit {
                    break;
                }
                
                let plots = setup::make_plots(&equations).unwrap();

                let img_point = Point {
                    x: item.x as f64,
                    y: (height - 1 - item.y) as f64,
                };
                let graph_point =
                    params.img_rect.map_point(&img_point, &params.graph_rect);

                let mut color = Rgb([255, 255, 255]);
                if params.draw_axes {
                    let axisness = diff::axis_diff(&graph_point, &params)
                        + diff::grid_diff(&graph_point, params.graph_pixel_r);
                    for channel in 0..3 {
                        color[channel] -= (axisness as u8).min(color[channel]);
                    }
                };
                let contexts = setup::make_contexts(graph_point, params.t_range);
                for plot in plots.iter() {
                    let mut max_diff: u8 = 0;
                    for context in contexts.iter() {
                        max_diff = max_diff
                            .max(diff::diff(&context, &plot, &params) as u8);
                    }
                    for channel in 0..3 {
                        if (plot.color >> channel) & 1 == 0 {
                            color[channel] -= max_diff.min(color[channel]);
                        }
                    }
                }
                item.result = Some(color);
                response.send(item).unwrap();
            }
        }))
    }
    
    let mut img = RgbImage::new(width, height);
    
    println!("Generating image...");

    // TODO multithreading
    let mut t = 0;
    for (x, y, _) in img.enumerate_pixels() {
        thread_channels[t].send(Tag {
            x,
            y,
            result: None,
            quit: false,
        }).unwrap();
        t = (t + 1) % thread_channels.len();
    }
    
    for channel in thread_channels {
        channel.send(Tag{x: 0, y: 0, result: None, quit: true}).unwrap();
    }
    
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
    
    for _ in (0..(width * height)).progress_with(pb) {
        let item = return_rx.recv().unwrap();
        img.put_pixel(item.x, item.y, item.result.unwrap());
    }

    // wait for all threads to finish
    for handle in handles {
        handle.join().unwrap();
    }

    img.save(&args.outfile).with_context(|| {
        format!("Couldn't save file '{}'", args.outfile.display())
    })?;
    println!("Done!");
    Ok(())
}
