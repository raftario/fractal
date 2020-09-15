mod colour;
mod config;
mod mandelbrot;

use self::{colour::Colour, config::Config};
use anyhow::{Error, Result};
use chrono::Local;
use ndarray::{Array2, Zip};
use notify::{RecursiveMode, Watcher};
use num_complex::Complex64;
use png::{BitDepth, ColorType, Encoder};
use sdl2::{
    event::{Event, WindowEvent},
    render::WindowCanvas,
};
use std::{
    fs::{self, File},
    io::{BufWriter, Write},
    path::PathBuf,
    sync::mpsc::{self, TryRecvError},
    thread,
    time::Duration,
};
use structopt::StructOpt;

#[derive(StructOpt)]
#[structopt(about, author)]
struct Opt {
    /// File to load the configuration from
    #[structopt(name = "FILE", default_value = "fractal.toml", env = "FRACTAL_CONFIG")]
    config: PathBuf,
}

#[paw::main]
fn main(args: Opt) -> Result<()> {
    let (mut config, config_rx, _w) = if fs::metadata(&args.config).is_ok() {
        println!(
            "[CONFIG] Using {}, refreshing enabled",
            args.config.display()
        );
        let c = self::config::read(&args.config)?;
        let (tx, rx) = mpsc::channel();
        let mut watcher = notify::watcher(tx, Duration::from_secs(2))?;
        watcher.watch(&args.config, RecursiveMode::NonRecursive)?;
        (c, Some(rx), Some(watcher))
    } else {
        println!("[CONFIG] Using default, refreshing disabled");
        (Default::default(), None, None)
    };

    let ctx = sdl2::init().map_err(Error::msg)?;
    let video = ctx.video().map_err(Error::msg)?;
    let window = video
        .window(
            env!("CARGO_PKG_NAME"),
            config.preview.resolution.width as _,
            config.preview.resolution.height as _,
        )
        .position_centered()
        .resizable()
        .allow_highdpi()
        .build()?;

    let mut canvas = window.into_canvas().build()?;
    canvas.set_logical_size(
        config.preview.resolution.width as _,
        config.preview.resolution.height as _,
    )?;
    let mut events = ctx.event_pump().map_err(Error::msg)?;

    let (mut fractal_dimensions, mut fractal_offsets) = {
        if config.preview.resolution.width as f64 / config.preview.resolution.height as f64
            >= 3.5 / 2.0
        {
            let width = 2.0 * config.preview.resolution.width as f64
                / config.preview.resolution.height as f64;
            ((width, 2.0), (-2.5 + (3.5 - width) / 2.0, -1.0))
        } else {
            let height = 3.5 * config.preview.resolution.height as f64
                / config.preview.resolution.width as f64;
            ((3.5, height), (-2.5, -1.0 + (1.0 - height) / 2.0))
        }
    };
    let mut scale_factor = fractal_dimensions.0 / config.preview.resolution.width as f64;

    preview(scale_factor, fractal_offsets, &mut canvas, &config)?;
    loop {
        if let Some(rx) = &config_rx {
            match rx.try_recv() {
                Ok(_) => match self::config::read(&args.config) {
                    Ok(c) => {
                        eprintln!("[CONFIG] Refreshed");

                        scale_factor = scale(
                            scale_factor,
                            (
                                config.preview.resolution.width as f64,
                                config.preview.resolution.height as f64,
                            ),
                            (
                                c.preview.resolution.width as f64,
                                c.preview.resolution.height as f64,
                            ),
                        );
                        config = c;
                        canvas.set_logical_size(
                            config.preview.resolution.width as _,
                            config.preview.resolution.height as _,
                        )?;

                        preview(scale_factor, fractal_offsets, &mut canvas, &config)?;
                    }
                    Err(e) => eprintln!("[CONFIG] {}", e),
                },
                Err(TryRecvError::Disconnected) => break,
                _ => (),
            }
        }

        match events.wait_event_timeout(2000) {
            Some(Event::Quit { .. }) => break,

            Some(Event::Window {
                win_event: WindowEvent::Resized(_, _),
                ..
            })
            | Some(Event::Window {
                win_event: WindowEvent::SizeChanged(_, _),
                ..
            }) => preview(scale_factor, fractal_offsets, &mut canvas, &config)?,

            Some(Event::KeyUp {
                keycode: Some(k), ..
            }) if config.preview.keys.zoom_in == k => {
                let old_center = (
                    fractal_dimensions.0 / 2.0 + fractal_offsets.0,
                    fractal_dimensions.1 / 2.0 + fractal_offsets.1,
                );

                fractal_dimensions.0 /= config.preview.zoom_factor;
                fractal_dimensions.1 /= config.preview.zoom_factor;

                let new_center = (fractal_dimensions.0 / 2.0, fractal_dimensions.1 / 2.0);
                fractal_offsets.0 = old_center.0 - new_center.0;
                fractal_offsets.1 = old_center.1 - new_center.1;

                scale_factor = fractal_dimensions.0 / config.preview.resolution.width as f64;
                preview(scale_factor, fractal_offsets, &mut canvas, &config)?;
            }
            Some(Event::KeyUp {
                keycode: Some(k), ..
            }) if config.preview.keys.zoom_out == k => {
                let old_center = (
                    fractal_dimensions.0 / 2.0 + fractal_offsets.0,
                    fractal_dimensions.1 / 2.0 + fractal_offsets.1,
                );

                fractal_dimensions.0 *= config.preview.zoom_factor;
                fractal_dimensions.1 *= config.preview.zoom_factor;

                let new_center = (fractal_dimensions.0 / 2.0, fractal_dimensions.1 / 2.0);
                fractal_offsets.0 = old_center.0 - new_center.0;
                fractal_offsets.1 = old_center.1 - new_center.1;

                scale_factor = fractal_dimensions.0 / config.preview.resolution.width as f64;
                preview(scale_factor, fractal_offsets, &mut canvas, &config)?;
            }

            Some(Event::KeyUp {
                keycode: Some(k), ..
            }) if config.preview.keys.up == k => {
                fractal_offsets.1 -= fractal_dimensions.1 * config.preview.move_factor;
                preview(scale_factor, fractal_offsets, &mut canvas, &config)?;
            }
            Some(Event::KeyUp {
                keycode: Some(k), ..
            }) if config.preview.keys.left == k => {
                fractal_offsets.0 -= fractal_dimensions.0 * config.preview.move_factor;
                preview(scale_factor, fractal_offsets, &mut canvas, &config)?;
            }
            Some(Event::KeyUp {
                keycode: Some(k), ..
            }) if config.preview.keys.down == k => {
                fractal_offsets.1 += fractal_dimensions.1 * config.preview.move_factor;
                preview(scale_factor, fractal_offsets, &mut canvas, &config)?;
            }
            Some(Event::KeyUp {
                keycode: Some(k), ..
            }) if config.preview.keys.right == k => {
                fractal_offsets.0 += fractal_dimensions.0 * config.preview.move_factor;
                preview(scale_factor, fractal_offsets, &mut canvas, &config)?;
            }

            Some(Event::KeyUp {
                keycode: Some(k), ..
            }) if config.preview.keys.render == k => {
                render(scale_factor, fractal_offsets, config.clone())
            }

            Some(Event::MouseButtonUp { x, y, .. }) => {
                let x = x as f64 * scale_factor + fractal_offsets.0;
                let y = y as f64 * scale_factor + fractal_offsets.1;
                println!("[COORDS] ({}, {})", x, y);
            }

            _ => (),
        }
    }

    Ok(())
}

