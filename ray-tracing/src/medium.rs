use std::sync::Arc;

use crate::{
    aabb::Aabb,
    hittable::{HitRecord, Hittable},
    material::{Isotropic, Material},
    rand_range,
    ray::Ray,
    render::Color,
    texture::Texture,
};

pub struct Constant<Hittable> {
    boundary: Hittable,
    phase_function: Arc<dyn Material>,
    neg_inv_density: f64,
}

impl<H> Constant<H>
where
    H: Hittable,
{
    pub fn new(boundary: H, d: f64, c: &Color) -> Self {
        Self {
            boundary,
            neg_inv_density: -1.0 / d,
            phase_function: Arc::new(Isotropic::new(*c)),
        }
    }

    pub fn with_texture<T: Texture + 'static>(boundary: H, d: f64, a: T) -> Self {
        Self {
            boundary,
            neg_inv_density: -1.0 / d,
            phase_function: Arc::new(Isotropic::with_texture(a)),
        }
    }
}

impl<H> Hittable for Constant<H>
where
    H: Hittable,
{
    fn bounding_box(&self, time0: f64, time1: f64, output: &mut Aabb) -> bool {
        self.boundary.bounding_box(time0, time1, output)
    }

    fn hit(&self, r: &Ray, t_min: f64, t_max: f64, rec: &mut HitRecord) -> bool {
        // Print occasional samples when debugging. To enable, set enableDebug true.
        const ENABLE_DEBUG: bool = false;
        let debugging = ENABLE_DEBUG && rand_range(0.0..1.0) < 0.00001;

        let mut rec1 = HitRecord::default();
        let mut rec2 = HitRecord::default();

        if !self
            .boundary
            .hit(r, f64::NEG_INFINITY, f64::INFINITY, &mut rec1)
        {
            return false;
        }

        if !self
            .boundary
            .hit(r, rec1.t + 0.001, f64::INFINITY, &mut rec2)
        {
            return false;
        }

        if debugging {
            eprintln!("\nt_min={}, t_max={}", t_min, t_max);
        }

        rec1.t = f64::max(rec1.t, t_min);
        rec2.t = f64::min(rec2.t, t_max);

        if rec1.t >= rec2.t {
            return false;
        }

        rec1.t = f64::max(rec1.t, 0.0);

        let ray_length = r.direction().length();
        let distance_inside_boundary = (rec2.t - rec1.t) * ray_length;
        let hit_distance = self.neg_inv_density * f64::ln(rand_range(0.0..1.0));

        if hit_distance > distance_inside_boundary {
            return false;
        }

        rec.t = rec1.t + hit_distance / ray_length;
        rec.p = r.at(rec.t);

        if debugging {
            eprintln!(
                "\nhit_distance = {}\nrec.t = {}\nrec.p = {:?}",
                hit_distance, rec.t, rec.p
            );
        }

        rec.normal = [1.0, 0.0, 0.0].into();
        rec.front_face = true;
        rec.mat = Some(self.phase_function.clone());

        true
    }
}
