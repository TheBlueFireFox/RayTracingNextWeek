use cfg_if::cfg_if;
use rayon::prelude::*;

use crate::{
    camera::Camera,
    clamp,
    hittable::{HitRecord, Hittable},
    ray::{Ray, Vec3},
    render::Color,
};

const ASPECT_RATIO: f64 = 16.0 / 9.0;
const IMAGE_WIDTH: usize = 160 * 4;
const IMAGE_HEIGHT: usize = (IMAGE_WIDTH as f64 / ASPECT_RATIO) as usize;
const SAMPLES_PER_PIXEL: usize = 100;
const MAX_DEPTH: usize = 50;
const GAMMA: f64 = 2.0;

#[derive(Clone)]
pub struct Config {
    aspect_ratio: f64,
    image_width: usize,
    image_height: usize,
    samples_per_pixel: usize,
    max_depth: usize,
    gamma: f64,
    background: Color,
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
        self.image_height = (self.image_width() as f64 / self.aspect_ratio()) as _;
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

    /// Get a reference to the config's samples per pixel.
    pub fn samples_per_pixel(&self) -> &usize {
        &self.samples_per_pixel
    }

    /// Get a reference to the config's max depth.
    pub fn max_depth(&self) -> &usize {
        &self.max_depth
    }

    /// Get a reference to the config's gamma.
    pub fn gamma(&self) -> &f64 {
        &self.gamma
    }

    /// Get a reference to the config's background.
    pub fn background(&self) -> &Color {
        &self.background
    }

    /// Set the config's samples per pixel.
    pub fn set_samples_per_pixel(&mut self, samples_per_pixel: usize) {
        self.samples_per_pixel = samples_per_pixel;
    }

    /// Set the config's background.
    pub fn set_background(&mut self, background: Color) {
        self.background = background;
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
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

#[cfg(feature = "progressbar")]
use indicatif::{ParallelProgressIterator, ProgressBar};

struct Runner<'hit, 'conf, 'cam, H: Hittable> {
    world: &'hit H,
    conf: &'conf Config,
    cam: &'cam Camera,
    #[cfg(feature = "progressbar")]
    pb: ProgressBar,
}

impl<'hit, 'conf, 'cam, H: Hittable> Runner<'hit, 'conf, 'cam, H> {
    fn irun(&self) -> Vec<Color> {
        cfg_if! {
            if #[cfg(feature = "progressbar")] {
               self.pb.set_position(0);
            }
        }
        // Render
        let calc = |o, l| ((o as f64) + crate::rand_range(0.0..1.0)) / (l - 1) as f64;

        // Divide the color by the number of samples
        let fix_scale: f64 = 1.0 / (self.conf.samples_per_pixel as f64);

        // gamma and clamping the values
        let fix_pixel_val = |v: f64| {
            let v = (fix_scale * v).powf(1.0 / self.conf.gamma);
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

        cfg_if! {
            if #[cfg(feature = "progressbar")] {
                self.pb.println("processing");
            }
        }

        let inner = |&j| {
            (0..self.conf.image_width)
                .map(|i| {
                    let pixel_color = (0..self.conf.samples_per_pixel)
                        .map(|_| {
                            let v = calc(j, self.conf.image_height);
                            let u = calc(i, self.conf.image_width);
                            let r = self.cam.get_ray(u, v);
                            ray_color(&r, &self.conf.background, self.world, self.conf.max_depth)
                        })
                        .reduce(|acc, v| acc + v)
                        .expect("This iteration should never yield a None");
                    fix_pixel(pixel_color)
                })
                .collect::<Vec<_>>()
        };

        let range = (0..self.conf.image_height).collect::<Vec<_>>();
        let data = range.par_iter().rev();

        cfg_if! {
            if #[cfg(feature = "progressbar")] {
                let data = data.progress_with(self.pb.clone());
            } 
        }

        data.flat_map(inner).collect()
    }
}

#[cfg(not(feature = "progressbar"))]
pub fn run<H: Hittable>(world: &H, conf: &Config, cam: &Camera) -> Vec<Color> {
    Runner { world, conf, cam }.irun()
}

#[cfg(feature = "progressbar")]
pub fn run<H: Hittable>(world: &H, conf: &Config, pb: ProgressBar, cam: &Camera) -> Vec<Color> {
    Runner {
        world,
        conf,
        pb,
        cam,
    }
    .irun()
}
