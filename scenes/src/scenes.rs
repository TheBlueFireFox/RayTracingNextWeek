use std::{cell::RefCell, sync::Arc};

use ray_tracing::{
    bvh::BvhNode,
    hittable::{HittableList, RotateY, Translate},
    material::{Dielectric, DiffuseLight, Lambertian, Mat, Metal},
    medium,
    objects::{rect, Cube, MovingSphere, Sphere},
    rand_range,
    ray::{Point, Vec3},
    render::Color,
    texture::{CheckerTexture, ImageTexture, NoiseTexture},
};

#[allow(unused)]
pub enum Worlds {
    RandomScene,
    TwoPerlinSpheres,
    TwoSpheres,
    Earth,
    SimpleLight,
    CornellBox,
    CornellBoxSmoke,
    FinalScene,
}

pub fn final_scene() -> anyhow::Result<HittableList> {
    let mut objects = HittableList::new();

    const CUBE_PER_SIDE: usize = 20;

    let mut cubes = HittableList::with_capacity(CUBE_PER_SIDE * CUBE_PER_SIDE);
    let ground = Lambertian::new([0.48, 0.83, 0.53].into());

    for i in 0..CUBE_PER_SIDE {
        for j in 0..CUBE_PER_SIDE {
            let w = 100.0;
            let x0 = -1000.0 + (i as f64) * w;
            let y0 = 0.0;
            let z0 = -1000.0 + (j as f64) * w;
            let x1 = x0 + w;
            let y1 = rand_range(1.0..101.0);
            let z1 = z0 + w;

            let cube = Cube::new(&[x0, y0, z0].into(), &[x1, y1, z1].into(), ground.clone());
            cubes.add(cube);
        }
    }

    objects.add(BvhNode::from_hittable_list(&cubes, 0.0, 1.0));

    let light = DiffuseLight::new([7.0, 7.0, 7.0].into());
    objects.add(rect::XZ::new(light, (123.0, 423.0), (147.0, 412.0), 554.0));

    let center1 = [400.0, 400.0, 200.0].into();
    let center2 = center1 + [30.0, 0.0, 0.0].into();
    let mam = Lambertian::new([0.7, 0.3, 0.1].into());
    objects.add(MovingSphere::new(
        (center1, center2),
        (0.0, 1.0),
        50.0,
        Arc::new(mam),
    ));

    for (c, m) in [
        ([260.0, 150.0, 45.0], Arc::new(Dielectric::new(1.5)) as Mat),
        (
            [0.0, 150.0, 145.0],
            Arc::new(Metal::new([0.8, 0.8, 0.9].into(), 1.0)),
        ),
    ] {
        objects.add(Sphere::new(c.into(), 50.0, m));
    }

    let glass = Arc::new(Dielectric::new(1.5));
    let boundary = Sphere::new([360.0, 150.0, 145.0].into(), 70.0, glass.clone());
    objects.add(boundary.clone());
    objects.add(medium::Constant::new(
        boundary,
        0.2,
        &[0.2, 0.4, 0.9].into(),
    ));

    let boundary = Sphere::new(Point::zeros(), 5000.0, glass);
    objects.add(medium::Constant::new(
        boundary,
        0.0001,
        &[1.0, 1.0, 1.0].into(),
    ));

    let emat = Lambertian::with_texture(ImageTexture::new("assets/earthmap.jpg")?);
    objects.add(Sphere::new(
        [400.0, 200.0, 400.0].into(),
        100.0,
        Arc::new(emat),
    ));

    let pertext = NoiseTexture::with_scale(0.1);
    objects.add(Sphere::new(
        [200.0, 280.0, 300.0].into(),
        80.0,
        Arc::new(Lambertian::with_texture(pertext)),
    ));

    let mut cubes = HittableList::new();
    let white = Arc::new(Lambertian::new([0.73, 0.73, 0.73].into()));
    const NS: usize = 1000;

    for _ in 0..NS {
        let sphere = Sphere::new(Point::random_range(0.0..165.0), 10.0, white.clone());
        cubes.add(sphere);
    }

    objects.add(Translate::new(
        RotateY::new(BvhNode::from_hittable_list(&cubes, 0.0, 1.0), 15.0),
        [-100.0, 270.0, 395.0].into(),
    ));

    Ok(objects)
}

