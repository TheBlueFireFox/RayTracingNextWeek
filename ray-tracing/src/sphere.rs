use crate::{
    aabb::Aabb,
    cvec::{dot, Vec3},
    hittable::{HitRecord, Hittable},
    material::Mat,
    ray::{Point, Ray},
};

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
}

impl Hittable for Sphere {
    fn hit(&self, r: &Ray, t_min: f64, t_max: f64, rec: &mut HitRecord) -> bool {
        let oc = r.origin() - self.center;
        let a = r.direction().length_squared();
        let half_b = dot(oc, r.direction());
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
        rec.mat = Some(self.mat.clone());

        true
    }

    fn bounding_box(&self, time0: f64, time1: f64, output: &mut Aabb) -> bool {
        *output = Aabb::new(
            self.center - Vec3::new(self.radius, self.radius, self.radius),
            self.center + Vec3::new(self.radius, self.radius, self.radius),
        );
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
        let oc = r.origin() - self.center(r.time());
        let a = r.direction().length_squared();
        let half_b = dot(oc, r.direction());
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
            let vec_r = Vec3::new(self.radius, self.radius, self.radius);
            Aabb::new(cen - vec_r, cen + vec_r)
        };
        let box0 = calc(time0);
        let box1 = calc(time1);
        *output = Aabb::surrounding_box(&box0, &box1);
        true
    }
}
