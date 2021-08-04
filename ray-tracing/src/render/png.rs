use std::{error, path::Path};

use image::{Rgb, RgbImage};

use super::Render;

pub fn save<'a, T: Render<'a>, P: AsRef<Path>>(
    img: T,
    path: P,
) -> Result<(), Box<dyn error::Error>> {
    let img = img.image();
    let path = path.as_ref().to_string_lossy();
    let path = if path.ends_with(".png") {
        path.to_string()
    } else {
        format!("{}.png", path)
    };

    let mut res = RgbImage::new(img.width() as u32, img.height() as u32);

    for y in 0..img.height() {
        let yy = y as u32;
        for x in 0..img.width() {
            let xx = x as u32;

            let pixel = img.pixels()[y * img.width() + x];

            let raw_vals = [pixel.x(), pixel.y(), pixel.z()];
            let mut vals = [0u8; 3];
            for (i, &v) in raw_vals.iter().enumerate() {
                vals[i] = v as u8;
            }

            res.put_pixel(xx, yy, Rgb(vals));
        }
    }
    res.save(path)?;

    Ok(())
}
