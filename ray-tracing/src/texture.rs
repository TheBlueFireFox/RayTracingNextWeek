use std::sync::Arc;

use crate::{perlin::Perlin, ray::Point, render::Color};

pub trait Texture: Send + Sync {
    fn value(&self, u: f64, v: f64, p: &Point) -> Color;
}

pub struct SolidColor {
    color_value: Color,
}

impl SolidColor {
    pub fn new(color_value: Color) -> Self {
        Self { color_value }
    }
}

impl Texture for SolidColor {
    fn value(&self, _u: f64, _v: f64, _p: &Point) -> Color {
        self.color_value
    }
}

pub struct CheckerTexture {
    odd: Arc<dyn Texture>,
    even: Arc<dyn Texture>,
}

impl CheckerTexture {
    pub fn with_color(even: Color, odd: Color) -> Self {
        Self::with_texture(SolidColor::new(even), SolidColor::new(odd))
    }

    pub fn with_texture<O, E>(even: O, odd: E) -> Self
    where
        O: Texture + 'static,
        E: Texture + 'static,
    {
        Self {
            odd: Arc::new(odd),
            even: Arc::new(even),
        }
    }
}

impl Texture for CheckerTexture {
    fn value(&self, u: f64, v: f64, p: &Point) -> Color {
        let calc = |v: f64| f64::sin(10.0 * v);
        let sines = calc(p.x()) * calc(p.y()) * calc(p.z());
        if sines < 0.0 {
            self.odd.value(u, v, p)
        } else {
            self.even.value(u, v, p)
        }
    }
}

pub struct NoiseTexture {
    noise: Perlin,
    scale: f64,
}

impl NoiseTexture {
    pub fn new() -> Self {
        Self::with_scale(1.0)
    }

    pub fn with_scale(scale: f64) -> Self {
        Self {
            noise: Perlin::new(),
            scale,
        }
    }
}

impl Texture for NoiseTexture {
    fn value(&self, _u: f64, _v: f64, p: &Point) -> Color {
        Color::new(1.0, 1.0, 1.0)
            * 0.5
            * (1.0 + f64::sin(self.scale * p.z() + 10.0 * self.noise.turb(p)))
    }
}

impl Default for NoiseTexture {
    fn default() -> Self {
        Self::new()
    }
}
