use image::RgbImage;
use ndarray::{Array2 as Matrix, Zip};
use num_complex::Complex64 as Complex;
use num_traits::Zero;
use palette::{Gradient, LinSrgb};

#[derive(Copy, Clone)]
pub struct Area {
    pub x_start: f64,
    pub x_end: f64,
    pub y_start: f64,
    pub y_end: f64,
}

impl Default for Area {
    fn default() -> Self {
        Area {
            x_start: -2.5,
            x_end: 1.0,
            y_start: -1.0,
            y_end: 1.0,
        }
    }
}

fn f(c: Complex, z: Complex) -> Complex {
    z.powf(2.0) + c
}

fn diverge_iterations(c: Complex, max: usize) -> (usize, Complex) {
    let mut z = Complex::zero();
    let mut i = 0;
    while z.norm_sqr() <= 4.0 && i < max {
        z = f(c, z);
        i += 1;
    }
    (i, z)
}

fn colour_scalar(i: usize, z: Complex, max: usize) -> f64 {
    let log_zn = z.norm_sqr().log10();
    let nu = (log_zn / 2f64.log10()).log10() / 2f64.log10();
    (i as f64 + 1f64 - nu) / max as f64
}

fn colourise(c: Complex, max: usize, gradient: &Gradient<LinSrgb>) -> LinSrgb {
    let (i, z) = diverge_iterations(c, max);
    if i < max {
        let scalar = colour_scalar(i, z, max);
        gradient.get(scalar as _)
    } else {
        LinSrgb::new(0.0, 0.0, 0.0)
    }
}

pub fn mandelbrot(
    width: usize,
    height: usize,
    area: &Area,
    max: usize,
    gradient: &Gradient<LinSrgb>,
) -> RgbImage {
    let image_ratio = width as f64 / height as f64;
    let area_ratio = (area.x_end - area.x_start) / (area.y_end - area.y_start);
    let scaling_factor = if image_ratio < area_ratio {
        (area.x_end - area.x_start) / width as f64
    } else {
        (area.y_end - area.y_start) / height as f64
    };

    let mut matrix: Matrix<LinSrgb> =
        Matrix::from_elem((width, height), LinSrgb::new(0.0, 0.0, 0.0));
    Zip::indexed(&mut matrix).par_apply(|(x, y), colour| {
        let c = Complex::new(
            x as f64 * scaling_factor + area.x_start,
            y as f64 * scaling_factor + area.y_start,
        );
        *colour = colourise(c, max, &gradient);
    });

    let mut image = RgbImage::new(width as _, height as _);
    for (x, y, colour) in image.enumerate_pixels_mut() {
        let c = matrix[[x as _, y as _]];
        colour[0] = (c.red * 255.0) as _;
        colour[1] = (c.green * 255.0) as _;
        colour[2] = (c.blue * 255.0) as _;
    }
    image
}