fn preview(
    scale_factor: f64,
    offsets: (f64, f64),
    canvas: &mut WindowCanvas,
    config: &Config,
) -> Result<()> {
    let (x, y) = (
        config.preview.resolution.width,
        config.preview.resolution.height,
    );
    for x in 0..(x as i32) {
        for y in 0..(y as i32) {
            let c = Complex64::new(
                x as f64 * scale_factor + offsets.0,
                y as f64 * scale_factor + offsets.1,
            );
            let colour = self::mandelbrot::colourise(c, config.max_iterations, &config.gradient);
            canvas.set_draw_color((colour.r, colour.g, colour.b));
            canvas.draw_point((x, y)).map_err(Error::msg)?;
        }
    }
    canvas.present();
    Ok(())
}

fn scale(mut factor: f64, old: (f64, f64), new: (f64, f64)) -> f64 {
    if new.0 / new.1 > old.0 / old.1 {
        factor *= old.0 / new.0;
    } else {
        factor *= old.1 / new.1;
    }
    factor
}

fn render(scale_factor: f64, offsets: (f64, f64), config: Config) {
    println!("[RENDER] Started rendering");
    thread::spawn(move || match render_inner(scale_factor, offsets, config) {
        Ok(p) => println!("[RENDER] Done rendering {}", p.display()),
        Err(e) => eprintln!("[RENDER] {}", e),
    });
}

fn render_inner(mut scale_factor: f64, offsets: (f64, f64), config: Config) -> Result<PathBuf> {
    scale_factor = scale(
        scale_factor,
        (
            config.preview.resolution.width as f64,
            config.preview.resolution.height as f64,
        ),
        (
            config.render.resolution.width as f64,
            config.render.resolution.height as f64,
        ),
    );

    let timestamp = Local::now();
    let filename = format!("{}.png", timestamp.format("%Y-%m-%d_%H-%M-%S"));
    let filepath = config.render.directory.join(filename);

    fs::create_dir_all(&config.render.directory)?;

    let (width, height) = (
        config.render.resolution.width,
        config.render.resolution.height,
    );
    let mut matrix: Array2<Colour> =
        Array2::from_elem((width, height), Colour { r: 0, g: 0, b: 0 });
    Zip::indexed(&mut matrix).par_apply(|(x, y), colour| {
        let c = Complex64::new(
            x as f64 * scale_factor + offsets.0,
            y as f64 * scale_factor + offsets.1,
        );
        *colour = self::mandelbrot::colourise(c, config.max_iterations, &config.gradient);
    });

    let mut encoder = Encoder::new(
        BufWriter::new(File::create(&filepath)?),
        width as _,
        height as _,
    );
    encoder.set_color(ColorType::RGB);
    encoder.set_depth(BitDepth::Eight);
    let mut writer = encoder.write_header()?.into_stream_writer();

    for y in 0..height {
        for x in 0..width {
            let colour = matrix[[x, y]];
            writer.write_all(&[colour.r, colour.g, colour.b])?;
        }
    }

    writer.finish()?;
    Ok(filepath)
}
