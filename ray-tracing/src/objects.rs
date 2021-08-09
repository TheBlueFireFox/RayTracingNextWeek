use std::{f64::consts::PI, sync::Arc};

use crate::{aabb::Aabb, hittable::{HitRecord, Hittable}, material::{Mat, Material}, ray::{Point, Ray, Vec3}};

pub struct Sphere {
    pub center: Point,
    pub radius: f64,
    pub mat: Mat,
}

impl Sphere {
    pub fn new(center: Point, radius: f64, mat: Mat) -> Self {
        Self {
            center,
            radius,
            mat,
        }
    }

    /// Returns (u, v)
    /// p: a given point on the sphere of radius one, centered at the origin.
    /// u: returned value [0,1] of angle around the Y axis from X=-1.
    /// v: returned value [0,1] of angle from Y=-1 to Y=+1.
    ///     <1 0 0> yields <0.50 0.50>       <-1  0  0> yields <0.00 0.50>
    ///     <0 1 0> yields <0.50 1.00>       < 0 -1  0> yields <0.50 0.00>
    ///     <0 0 1> yields <0.25 0.50>       < 0  0 -1> yields <0.75 0.50>
    fn get_sphere_uv(&self, p: &Point) -> (f64, f64) {
        let theta = f64::acos(-p.y());
        let phi = f64::atan2(-p.z(), p.x()) + PI;
        let u = phi / (2.0 * PI);
        let v = theta / PI;
        (u, v)
    }
}

impl Hittable for Sphere {
    fn hit(&self, r: &Ray, t_min: f64, t_max: f64, rec: &mut HitRecord) -> bool {
        let oc = *r.origin() - self.center;
        let a = r.direction().length_squared();
        let half_b = Vec3::dot(&oc, &r.direction());
        let c = oc.length_squared() - self.radius * self.radius;

        let discriminant = half_b * half_b - a * c;

        if discriminant < 0.0 {
            return false;
        }

        let sqrtd = discriminant.sqrt();

        // Find the nearest root that lies in the acceptable range.
        let mut root = (-half_b - sqrtd) / a;
        if root < t_min || t_max < root {
            root = (-half_b + sqrtd) / a;
            if root < t_min || t_max < root {
                return false;
            }
        }
        rec.t = root;
        rec.p = r.at(rec.t);
        let outward_normal = (rec.p - self.center) / self.radius;
        rec.set_face_normal(r, &outward_normal);
        let t = self.get_sphere_uv(&outward_normal);
        rec.u = t.0;
        rec.v = t.1;
        rec.mat = Some(self.mat.clone());

        true
    }

    fn bounding_box(&self, _time0: f64, _time1: f64, output: &mut Aabb) -> bool {
        let v = [self.radius; 3].into();
        *output = Aabb::new(self.center - v, self.center + v);
        true
    }
}

pub struct MovingSphere {
    pub center0: Point,
    pub center1: Point,
    pub radius: f64,
    pub time0: f64,
    pub time1: f64,
    pub mat: Mat,
}

impl MovingSphere {
    pub fn new(
        center0: Point,
        center1: Point,
        time0: f64,
        time1: f64,
        radius: f64,
        mat: Mat,
    ) -> Self {
        Self {
            center0,
            center1,
            time0,
            time1,
            radius,
            mat,
        }
    }

    pub fn center(&self, time: f64) -> Point {
        self.center0
            + ((time - self.time0) / (self.time1 - self.time0)) * (self.center1 - self.center0)
    }
}

impl Hittable for MovingSphere {
    fn hit(&self, r: &Ray, t_min: f64, t_max: f64, rec: &mut HitRecord) -> bool {
        let oc = *r.origin() - self.center(r.time());
        let a = r.direction().length_squared();
        let half_b = Vec3::dot(&oc, &r.direction());
        let c = oc.length_squared() - self.radius * self.radius;

        let discriminant = half_b * half_b - a * c;

        if discriminant < 0.0 {
            return false;
        }

        let sqrtd = discriminant.sqrt();

        // Find the nearest root that lies in the acceptable range.
        let mut root = (-half_b - sqrtd) / a;
        if root < t_min || t_max < root {
            root = (-half_b + sqrtd) / a;
            if root < t_min || t_max < root {
                return false;
            }
        }
        rec.t = root;
        rec.p = r.at(rec.t);
        let outward_normal = (rec.p - self.center(r.time())) / self.radius;
        rec.set_face_normal(r, &outward_normal);
        rec.mat = Some(self.mat.clone());

        true
    }

    fn bounding_box(&self, time0: f64, time1: f64, output: &mut Aabb) -> bool {
        let calc = |time| {
            let cen = self.center(time);
            let vec_r = [self.radius; 3].into();
            Aabb::new(cen - vec_r, cen + vec_r)
        };
        let box0 = calc(time0);
        let box1 = calc(time1);

        *output = Aabb::surrounding_box(&box0, &box1);

        true
    }
}

pub struct XYRect {
    mp: Mat,
    x0: f64,
    x1: f64,
    y0: f64,
    y1: f64,
    k: f64,
}

impl XYRect {
    pub fn new<M: Material + 'static>(mp: M, x0: f64, x1: f64, y0: f64, y1: f64, k: f64) -> Self {
        Self {
            mp: Arc::new(mp),
            x0,
            x1,
            y0,
            y1,
            k,
        }
    }
}

impl Hittable for XYRect {
    fn bounding_box(&self, _time0: f64, _time1: f64, output: &mut Aabb) -> bool {
        // The bounding box must have non-zero width in each dimension, so pad the Z
        // dimension a small amount.
        *output = Aabb::new(
            [self.x0, self.y0, self.k - 0.0001].into(),
            [self.x1, self.y1, self.k + 0.0001].into(),
        );
        true
    }

    fn hit(&self, r: &Ray, t_min: f64, t_max: f64, rec: &mut HitRecord) -> bool {
        let org = r.origin();
        let dir = r.direction();
        let t = (self.k - org.z()) / dir.z();

        let bounds = |t, min, max| t < min || t > max;

        if bounds(t, t_min, t_max) {
            return false;
        }

        let x = org.x() + t * dir.x();
        let y = org.y() + t * dir.y();

        if bounds(x, self.x0, self.x1) || bounds(y, self.y0, self.y1) {
            return false;
        }

        rec.u = (x - self.x0) / (self.x1 - self.x0);
        rec.v = (y - self.y0) / (self.y1 - self.y0);
        rec.t = t;

        let outward_normal = [0.0, 0.0, 0.1].into();
        rec.set_face_normal(r, &outward_normal);
        rec.mat = Some(self.mp.clone());
        rec.p = r.at(t);

        true
    }
}
