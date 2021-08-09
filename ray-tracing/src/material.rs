use std::sync::Arc;

use crate::{
    hittable::HitRecord,
    ray::{Point, Ray, Vec3},
    render::Color,
    rtweekend,
    texture::{SolidColor, Texture},
};

pub type Mat = Arc<dyn Material>;

pub trait Material: Send + Sync {
    fn scatter(
        &self,
        r_in: &Ray,
        rec: &HitRecord,
        attenuation: &mut Color,
        scattered: &mut Ray,
    ) -> bool;

    fn emitted(&self, _u: f64, _v: f64, _p: &Point) -> Color {
        Color::zeros()
    }
}

#[derive(Clone)]
pub struct Lambertian {
    albedo: Arc<dyn Texture>,
}

impl Lambertian {
    pub fn new(a: Color) -> Self {
        Self::with_texture(SolidColor::new(a))
    }

    pub fn with_texture<T>(a: T) -> Self
    where
        T: Texture + 'static,
    {
        Self {
            albedo: Arc::new(a),
        }
    }
}

impl Material for Lambertian {
    fn scatter(
        &self,
        r_in: &Ray,
        rec: &HitRecord,
        attenuation: &mut Color,
        scattered: &mut Ray,
    ) -> bool {
        let mut scatter_direction = rec.normal + Vec3::random_unit_vector();
        if scatter_direction.near_zero() {
            scatter_direction = rec.normal;
        }
        *scattered = Ray::with_time(rec.p, scatter_direction, r_in.time());
        *attenuation = self.albedo.value(rec.u, rec.v, &rec.p);

        true
    }
}

#[derive(Clone)]
pub struct Metal {
    pub albedo: Color,
    pub fuzz: f64,
}

impl Metal {
    pub fn new(a: Color, f: f64) -> Self {
        Self { albedo: a, fuzz: f }
    }
}

impl Material for Metal {
    fn scatter(
        &self,
        r_in: &Ray,
        rec: &HitRecord,
        attenuation: &mut Color,
        scattered: &mut Ray,
    ) -> bool {
        let reflected = Vec3::reflect(&r_in.direction().unit_vector(), &rec.normal);
        *scattered = Ray::with_time(
            rec.p,
            reflected + self.fuzz * Vec3::random_in_unit_sphere(),
            r_in.time(),
        );
        *attenuation = self.albedo;

        Vec3::dot(&scattered.direction(), &rec.normal) > 0.0
    }
}

#[derive(Clone)]
pub struct Dielectric {
    ir: f64,
}

impl Dielectric {
    pub fn new(ir: f64) -> Self {
        Self { ir }
    }

    fn reflectance(cosine: f64, ref_idx: f64) -> f64 {
        // Use Schlick's approximation for reflextance.
        let mut r0 = (1.0 - ref_idx) / (1.0 + ref_idx);
        r0 = r0.powf(2.0);
        r0 + (1.0 - r0) * ((1.0 - cosine).powf(5.0))
    }
}

impl Material for Dielectric {
    fn scatter(
        &self,
        r_in: &Ray,
        rec: &HitRecord,
        attenuation: &mut Color,
        scattered: &mut Ray,
    ) -> bool {
        *attenuation = Color::new(1.0, 1.0, 1.0);
        let refraction_ratio = if rec.front_face {
            (1.0) / self.ir
        } else {
            self.ir
        };

        let unit_direction = r_in.direction().unit_vector();
        let cos_theta = Vec3::dot(&(-unit_direction), &rec.normal).min(1.0);
        let sin_theta = (1.0 - cos_theta * cos_theta).sqrt();

        let cannot_refract = refraction_ratio * sin_theta > 1.0;

        let direction = if cannot_refract
            || Self::reflectance(cos_theta, refraction_ratio) > rtweekend::rand_range(0.0..1.0)
        {
            Vec3::reflect(&unit_direction, &rec.normal)
        } else {
            Vec3::refract(&unit_direction, &rec.normal, refraction_ratio)
        };

        *scattered = Ray::with_time(rec.p, direction, r_in.time());

        true
    }
}

#[derive(Clone)]
pub struct DiffuseLight {
    emit: Arc<dyn Texture>,
}

impl DiffuseLight {
    pub fn new(c: Color) -> Self {
        let solid = SolidColor::new(c);
        Self::with_texture(solid)
    }

    pub fn with_texture<T: Texture + 'static>(emit: T) -> Self {
        let iemit = Arc::new(emit);
        Self { emit: iemit }
    }
}

impl Material for DiffuseLight {
    fn scatter(
        &self,
        _r_in: &Ray,
        _rec: &HitRecord,
        _attenuation: &mut Color,
        _scattered: &mut Ray,
    ) -> bool {
        false
    }

    fn emitted(&self, u: f64, v: f64, p: &Point) -> Color {
        self.emit.value(u, v, p)
    }
}
