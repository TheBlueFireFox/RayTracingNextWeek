use crate::helpers;

pub type Point = helpers::cvec::Point<f64>;
pub type Vec3  = helpers::cvec::Vec3<f64>;

pub struct Ray {
    orig: Point,
    dir: Vec3,
    tm: f64,
}

impl Ray {
    pub fn new(orig: Point, dir: Vec3) -> Self {
        Self::with_time(orig, dir, 0.0)
    }

    pub fn with_time(orig: Point, dir: Vec3, time: f64) -> Self {
        Self {
            orig,
            dir,
            tm: time,
        }
    }

    pub fn time(&self) -> f64 {
        self.tm
    }

    pub fn origin(&self) -> Point {
        self.orig
    }

    pub fn direction(&self) -> Vec3 {
        self.dir
    }

    pub fn at(&self, t: f64) -> Point {
        self.orig + t * self.dir
    }
}
