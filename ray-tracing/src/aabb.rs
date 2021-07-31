use std::mem;

use crate::ray::{Point, Ray};

#[derive(Default, Clone, Copy)]
pub struct Aabb {
    minimum: Point,
    maximum: Point,
}

impl Aabb {
    pub fn new(minimum: Point, maximum: Point) -> Self {
        Self { minimum, maximum }
    }

    /// Get a reference to the aabb's minimum.
    pub fn min(&self) -> &Point {
        &self.minimum
    }

    /// Get a reference to the aabb's maximum.
    pub fn max(&self) -> &Point {
        &self.maximum
    }

    pub fn hit(&self, r: &Ray, mut t_min: f64, mut t_max: f64) -> bool {
        for a in 0..3 {
            let max = self.max().data()[a];
            let min = self.min().data()[a];
            let org = r.origin().data()[a];
            let dir = r.direction().data()[a];
            let inv_d = 1.0 / dir;

            let mut t0 = (min - org) * inv_d;
            let mut t1 = (max - org) * inv_d;

            if inv_d < 0.0 {
                mem::swap(&mut t0, &mut t1);
            }

            t_min = t0.max(t_min);
            t_max = t1.min(t_max);

            if t_max <= t_min {
                return false;
            }
        }
        true
    }

    pub fn surrounding_box(box0: &Self, box1: &Self) -> Self {
        let calc = |b0: &Point, b1: &Point, func: &dyn Fn(f64, f64) -> f64| {
            let mut a = [0.0; 3];
            for (i, v) in a.iter_mut().enumerate() {
                *v = func(b0.data()[i], b1.data()[i]);
            }
            a.into()
        };

        let small = calc(box0.min(), box1.min(), &f64::min);
        let big = calc(box0.max(), box1.max(), &f64::max);

        Aabb::new(small, big)
    }
}
