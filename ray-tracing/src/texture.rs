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

#[derive(Default)]
pub struct NoiseTexture {
    noise: Perlin,
}

impl NoiseTexture {
    pub fn new() -> Self {
        Default::default()
    }
}

impl Texture for NoiseTexture {
    fn value(&self, _u: f64, _v: f64, p: &Point) -> Color {
        Color::new(1.0, 1.0, 1.0) * self.noise.noise(p)
    }
}
