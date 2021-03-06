use std::{f64::consts::PI, sync::Arc};

use crate::{
    aabb::Aabb,
    hittable::{HitRecord, Hittable, HittableList},
    material::{Mat, Material},
    ray::{Point, Ray, Vec3},
};

#[derive(Clone)]
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
    fn hit(&self, r: &Ray, t_min: f64, t_max: f64) -> Option<HitRecord> {
        let oc = *r.origin() - self.center;
        let a = r.direction().length_squared();
        let half_b = Vec3::dot(&oc, &r.direction());
        let c = oc.length_squared() - self.radius * self.radius;

        let discriminant = half_b * half_b - a * c;

        if discriminant < 0.0 {
            return None;
        }

        let sqrtd = discriminant.sqrt();

        // Find the nearest root that lies in the acceptable range.
        let mut root = (-half_b - sqrtd) / a;
        if root < t_min || t_max < root {
            root = (-half_b + sqrtd) / a;
            if root < t_min || t_max < root {
                return None;
            }
        }
        let mut rec = HitRecord::default();

        rec.t = root;
        rec.p = r.at(root);

        let outward_normal = (rec.p - self.center) / self.radius;
        rec.set_face_normal(r, &outward_normal);
        let t = self.get_sphere_uv(&outward_normal);

        rec.u = t.0;
        rec.v = t.1;
        rec.mat = Some(self.mat.clone());

        Some(rec)
    }

    fn bounding_box(&self, _time0: f64, _time1: f64) -> Option<Aabb> {
        let v = [self.radius; 3].into();
        Some(Aabb::new(self.center - v, self.center + v))
    }
}

pub struct MovingSphere {
    center: (Point, Point),
    radius: f64,
    time: (f64, f64),
    mat: Mat,
}

impl MovingSphere {
    pub fn new(center: (Point, Point), time: (f64, f64), radius: f64, mat: Mat) -> Self {
        Self {
            center,
            time,
            radius,
            mat,
        }
    }

    pub fn center(&self, time: f64) -> Point {
        self.center.0
            + ((time - self.time.0) / (self.time.1 - self.time.0)) * (self.center.1 - self.center.0)
    }
}

impl Hittable for MovingSphere {
    fn hit(&self, r: &Ray, t_min: f64, t_max: f64) -> Option<HitRecord> {
        let oc = *r.origin() - self.center(r.time());
        let a = r.direction().length_squared();
        let half_b = Vec3::dot(&oc, &r.direction());
        let c = oc.length_squared() - self.radius * self.radius;

        let discriminant = half_b * half_b - a * c;

        if discriminant < 0.0 {
            return None;
        }

        let sqrtd = discriminant.sqrt();

        // Find the nearest root that lies in the acceptable range.
        let mut root = (-half_b - sqrtd) / a;
        if root < t_min || t_max < root {
            root = (-half_b + sqrtd) / a;
            if root < t_min || t_max < root {
                return None;
            }
        }

        let mut rec = HitRecord::default();

        rec.t = root;
        rec.p = r.at(rec.t);
        let outward_normal = (rec.p - self.center(r.time())) / self.radius;
        rec.set_face_normal(r, &outward_normal);
        rec.mat = Some(self.mat.clone());

        Some(rec)
    }

    fn bounding_box(&self, time0: f64, time1: f64) -> Option<Aabb> {
        let calc = |time| {
            let cen = self.center(time);
            let vec_r = [self.radius; 3].into();
            Aabb::new(cen - vec_r, cen + vec_r)
        };
        let box0 = calc(time0);
        let box1 = calc(time1);

        Some(Aabb::surrounding_box(&box0, &box1))
    }
}

pub struct Cube {
    box_min: Point,
    box_max: Point,
    sides: HittableList,
}

impl Cube {
    pub fn new<M: Material + 'static>(p0: &Point, p1: &Point, mat: M) -> Self {
        let mut sides = HittableList::with_capacity(6);

        let mat = Arc::new(mat);

        for p in [p0, p1] {
            sides.add(rect::XY::with_arc(
                mat.clone(),
                (p0.x(), p1.x()),
                (p0.y(), p1.y()),
                p.z(),
            ));

            sides.add(rect::XZ::with_arc(
                mat.clone(),
                (p0.x(), p1.x()),
                (p0.z(), p1.z()),
                p.y(),
            ));

            sides.add(rect::YZ::with_arc(
                mat.clone(),
                (p0.y(), p1.y()),
                (p0.z(), p1.z()),
                p.x(),
            ));
        }

        Self {
            box_min: p0.clone(),
            box_max: p1.clone(),
            sides,
        }
    }
}

impl Hittable for Cube {
    fn bounding_box(&self, _time0: f64, _time1: f64) -> Option<Aabb> {
        Some(Aabb::new(self.box_min, self.box_max))
    }

    fn hit(&self, r: &Ray, t_min: f64, t_max: f64) -> Option<HitRecord> {
        self.sides.hit(r, t_min, t_max)
    }
}

pub mod rect {

    use super::*;

    pub struct XY<M> {
        mp: Arc<M>,
        x: (f64, f64),
        y: (f64, f64),
        k: f64,
    }

    impl<M: Material + 'static> XY<M> {
        pub fn new(mp: M, x: (f64, f64), y: (f64, f64), k: f64) -> Self {
            Self::with_arc(Arc::new(mp), x, y, k)
        }

