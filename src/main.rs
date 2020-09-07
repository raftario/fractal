mod mandelbrot;

use self::mandelbrot::Area;
use palette::{Gradient, LinSrgb};
use std::{path::PathBuf, str::FromStr};
use structopt::StructOpt;
use thiserror::Error;

#[derive(StructOpt)]
#[structopt(about, author)]
enum Opt {
    /// Renders the Mandelbrot set
    Mandelbrot {
        /// Width of the output image in pixels
        #[structopt(name = "WIDTH")]
        width: usize,
        /// Height of the output image in pixels
        #[structopt(name = "HEIGHT")]
        height: usize,
        /// Filename of the output image (the format will be determined by the extension)
        #[structopt(name = "OUTPUT")]
        output: PathBuf,

        /// Leftmost rendered point on the X axis
        #[structopt(
            short = "x",
            long,
            name = "x",
            default_value = "-2.5",
            display_order = 0
        )]
        x_start: f64,
        /// Rightmost rendered point on the X axis
        #[structopt(
            short = "X",
            long,
            name = "X",
            default_value = "1.0",
            display_order = 1
        )]
        x_end: f64,
        /// Leftmost rendered point on the Y axis
        #[structopt(
            short = "y",
            long,
            name = "y",
            default_value = "-1.0",
            display_order = 2
        )]
        y_start: f64,
        /// Rightmost rendered point on the Y axis
        #[structopt(
            short = "Y",
            long,
            name = "Y",
            default_value = "1.0",
            display_order = 3
        )]
        y_end: f64,

        /// Maximum number of iterations to go through before considering a point is part of the set
        #[structopt(
            short = "n",
            long,
            name = "ITERS",
            default_value = "1000",
            display_order = 4
        )]
        max_iterations: usize,
        /// RGB colour gradient
        #[structopt(
            short,
            long,
            name = "COLOURS",
            default_value = "#050a3c #8c2846 #f0c83c #050a3c",
            display_order = 5
        )]
        gradient: OptGradient,
    },
}

struct OptGradient(Gradient<LinSrgb>);

#[derive(Debug, Error)]
enum OptGradientFromStrError {
    #[error("{0}")]
    Custom(&'static str),
    #[error("invalid red value: {0}")]
    Red(std::num::ParseIntError),
    #[error("invalid green value: {0}")]
    Green(std::num::ParseIntError),
    #[error("invalid blue value: {0}")]
    Blue(std::num::ParseIntError),
}

impl FromStr for OptGradient {
    type Err = OptGradientFromStrError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let colours = s
            .split(' ')
            .map(|mut s| {
                s = s.trim();
                if !s.starts_with('#') || s.len() != 7 {
                    return Err(OptGradientFromStrError::Custom(
                        "colors must be in the `#rrggbb` format",
                    ));
                }

                let r = u8::from_str_radix(&s[1..3], 16).map_err(OptGradientFromStrError::Red)?;
                let g = u8::from_str_radix(&s[3..5], 16).map_err(OptGradientFromStrError::Green)?;
                let b = u8::from_str_radix(&s[5..7], 16).map_err(OptGradientFromStrError::Blue)?;

                Ok(LinSrgb::new(
                    r as f32 / 255.0,
                    g as f32 / 255.0,
                    b as f32 / 255.0,
                ))
            })
            .collect::<Result<Vec<LinSrgb>, Self::Err>>()?;
        Ok(Self(Gradient::new(colours)))
    }
}

#[paw::main]
fn main(args: Opt) -> image::ImageResult<()> {
    match args {
        Opt::Mandelbrot {
            width,
            height,
            output,
            x_start,
            x_end,
            y_start,
            y_end,
            max_iterations,
            gradient,
        } => {
            let area = Area {
                x_start,
                x_end,
                y_start,
                y_end,
            };
            let gradient = gradient.0;
            let image =
                self::mandelbrot::mandelbrot(width, height, &area, max_iterations, &gradient);
            image.save(output)
        }
    }
}