pub fn cornell_box() -> HittableList {
    let mut world = HittableList::new();

    // Setup colors
    let red = Lambertian::new([0.65, 0.05, 0.05].into());
    let white = Lambertian::new([0.73, 0.73, 0.73].into());
    let green = Lambertian::new([0.12, 0.45, 0.15].into());
    let light = DiffuseLight::new([15.0, 15.0, 15.0].into());

    // Walls
    for (k, mp) in [(555.0, green), (0.0, red)] {
        let yz = rect::YZ::new(mp, (0.0, 555.0), (0.0, 555.0), k);
        world.add(yz);
    }

    world.add(rect::XZ::new(light, (213.0, 343.0), (227.0, 332.0), 554.0));

    for k in [555.0, 0.0] {
        let xz = rect::XZ::new(white.clone(), (0.0, 555.0), (0.0, 555.0), k);
        world.add(xz);
    }

    world.add(rect::XY::new(
        white.clone(),
        (0.0, 555.0),
        (0.0, 555.0),
        555.0,
    ));

    // Cubes in the middle
    let cubes = [
        ([165.0, 333.0, 165.0], 15.0, [265.0, 0.0, 295.0]),
        ([165.0, 165.0, 165.0], -18.0, [130.0, 0.0, 65.0]),
    ];
    for (pos, ang, tra) in cubes {
        let cube = Cube::new(&Point::zeros(), &pos.into(), white.clone());
        let cube = RotateY::new(cube, ang);
        let cube = Translate::new(cube, tra.into());
        world.add(cube);
    }

    world
}

pub fn cornell_box_smoke() -> HittableList {
    let mut world = HittableList::new();

    // Setup colors
    let red = Lambertian::new([0.65, 0.05, 0.05].into());
    let white = Lambertian::new([0.73, 0.73, 0.73].into());
    let green = Lambertian::new([0.12, 0.45, 0.15].into());
    let light = DiffuseLight::new([15.0, 15.0, 15.0].into());

    // Walls
    for (k, mp) in [(555.0, green), (0.0, red)] {
        let yz = rect::YZ::new(mp, (0.0, 555.0), (0.0, 555.0), k);
        world.add(yz);
    }

    world.add(rect::XZ::new(light, (213.0, 343.0), (227.0, 332.0), 554.0));

    for k in [555.0, 0.0] {
        let xz = rect::XZ::new(white.clone(), (0.0, 555.0), (0.0, 555.0), k);
        world.add(xz);
    }

    world.add(rect::XY::new(
        white.clone(),
        (0.0, 555.0),
        (0.0, 555.0),
        555.0,
    ));

    // Cubes in the middle
    let cubes = [
        (
            [165.0, 333.0, 165.0],
            15.0,
            [265.0, 0.0, 295.0],
            Color::zeros(),
        ),
        (
            [165.0, 165.0, 165.0],
            -18.0,
            [130.0, 0.0, 65.0],
            Color::ones(),
        ),
    ];
    for (pos, ang, tra, col) in cubes {
        let cube = Cube::new(&Point::zeros(), &pos.into(), white.clone());
        let cube = RotateY::new(cube, ang);
        let cube = Translate::new(cube, tra.into());
        let cube = medium::Constant::new(cube, 0.01, &col);
        world.add(cube);
    }

    world
}

pub fn simple_light() -> HittableList {
    let mut world = HittableList::new();

    let pertext = NoiseTexture::with_scale(4.0);
    let lam = Lambertian::with_texture(pertext);
    let lam = Arc::new(lam);
    let objs = [
        ([0.0, -1000.0, 0.0].into(), 1000.0),
        ([0.0, 2.0, 0.0].into(), 2.0),
    ];
    for s in objs {
        world.add(Sphere::new(s.0, s.1, lam.clone()));
    }

    let difflight = DiffuseLight::new([4.0, 4.0, 4.0].into());
    let rect = rect::XY::new(difflight, (3.0, 5.0), (1.0, 3.0), -2.0);
    world.add(rect);

    world
}

pub fn earth() -> anyhow::Result<HittableList> {
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
            .add(MovingSphere::new((c1, c2), (t1, t2), r, m));
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
