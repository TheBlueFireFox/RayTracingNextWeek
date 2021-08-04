use std::{cell::RefCell, error, sync::Arc};

use ray_tracing::{
    hittable::HittableList,
    material::{Dielectric, Lambertian, Mat, Metal},
    rand_range,
    ray::{Point, Vec3},
    render::Color,
    sphere::{MovingSphere, Sphere},
    texture::{CheckerTexture, ImageTexture, NoiseTexture},
};

#[allow(unused)]
pub enum Worlds {
    TwoPerlinSpheres,
    TwoSpheres,
    RandomScene,
    Earth,
}

pub fn earth() -> Result<HittableList, Box<dyn error::Error + Send>> {
    let mut world = HittableList::new();

    let earth_texture = ImageTexture::new("assets/earthmap.jpg")?;
    let earth_surface = Lambertian::with_texture(earth_texture);
    let globe = Sphere::new([0.0, 0.0, 0.0].into(), 2.0, Arc::new(earth_surface));

    world.add(globe);

    Ok(world)
}

pub fn two_perlin_spheres() -> HittableList {
    let mut world = HittableList::with_capacity(2);
    let pertext = NoiseTexture::with_scale(4.0);
    let lam = Lambertian::with_texture(pertext);
    let lam = Arc::new(lam);

    let spheres = [((0.0, -1000.0, 0.0), 1000.0), ((0.0, 2.0, 0.0), 2.0)];

    for ((x, y, z), v) in spheres {
        let sphere = Sphere::new(Point::new(x, y, z), v, lam.clone());
        world.add(sphere);
    }

    world
}

pub fn two_spheres() -> HittableList {
    let mut world = HittableList::with_capacity(2);

    let checker = CheckerTexture::with_color(Color::new(0.2, 0.3, 0.1), Color::new(0.9, 0.9, 0.9));
    let checker = Lambertian::with_texture(checker);
    let checker = Arc::new(checker);
    let spheres = &[(0.0, -10.0, 0.0), (0.0, 10.0, 0.0)];

    for (x, y, z) in spheres.iter() {
        let sphere = Sphere::new(Point::new(*x, *y, *z), 10.0, checker.clone());
        world.add(sphere);
    }

    world
}

pub fn random_scene() -> HittableList {
    let world = HittableList::with_capacity(11 * 2 * 2);
    // RefCell is needed here as the world
    // is mutably borrowed by two functions
    // and the borrow checker cannot prove
    // that they will not be run at the same
    // time
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

    let make_lam = |p: Color| Arc::new(Lambertian::new(p));
    let make_met = |p: Color, f| Arc::new(Metal::new(p, f));
    let make_diel = |x| Arc::new(Dielectric::new(x));

    let make_lam_o = |(x, y, z)| make_lam(Color::new(x, y, z));
    let make_met_o = |(x, y, z), f| make_met(Color::new(x, y, z), f);

    // Add ground

    let checker = CheckerTexture::with_color(Color::new(0.2, 0.3, 0.1), Color::new(0.9, 0.9, 0.9));
    let checker = Lambertian::with_texture(checker);
    adder_point(Point::new(0.0, -1000.0, 0.0), 1000.0, Arc::new(checker));

    let calc = |v| (v as f64) + 0.9 * rand_range(0.0..1.0);

    for a in -11..11 {
        for b in -11..11 {
            let choose_mat = rand_range(0.0..1.0);
            let center = Point::new(calc(a), 0.2, calc(b));

            if (center - Point::new(4.0, 0.2, 0.0)).length() <= 0.9 {
                continue;
            }

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

    let a: &[(_, Mat)] = &[
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
