use std::sync::Arc;

use crate::{
    aabb::Aabb,
    cvec::dot,
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
        let outward_normal = *outward_normal;
        self.front_face = dot(r.direction(), outward_normal) < 0.0;
        self.normal = if self.front_face {
            outward_normal
        } else {
            -outward_normal
        }
    }
}

pub trait Hittable: Send + Sync {
    fn bounding_box(&self, time0: f64, time1: f64, output: &mut Aabb) -> bool;
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
        let mut temp_rec = Default::default();
        let mut hit_anything = false;
        let mut closest_so_far = t_max;
        for obj in &self.objects {
            if obj.hit(r, t_min, closest_so_far, &mut temp_rec) {
                hit_anything = true;
                closest_so_far = temp_rec.t.clone();
                *rec = temp_rec.clone();
            }
        }

        hit_anything
    }

    fn bounding_box(&self, time0: f64, time1: f64, output: &mut Aabb) -> bool {
        if self.objects.len() == 0 {
            return false;
        }
        let mut temp_box = Aabb::default();
        let mut first_box = true;

        for obj in self.objects.iter() {
            if obj.bounding_box(time0, time1, &mut temp_box) {
                return false;
            }

            *output = if first_box {
                temp_box.clone()
            } else {
                Aabb::surrounding_box(&output, &temp_box)
            };

            first_box = false;
        }

        true
    }
}
