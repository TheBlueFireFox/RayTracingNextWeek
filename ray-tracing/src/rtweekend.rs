use rand::{
    distributions::uniform::{SampleRange, SampleUniform},
    Rng,
};
use std::f64::consts::PI;

#[inline]
pub fn degrees_to_radians(deg: f64) -> f64 {
    debug_assert!(deg >= 0.0 && deg <= 360.0);
    deg * PI / 180.0
}

#[inline]
pub fn clamp<T: PartialOrd>(x: T, min: T, max: T) -> T {
    if x < min {
        min
    } else if x > max {
        max
    } else {
        x
    }
}

#[inline]
pub fn rand_range<T, R>(range: R) -> T
where
    T: SampleUniform,
    R: SampleRange<T>,
{
    rand::thread_rng().gen_range(range)
}
