use indicatif::{ParallelProgressIterator, ProgressBar, ProgressIterator};
use std::{borrow::BorrowMut, cell::RefCell, rc::Rc, sync::Arc};

use rayon::iter::{IntoParallelRefIterator, ParallelIterator};

use ray_tracing::{camera::Camera, clamp, hittable::{Hittable, HittableList}, material::{Dielectric, Lambertian, Material, Metal}, rand_range, ray::{Point, Ray, Vec3}, render::Color, sphere::{MovingSphere, Sphere}, texture::CheckerTexture};

pub const REPETITION: usize = 2;
pub const ASPECT_RATIO: f64 = 16.0 / 9.0;
pub const IMAGE_WIDTH: usize = 400;
pub const IMAGE_HEIGHT: usize = (IMAGE_WIDTH as f64 / ASPECT_RATIO) as usize;
pub const SAMPLES_PER_PIXEL: usize = 100;
pub const MAX_DEPTH: usize = 50;
pub const GAMMA: f64 = 2.0;

fn random_scene() -> HittableList {
    let world = HittableList::with_capacity(11 * 2 * 2);
    let world = RefCell::new(Some(world));

    let adder_point = |p, r, m| {
        (world)
            .borrow_mut()
            .as_mut()
            .unwrap()
            .add(Sphere::new(p, r, m));
    };

    let adder_m_point = |c1, c2, t1, t2, r, m| {
        (world)
            .borrow_mut()
            .as_mut()
            .unwrap()
            .add(MovingSphere::new(c1, c2, t1, t2, r, m));
    };

    let make_lam_o = |(x, y, z)| Arc::new(Lambertian::new(Color::new(x, y, z)));
    let make_met_o = |(x, y, z), f| Arc::new(Metal::new(Color::new(x, y, z), f));
    let make_diel_o = |x| Arc::new(Dielectric::new(x));

    let ground_material = make_lam_o((0.5, 0.5, 0.5));
    adder_point(Point::new(0.0, -1000.0, 0.0), 1000.0, ground_material);

    let make_lam = |p: Color| {
        let data = p.data();
        make_lam_o((data[0], data[1], data[1]))
    };

    let make_met = |p: Color, f| {
        let data = p.data();
        make_met_o((data[0], data[1], data[1]), f)
    };

    let make_diel = |v| make_diel_o(v);
    let checker = CheckerTexture::with_color(Color::new(0.2, 0.3, 0.1), Color::new(0.9, 0.9, 0.9));
    let checker = Lambertian::with_texture(checker);
    adder_point(Point::new(0.0, -1000.0, 0.0), 1000.0, Arc::new(checker));
    let calc = |v| (v as f64) + 0.9 * rand_range(0.0..1.0);

    for a in -11..11 {
        for b in -11..11 {
            let choose_mat = rand_range(0.0..1.0);
            let center = Point::new(calc(a), 0.2, calc(b));

            if (center - Point::new(4.0, 0.2, 0.0)).length() > 0.9 {
                if choose_mat < 0.5 {
                    let albedo = Color::random_range(0.05..0.95) * Color::random_range(0.05..0.95);
                    let center2 = center + Vec3::new(0.0, rand_range(0.0..0.5), 0.0);
                    adder_m_point(center, center2, 0.0, 1.0, 0.2, make_lam(albedo));
                } else if choose_mat < 0.85 {
                    let albedo = Color::random_range(0.5..1.0);
                    let fuzz = rand_range(0.0..0.5);
                    adder_point(center, 0.2, make_met(albedo, fuzz));
                } else {
                    adder_point(center, 0.2, make_diel(1.5));
                }
            }
        }
    }

    let a: &[((f64, f64, f64), Arc<dyn Material>)] = &[
        ((0.0, 1.0, 0.0), make_diel(1.5)),
        (
            (-4.0, 1.0, 0.0),
            make_lam_o((130.0 / 256.0, 22.0 / 256.0, 22.0 / 256.0)),
        ),
        ((4.0, 1.0, 0.0), make_met_o((0.7, 0.6, 0.5), 0.0)),
    ];

    for m in a {
        let p = Point::new(m.0 .0, m.0 .1, m.0 .2);
        adder_point(p, 1.0, m.1.clone());
    }

    // unwrap is safe to work here
    // as the Option above is soley
    // used to be able to take the 
    // resulting world
    let res = world.borrow_mut().take().unwrap();
    res
}

fn ray_color<H: Hittable>(r: &Ray, world: &H, depth: usize) -> Color {
    if depth == 0 {
        return Color::new(0.0, 0.0, 0.0);
    }

    let mut rec = Default::default();

    if world.hit(r, 0.001, f64::INFINITY, &mut rec) {
        let mut scattered = Ray::new(Vec3::new(0.0, 0.0, 0.0), Vec3::new(0.0, 0.0, 0.0));
        let mut attenuation = Color::new(0.0, 0.0, 0.0);

        if let Some(ref mat) = rec.mat {
            if mat.scatter(r, &rec, &mut attenuation, &mut scattered) {
                return attenuation * ray_color(&scattered, world, depth - 1);
            }

            return Color::new(0.0, 0.0, 0.0);
        }
    }
    let unit_direction = r.direction().unit_vector();
    let t = 0.5 * (unit_direction.y() + 1.0);
    (1.0 - t) * Color::new(1.0, 1.0, 1.0) + t * Color::new(0.5, 0.7, 1.0)
}

fn irun<H: Hittable>(world: &H, pb: ProgressBar) -> Vec<Color> {
    pb.set_position(0);

    // Camera
    let lookfrom = Point::new(13.0, 2.0, 3.0);
    let lookat = Point::new(0.0, 0.0, 0.0);
    let vup = Point::new(0.0, 1.0, 0.0);
    let focus_dist = 10.0;
    let aperture = 0.1;
    let vfov = 20.0;

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

    // Render

    let calc = |o, l| ((o as f64) + ray_tracing::rand_range(0.0..1.0)) / (l - 1) as f64;

    // Divide the color by the number of samples
    let fix_scale = 1.0 / (SAMPLES_PER_PIXEL as f64);

    // gamma and clamping the values
    let fix_pixel_val = |v: f64| {
        let v = (fix_scale * v).powf(1.0 / GAMMA);
        let c = clamp(v, 0.0, 0.999);
        256.0 * c
    };

    let fix_pixel = |p: Color| {
        let r = fix_pixel_val(p.x());
        let g = fix_pixel_val(p.y());
        let b = fix_pixel_val(p.z());

        Color::new(r, g, b)
    };

    let outer: Vec<_> = (0..IMAGE_HEIGHT).rev().collect();

    let data: Vec<_> = outer
        .par_iter()
        .map(|&j| {
            (0..IMAGE_WIDTH)
                .map(|i| {
                    let mut pixel_color = Color::new(0.0, 0.0, 0.0);

                    for _ in 0..SAMPLES_PER_PIXEL {
                        let v = calc(j, IMAGE_HEIGHT);
                        let u = calc(i, IMAGE_WIDTH);
                        let r = cam.get_ray(u, v);
                        pixel_color += ray_color(&r, world, MAX_DEPTH);
                    }

                    fix_pixel(pixel_color)
                })
                .collect::<Vec<_>>()
        })
        .progress_with(pb)
        .flatten()
        .collect();
    data
}

pub fn run(pb_run: ProgressBar, pb_int: ProgressBar) -> Vec<Color> {
    pb_run.set_position(0);

    // World
    let world = random_scene();

    // run
    let mut tmp: Vec<_> = (0..REPETITION)
        .map(|_| Some(irun(&world, pb_int.clone())))
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

    res
}
