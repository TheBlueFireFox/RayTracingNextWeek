use indicatif::{ParallelProgressIterator, ProgressBar, ProgressIterator};

use rayon::prelude::*;

use ray_tracing::{
    camera::Camera,
    clamp,
    hittable::{HitRecord, Hittable},
    ray::{Point, Ray, Vec3},
    render::Color,
};

use crate::scenes::{self, Worlds};

pub const WORLD: Worlds = Worlds::Earth;

pub const REPETITION: usize = 2;
pub const ASPECT_RATIO: f64 = 16.0 / 9.0;
pub const IMAGE_WIDTH: usize = 160 * 4;
pub const IMAGE_HEIGHT: usize = (IMAGE_WIDTH as f64 / ASPECT_RATIO) as usize;
pub const SAMPLES_PER_PIXEL: usize = 100;
pub const MAX_DEPTH: usize = 50;
pub const GAMMA: f64 = 2.0;

fn ray_color<H: Hittable>(r: &Ray, world: &H) -> Color {
    ray_color_inner(r, world, MAX_DEPTH)
}

fn ray_color_inner<H: Hittable>(r: &Ray, world: &H, depth: usize) -> Color {
    if depth == 0 {
        return Color::zeros();
    }

    let mut rec = HitRecord::default();

    if world.hit(r, 0.001, f64::INFINITY, &mut rec) {
        let mut scattered = Ray::new(Vec3::zeros(), Vec3::zeros());
        let mut attenuation = Color::zeros();

        if let Some(ref mat) = rec.mat {
            if mat.scatter(r, &rec, &mut attenuation, &mut scattered) {
                return attenuation * ray_color_inner(&scattered, world, depth - 1);
            }

            return Color::zeros();
        }
    }
    let unit_direction = r.direction().unit_vector();
    let t = 0.5 * (unit_direction.y() + 1.0);
    (1.0 - t) * Color::ones() + t * Color::new(0.5, 0.7, 1.0)
}

fn irun<H: Hittable>(world: &H, pb: ProgressBar, cam: &Camera) -> Vec<Color> {
    pb.set_position(0);

    // Render

    let calc = |o, l| ((o as f64) + ray_tracing::rand_range(0.0..1.0)) / (l - 1) as f64;

    // Divide the color by the number of samples
    const FIX_SCALE : f64 = 1.0 / (SAMPLES_PER_PIXEL as f64);

    // gamma and clamping the values
    let fix_pixel_val = |v: f64| {
        let v = (FIX_SCALE * v).powf(1.0 / GAMMA);
        let c = clamp(v, 0.0, 0.999);
        256.0 * c
    };

    let fix_pixel = |p: Color| {
        let mut res = [p.x(), p.y(), p.z()];
        for v in res.as_mut() {
            *v = fix_pixel_val(*v);
        }
        res.into()
    };

    let data: Vec<_> = (0..IMAGE_HEIGHT)
        .into_par_iter()
        .rev()
        .progress_with(pb)
        .flat_map(|j| {
            (0..IMAGE_WIDTH)
                .map(|i| {
                    let pixel_color = (0..SAMPLES_PER_PIXEL)
                        .map(|_| {
                            let v = calc(j, IMAGE_HEIGHT);
                            let u = calc(i, IMAGE_WIDTH);
                            let r = cam.get_ray(u, v);
                            ray_color(&r, world)
                        })
                        .reduce(|acc, v| acc + v)
                        .expect("This iteration should never yield a None");
                    fix_pixel(pixel_color)
                })
                .collect::<Vec<_>>()
        })
        .collect();
    data
}

pub fn run(pb_run: ProgressBar, pb_int: ProgressBar) -> anyhow::Result<Vec<Color>> {
    pb_run.set_position(0);

    // Camera settings
    let lookfrom = Point::new(13.0, 2.0, 3.0);
    let lookat = Point::zeros();
    let vup = Point::new(0.0, 1.0, 0.0);
    let focus_dist = 10.0;
    let mut aperture = 0.0;
    let vfov = 20.0;

    // World
    let world = match WORLD {
        Worlds::RandomScene => {
            aperture = 0.1;
            scenes::random_scene()
        }
        Worlds::TwoSpheres => scenes::two_spheres(),
        Worlds::TwoPerlinSpheres => scenes::two_perlin_spheres(),
        Worlds::Earth => scenes::earth()?,
    };

    // Camera
    let cam = Camera::new(
        lookfrom,
        lookat,
        vup,
        vfov,
        ASPECT_RATIO,
        aperture,
        focus_dist,
        0.0,
        1.0,
    );

    // run
    let mut tmp: Vec<_> = (0..REPETITION)
        .map(|_| Some(irun(&world, pb_int.clone(), &cam)))
        .progress_with(pb_run)
        .collect();

    // prepare the solution
    // SAFETY: all the unwraps are safe here,
    // as above all the results get to be combined.
    let mut res = tmp[0].take().unwrap();

    for arr in tmp.iter().skip(1) {
        let arr = arr.as_ref().unwrap();
        for (i, val) in arr.iter().enumerate() {
            res[i] += *val;
        }
    }

    for val in res.iter_mut() {
        *val /= tmp.len() as f64;
    }

    Ok(res)
}
