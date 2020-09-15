use crate::colour::{Colour, Gradient};
use num_complex::Complex64;
use num_traits::Zero;

#[derive(Copy, Clone)]
pub struct Area {
    pub x_start: f64,
    pub x_end: f64,
    pub y_start: f64,
    pub y_end: f64,
}

fn f(c: Complex64, z: Complex64) -> Complex64 {
    z * z + c
}

fn diverge_iterations(c: Complex64, max: usize) -> (usize, Complex64) {
    let mut z = Complex64::zero();
    let mut i = 0;
    while z.norm_sqr() <= 4.0 && i < max {
        z = f(c, z);
        i += 1;
    }
    (i, z)
}

fn colour_scalar(i: usize, z: Complex64, max: usize) -> f64 {
    let log_zn = z.norm_sqr().log10() / 2f64;
    let nu = (log_zn / 2f64.log10()).log2();
    (i as f64 + 1f64 - nu) / max as f64
}

pub fn colourise(c: Complex64, max: usize, gradient: &Gradient, black: Colour) -> Colour {
    let (i, z) = diverge_iterations(c, max);
    if i < max {
        let scalar = colour_scalar(i, z, max);
        gradient.get(scalar)
    } else {
        black
    }
}