        pub fn with_arc(mp: Arc<M>, x: (f64, f64), y: (f64, f64), k: f64) -> Self {
            Self { mp, x, y, k }
        }
    }

    impl<M: Material + 'static> Hittable for XY<M> {
        fn bounding_box(&self, _time0: f64, _time1: f64) -> Option<Aabb> {
            // The bounding box must have non-zero width in each dimension, so pad the Z
            // dimension a small amount.
            Some(Aabb::new(
                [self.x.0, self.y.0, self.k - 0.0001].into(),
                [self.x.1, self.y.1, self.k + 0.0001].into(),
            ))
        }

        fn hit(&self, r: &Ray, t_min: f64, t_max: f64) -> Option<HitRecord> {
            let org = r.origin();
            let dir = r.direction();

            let t = (self.k - org.z()) / dir.z();

            let bounds = |val, min, max| val < min || max < val;

            if bounds(t, t_min, t_max) {
                return None;
            }

            let x = org.x() + t * dir.x();
            let y = org.y() + t * dir.y();

            if bounds(x, self.x.0, self.x.1) || bounds(y, self.y.0, self.y.1) {
                return None;
            }

            let mut rec = HitRecord::default();

            let calc = |a, b: (_, _)| (a - b.0) / (b.1 - b.0);

            rec.u = calc(x, self.x);
            rec.v = calc(y, self.y);
            rec.t = t;

            let outward_normal = [0.0, 0.0, 1.0].into();
            rec.set_face_normal(r, &outward_normal);
            rec.mat = Some(self.mp.clone());
            rec.p = r.at(t);

            Some(rec)
        }
    }

    pub struct XZ<M: Material> {
        mp: Arc<M>,
        x: (f64, f64),
        z: (f64, f64),
        k: f64,
    }

    impl<M: Material + 'static> XZ<M> {
        pub fn new(mp: M, x: (f64, f64), z: (f64, f64), k: f64) -> Self {
            Self::with_arc(Arc::new(mp), x, z, k)
        }

        pub fn with_arc(mp: Arc<M>, x: (f64, f64), z: (f64, f64), k: f64) -> Self {
            Self { mp, x, z, k }
        }
    }

    impl<M: Material + 'static> Hittable for XZ<M> {
        fn bounding_box(&self, _time0: f64, _time1: f64) -> Option<Aabb> {
            // The bounding box must have non-zero width in each dimension, so pad the Y
            // dimension a small amount.
            Some(Aabb::new(
                [self.x.0, self.k - 0.0001, self.z.0].into(),
                [self.x.1, self.k + 0.0001, self.z.1].into(),
            ))
        }

        fn hit(&self, r: &Ray, t_min: f64, t_max: f64) -> Option<HitRecord> {
            let org = r.origin();
            let dir = r.direction();
            let t = (self.k - org.y()) / dir.y();

            let bounds = |val, min, max| val < min || max < val;

            if bounds(t, t_min, t_max) {
                return None;
            }

            let x = org.x() + t * dir.x();
            let z = org.z() + t * dir.z();

            if bounds(x, self.x.0, self.x.1) || bounds(z, self.z.0, self.z.1) {
                return None;
            }

            let calc = |a, b: (_, _)| (a - b.0) / (b.1 - b.0);

            let mut rec = HitRecord::default();

            rec.u = calc(x, self.x);
            rec.v = calc(z, self.z);
            rec.t = t;

            let outward_normal = [0.0, 1.0, 0.0].into();
            rec.set_face_normal(r, &outward_normal);
            rec.mat = Some(self.mp.clone());
            rec.p = r.at(t);

            Some(rec)
        }
    }

    pub struct YZ<M: Material + 'static> {
        mp: Arc<M>,
        y: (f64, f64),
        z: (f64, f64),
        k: f64,
    }

    impl<M: Material + 'static> YZ<M> {
        pub fn new(mp: M, y: (f64, f64), z: (f64, f64), k: f64) -> Self {
            Self::with_arc(Arc::new(mp), y, z, k)
        }
        pub fn with_arc(mp: Arc<M>, y: (f64, f64), z: (f64, f64), k: f64) -> Self {
            Self { mp, y, z, k }
        }
    }

    impl<M: Material + 'static> Hittable for YZ<M> {
        fn bounding_box(&self, _time0: f64, _time1: f64) -> Option<Aabb> {
            // The bounding box must have non-zero width in each dimension, so pad the X
            // dimension a small amount.
            Some(Aabb::new(
                [self.k - 0.0001, self.y.0, self.z.0].into(),
                [self.k + 0.0001, self.y.1, self.z.1].into(),
            ))
        }

        fn hit(&self, r: &Ray, t_min: f64, t_max: f64) -> Option<HitRecord> {
            let org = r.origin();
            let dir = r.direction();
            let t = (self.k - org.x()) / dir.x();

            let bounds = |val, min, max| val < min || max < val;

            if bounds(t, t_min, t_max) {
                return None;
            }

            let y = org.y() + t * dir.y();
            let z = org.z() + t * dir.z();

            if bounds(y, self.y.0, self.y.1) || bounds(z, self.z.0, self.z.1) {
                return None;
            }

            let calc = |a, b: (_, _)| (a - b.0) / (b.1 - b.0);

            let mut rec = HitRecord::default();

            rec.u = calc(y, self.y);
            rec.v = calc(z, self.z);
            rec.t = t;

            let outward_normal = [1.0, 0.0, 0.0].into();
            rec.set_face_normal(r, &outward_normal);
            rec.mat = Some(self.mp.clone());
            rec.p = r.at(t);

            Some(rec)
        }
    }
}
