mod ppm;
mod png;

use std::{io, path::Path};

use helpers::cvec;

use crate::helpers;

pub type Color = cvec::Color<f64>;

pub struct Image<'a> {
    pixels: &'a [Color],
    height: usize,
    width: usize,
}

impl<'a> Image<'a> {
    pub fn new(pixels: &'a [Color], height: usize, width: usize) -> Self {
        debug_assert!(pixels.len() == height * width, "incorrect pixel length");

        Self {
            pixels,
            height,
            width,
        }
    }
    pub fn pixels(&self) -> &'_ [Color] {
        self.pixels
    }

    pub fn height(&self) -> usize {
        self.height
    }

    pub fn width(&self) -> usize {
        self.width
    }
}

impl<'a> Render<'a> for Image<'a> {
    fn image(&self) -> &Image<'_> {
        self
    }
}

pub trait Render<'a> {
    fn image(&self) -> &Image<'_>;
}

#[allow(unused)]
pub enum FileFormat {
    PPM,
    PNG
}

pub fn save<'a, T: Render<'a>, P: AsRef<Path>>(image: T, path: P, format: FileFormat) -> Result<(), io::Error> {
    match format {
        FileFormat::PPM => ppm::save(image, path),
        FileFormat::PNG => png::save(image, path),
    }
}

