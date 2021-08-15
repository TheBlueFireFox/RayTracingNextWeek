use indicatif::{ProgressBar, ProgressIterator};

use ray_tracing::{camera::Camera, hittable::HittableList, ray::Point, render::Color, Config};

use crate::scenes::{self, Worlds};

const WORLD: Worlds = Worlds::FinalScene;

pub const REPETITION: usize = 1;

pub struct WorldSettings {
    pub conf: Config,
    world: HittableList,
    cam: Camera,
}

pub fn setup() -> anyhow::Result<WorldSettings> {
    // World settigns
    let mut world_conf = Config::default();
    world_conf.background = [0.7, 0.8, 1.0].into();

    // Camera settings
    let mut lookfrom = Point::new(13.0, 2.0, 3.0);
    let mut lookat = Point::zeros();
    let vup = Point::new(0.0, 1.0, 0.0);
    let focus_dist = 10.0;
    let mut aperture = 0.0;
    let mut vfov = 20.0;

    // World
    let world = match WORLD {
        Worlds::RandomScene => {
            aperture = 0.1;
            scenes::random_scene()
        }
        Worlds::TwoSpheres => scenes::two_spheres(),
        Worlds::TwoPerlinSpheres => scenes::two_perlin_spheres(),
        Worlds::Earth => scenes::earth()?,
        Worlds::SimpleLight => {
            lookfrom = [26.0, 3.0, 6.0].into();
            lookat = [0.0, 2.0, 0.0].into();
            world_conf.samples_per_pixel = 400;
            world_conf.background = Color::zeros();
            scenes::simple_light()
        }
        Worlds::CornellBox => {
            world_conf.set_aspect_ratio(1.0);
            world_conf.set_image_width(600);
            world_conf.samples_per_pixel = 200;
            world_conf.background = Color::zeros();
            lookfrom = [278.0, 278.0, -800.0].into();
            lookat = [278.0, 278.0, 0.0].into();
            vfov = 40.0;
            scenes::cornell_box()
        }
        Worlds::CornellBoxSmoke => {
            world_conf.set_aspect_ratio(1.0);
            world_conf.set_image_width(600);
            world_conf.samples_per_pixel = 200;
            world_conf.background = Color::zeros();
            lookfrom = [278.0, 278.0, -800.0].into();
            lookat = [278.0, 278.0, 0.0].into();
            vfov = 40.0;
            scenes::cornell_box_smoke()
        }
        Worlds::FinalScene => {
            world_conf.set_aspect_ratio(1.0);
            world_conf.set_image_width(600);
            world_conf.samples_per_pixel = 100;
            world_conf.background = Color::zeros();
            lookfrom = [478.0, 278.0, -600.0].into();
            lookat = [278.0, 278.0, 0.0].into();
            vfov = 40.0;
            scenes::final_scene()?
        }
    };

    // Camera
    let cam = Camera::new(
        lookfrom,
        lookat,
        vup,
        vfov,
        world_conf.aspect_ratio(),
        aperture,
        focus_dist,
        0.0,
        1.0,
    );

    Ok(WorldSettings {
        conf: world_conf,
        world,
        cam,
    })
}

pub fn run(
    WorldSettings { conf, world, cam }: &WorldSettings,
    pb_run: ProgressBar,
    pb_int: ProgressBar,
) -> anyhow::Result<Vec<Color>> {
    pb_run.set_position(0);

    // run
    // SAFETY: the unwrap is safe here as we know
    // that there allways will be a result.
    let mut res = (0..REPETITION)
        .map(|_| ray_tracing::run(world, &conf, pb_int.clone(), &cam))
        .progress_with(pb_run)
        .reduce(|mut acc, v| {
            for (a, b) in acc.iter_mut().zip(v.iter()) {
                *a += *b;
            }
            acc
        })
        .unwrap();

    let len = REPETITION as f64;
    for val in res.iter_mut() {
        *val /= len;
    }

    Ok(res)
}
