use indicatif::{ParallelProgressIterator, ProgressBar, ProgressIterator};

use rayon::prelude::*;

use ray_tracing::{
    camera::Camera,
    clamp,
    hittable::{HitRecord, Hittable, HittableList},
    ray::{Point, Ray, Vec3},
    render::Color,
};

use crate::scenes::{self, Worlds};

const WORLD: Worlds = Worlds::CornellBox;

const REPETITION: usize = 2;
const ASPECT_RATIO: f64 = 16.0 / 9.0;
const IMAGE_WIDTH: usize = 160 * 4;
const IMAGE_HEIGHT: usize = (IMAGE_WIDTH as f64 / ASPECT_RATIO) as usize;
const SAMPLES_PER_PIXEL: usize = 100;
const MAX_DEPTH: usize = 50;
const GAMMA: f64 = 2.0;

#[derive(Clone)]
pub struct Config {
    pub rep: usize,
    aspect_ratio: f64,
    image_width: usize,
    image_height: usize,
    pub samples_per_pixel: usize,
    pub max_depth: usize,
    pub gamma: f64,
    pub background: Color,
}

impl Config {
    /// Get a reference to the config's aspect ratio.
    pub fn aspect_ratio(&self) -> f64 {
        self.aspect_ratio
    }

    /// Get a reference to the config's image width.
    pub fn image_width(&self) -> usize {
        self.image_width
    }

    /// Get a reference to the config's image height.
    pub fn image_height(&self) -> usize {
        self.image_height
    }

    fn fix_image_height(&mut self) {
        self.image_height = (self.image_width as f64 / self.aspect_ratio) as _;
    }

    /// Set the config's image width.
    pub fn set_image_width(&mut self, image_width: usize) {
        self.image_width = image_width;
        self.fix_image_height();
    }

    /// Set the config's aspect ratio.
    pub fn set_aspect_ratio(&mut self, aspect_ratio: f64) {
        self.aspect_ratio = aspect_ratio;
        self.fix_image_height();
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            rep: REPETITION,
            aspect_ratio: ASPECT_RATIO,
            image_width: IMAGE_WIDTH,
            image_height: IMAGE_HEIGHT,
            samples_per_pixel: SAMPLES_PER_PIXEL,
            max_depth: MAX_DEPTH,
            gamma: GAMMA,
            background: Color::zeros(),
        }
    }
}

fn ray_color<H: Hittable>(r: &Ray, background: &Color, world: &H, depth: usize) -> Color {
    // If we've exceeded the ray bounce limit, no more light is gathered.
    if depth == 0 {
        return Color::zeros();
    }

    let mut rec = HitRecord::default();

    // If the ray hits nothing, return the background color.
    if !world.hit(r, 0.001, f64::INFINITY, &mut rec) {
        return *background;
    }

    let mut scattered = Ray::new(Vec3::zeros(), Vec3::zeros());
    let mut attenuation = Color::zeros();

    let mat = rec
        .mat
        .as_ref()
        .expect("at this point the rec should have a inner material");

    let emitted = mat.emitted(rec.u, rec.v, &rec.p);

    if !mat.scatter(r, &rec, &mut attenuation, &mut scattered) {
        emitted
    } else {
        emitted + attenuation * ray_color(&scattered, background, world, depth - 1)
    }
}

fn irun<H: Hittable>(world: &H, conf: &Config, pb: ProgressBar, cam: &Camera) -> Vec<Color> {
    pb.set_position(0);

    // Render
    let calc = |o, l| ((o as f64) + ray_tracing::rand_range(0.0..1.0)) / (l - 1) as f64;

    // Divide the color by the number of samples
    let fix_scale: f64 = 1.0 / (conf.samples_per_pixel as f64);

    // gamma and clamping the values
    let fix_pixel_val = |v: f64| {
        let v = (fix_scale * v).powf(1.0 / conf.gamma);
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

    let data: Vec<_> = (0..conf.image_height)
        .into_par_iter()
        .rev()
        .progress_with(pb)
        .flat_map(|j| {
            (0..conf.image_width)
                .map(|i| {
                    let pixel_color = (0..conf.samples_per_pixel)
                        .map(|_| {
                            let v = calc(j, conf.image_height);
                            let u = calc(i, conf.image_width);
                            let r = cam.get_ray(u, v);
                            ray_color(&r, &conf.background, world, conf.max_depth)
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
    let mut tmp: Vec<_> = (0..conf.rep)
        .map(|_| Some(irun(world, &conf, pb_int.clone(), &cam)))
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
