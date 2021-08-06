use std::path::Path;

use image::{io::Reader, GenericImageView};

use crate::render::{Color, Image, Render};

#[derive(Debug)]
pub struct ImageHolder {
    pixels: Vec<Color>,
    height: usize,
    width: usize,
}

impl ImageHolder {
    /// Get a reference to the image holder's pixel.
    pub fn pixels(&self) -> &[Color] {
        &self.pixels
    }

    /// Get a reference to the image holder's height.
    pub fn height(&self) -> usize {
        self.height
    }

    /// Get a mutable reference to the image holder's width.
    pub fn width(&self) -> usize {
        self.width
    }

    pub fn pixel(&self, x: usize, y: usize) -> &Color {
        &self.pixels()[y * self.height + x]
    }
}

impl Render<'_> for ImageHolder {
    fn image(&self) -> Image<'_> {
        Image::new(&self.pixels, self.height, self.width)
    }
}

pub fn read<P: AsRef<Path>>(path: P) -> anyhow::Result<ImageHolder> {
    let img = Reader::open(path)?.decode()?;

    let height = img.height() as _;
    let width = img.width() as _;

    let pixel = img
        .into_rgb8()
        .pixels()
        .map(|v| {
            let mut tmp = [0.0; 3];
            for (i, &val) in v.0.iter().enumerate() {
                tmp[i] = val as f64;
            }
            tmp.into()
        })
        .collect();

    Ok(ImageHolder {
        pixels: pixel,
        height,
        width,
    })
}
