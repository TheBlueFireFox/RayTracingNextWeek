use std::{ops::Index, path::Path};

use image::{io::Reader, GenericImageView};

use crate::render::{Color, Image, Render};

#[derive(Debug)]
pub struct ImageHolder {
    pixels: Vec<Color>,
    height: usize,
    width: usize,
}

impl ImageHolder {
    fn new(pixels: Vec<Color>, height: usize, width: usize) -> Self {
        Self {
            pixels,
            height,
            width,
        }
    }

    /// Get a reference to the image holder's height.
    pub fn height(&self) -> usize {
        self.height
    }

    /// Get a mutable reference to the image holder's width.
    pub fn width(&self) -> usize {
        self.width
    }
}

impl Index<usize> for ImageHolder {
    type Output = [Color];

    fn index(&self, index: usize) -> &Self::Output {
        assert!(
            index < self.height,
            "Index out of bound error {} of {}",
            index,
            self.height
        );
        &self.pixels[(index * self.width)..((index + 1) * self.width)]
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

    let pixels = img
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

    Ok(ImageHolder::new(pixels, height, width))
}
