use itertools::{self, izip};

use std::sync::Arc;

use crate::{
    aabb::Aabb,
    degrees_to_radians,
    material::Material,
    ray::{Point, Ray, Vec3},
};

#[derive(Clone, Default)]
pub struct HitRecord {
    pub p: Point,
    pub normal: Vec3,
    pub mat: Option<Arc<dyn Material>>,
    pub t: f64,
    pub u: f64,
    pub v: f64,
    pub front_face: bool,
}

impl HitRecord {
    pub fn set_face_normal(&mut self, r: &Ray, outward_normal: &Vec3) {
        self.front_face = Vec3::dot(r.direction(), outward_normal) < 0.0;

        let outward_normal = *outward_normal;
        self.normal = if self.front_face {
            outward_normal
        } else {
            -outward_normal
        }
    }
}

pub trait Hittable: Send + Sync {
    fn bounding_box(&self, time0: f64, time1: f64) -> Option<Aabb>;
    fn hit(&self, r: &Ray, t_min: f64, t_max: f64, rec: &mut HitRecord) -> bool;
}

pub type HittableObject = Arc<dyn Hittable>;

pub struct HittableList {
    objects: Vec<HittableObject>,
}

impl HittableList {
    pub fn new() -> Self {
        Self {
            objects: Vec::new(),
        }
    }

    pub fn with_capacity(cap: usize) -> Self {
        Self {
            objects: Vec::with_capacity(cap),
        }
    }

    /// Get a reference to the hittable list's objects.
    pub fn objects(&self) -> &[HittableObject] {
        &self.objects
    }

    pub fn clear(&mut self) {
        self.objects.clear();
    }

    pub fn add<HO: Hittable + 'static>(&mut self, object: HO) {
        self.objects.push(Arc::new(object))
    }
}

impl Hittable for HittableList {
    fn hit(&self, r: &Ray, t_min: f64, t_max: f64, rec: &mut HitRecord) -> bool {
        let mut temp_rec = HitRecord::default();
        let mut hit_anything = false;
        let mut closest_so_far = t_max;

        for obj in self.objects.iter() {
            if obj.hit(r, t_min, closest_so_far, &mut temp_rec) {
                hit_anything = true;
                closest_so_far = temp_rec.t;
                *rec = temp_rec.clone();
            }
        }

        hit_anything
    }

    fn bounding_box(&self, time0: f64, time1: f64) -> Option<Aabb> {
        if self.objects.len() == 0 {
            return None;
        }
        let mut first_box = true;
        let mut output = Aabb::default();

        for obj in self.objects.iter() {
            let temp_box = obj.bounding_box(time0, time1)?;

            output = if first_box {
                temp_box.clone()
            } else {
                Aabb::surrounding_box(&output, &temp_box)
            };

            first_box = false;
        }

        Some(output)
    }
}

pub struct Translate<H: Hittable> {
    ptr: H,
    offset: Vec3,
}

impl<H: Hittable> Translate<H> {
    pub fn new(ptr: H, offset: Vec3) -> Self {
        Self { ptr, offset }
    }
}

impl<H> Hittable for Translate<H>
where
    H: Hittable,
{
    fn bounding_box(&self, time0: f64, time1: f64) -> Option<Aabb> {
        let output = self.ptr.bounding_box(time0, time1)?;

        let output = Aabb::new(*output.min() + self.offset, *output.max() + self.offset);

        Some(output)
    }

    fn hit(&self, r: &Ray, t_min: f64, t_max: f64, rec: &mut HitRecord) -> bool {
        let moved = Ray::with_time(*r.origin() - self.offset, *r.direction(), r.time());

        if !self.ptr.hit(&moved, t_min, t_max, rec) {
            return false;
        }

        rec.p += self.offset;
        let normal = rec.normal;
        rec.set_face_normal(&moved, &normal);

        true
    }
}

pub struct RotateY<H: Hittable> {
    ptr: H,
    sin_theta: f64,
    cos_theta: f64,
    bbox: Option<Aabb>,
}

impl<H: Hittable> RotateY<H> {
    pub fn new(p: H, angle: f64) -> Self {
        let rads = degrees_to_radians(angle);
        let sin_theta = rads.sin();
        let cos_theta = rads.cos();

        let bbox = p.bounding_box(0.0, 1.0);

        Self {
            ptr: p,
            sin_theta,
            cos_theta,
            bbox: Self::calculate_bound(bbox, sin_theta, cos_theta),
        }
    }

    fn calculate_bound(bbox: Option<Aabb>, sin_theta: f64, cos_theta: f64) -> Option<Aabb> {
        let bbox = bbox?;
        let mut min = [f64::INFINITY; 3];
        let mut max = [-f64::INFINITY; 3];

        let calc = |iv, bv| {
            let iv = iv as f64;
            iv * bv + (1.0 - iv) * bv
        };

        for i in 0..2 {
            let x = calc(i, bbox.max().x());
            let cosx = cos_theta * x;
            let sinx = -sin_theta * x;

            for j in 0..2 {
                let y = calc(j, bbox.max().y());

                for k in 0..2 {
                    let z = calc(k, bbox.max().z());

                    let newx = cosx + sin_theta * z;
                    let newz = sinx + cos_theta * z;

                    let tester = [newx, y, newz];

                    for (min, max, tes) in izip!(&mut min, &mut max, tester) {
                        *min = f64::min(*min, tes);
                        *max = f64::max(*max, tes);
                    }
                }
            }
        }
        Some(Aabb::new(min.into(), max.into()))
    }
}

impl<H> Hittable for RotateY<H>
where
    H: Hittable,
{
    fn bounding_box(&self, _time0: f64, _time11: f64) -> Option<Aabb> {
        self.bbox.clone()
    }

    fn hit(&self, r: &Ray, t_min: f64, t_max: f64, rec: &mut HitRecord) -> bool {
        let oorg = r.origin();
        let odir = r.direction();

        let calc = |data: &Vec3| {
            let mut res = data.clone();

            res.data_mut()[0] = self.cos_theta * data.data()[0] - self.sin_theta * data.data()[2];
            res.data_mut()[2] = self.sin_theta * data.data()[0] + self.cos_theta * data.data()[2];

            res
        };

        let org = calc(oorg);
        let dir = calc(odir);

        let rotated = Ray::with_time(org, dir, r.time());

        if !self.ptr.hit(&rotated, t_min, t_max, rec) {
            return false;
        }

        let calc = |data: &Vec3| {
            let mut res = data.clone();

            res.data_mut()[0] = self.cos_theta * data.data()[0] + self.sin_theta * data.data()[2];
            res.data_mut()[2] = -self.sin_theta * data.data()[0] + self.cos_theta * data.data()[2];

            res
        };

        let p = calc(&rec.p);
        let normal = calc(&rec.normal);

        rec.p = p;
        rec.set_face_normal(&rotated, &normal);

        true
    }
}
