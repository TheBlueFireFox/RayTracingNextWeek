use crate::render::Render;
use std::{error, fmt::Write, fs, path::Path};

pub fn save<'a, T: Render<'a>, P: AsRef<Path>>(
    image: T,
    path: P,
) -> Result<(), Box<dyn error::Error>> {
    let img = image.image();

    const LINE_LENGTH: usize = 3 * 3 + 3; // => 255 255 255\n

    let text_length = {
        let mut len = 3; // P3\n
        len += ((img.width() as f64).log10() + 1.0) as usize;
        len += 1;
        len += ((img.height() as f64).log10() + 1.0) as usize;
        len += 5; // => \n255\n
        len += img.height() + img.height() * img.width() * LINE_LENGTH;
        len
    };

    let mut s = String::with_capacity(text_length);

    // no error possible as per docs
    let _ = write!(s, "P3\n{} {}\n255\n", img.width(), img.height());

    for p in img.pixels() {
        let r = p.x() as u8;
        let g = p.y() as u8;
        let b = p.z() as u8;

        // no error possible as per docs
        let _ = write!(s, "{} {} {}\n", r, g, b);
    }

    let path = path.as_ref().to_string_lossy();

    // write file content
    fs::write(
        if path.ends_with(".ppm") {
            path.to_string()
        } else {
            format!("{}.ppm", path)
        },
        &s,
    )?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::{
        cmp::min,
        io::{self, Read},
    };
    use tempfile;

    use crate::render::{ppm::save, Color, Image};

    #[test]
    fn test_rainbow() -> Result<(), io::Error> {
        const IMAGE_WIDTH: usize = 256;
        const IMAGE_HEIGHT: usize = 256;

        let mut px = vec![Color::new(0.0, 0.0, 0.0); IMAGE_HEIGHT * IMAGE_WIDTH];

        let mut res = format!("P3\n{} {}\n255\n", IMAGE_WIDTH, IMAGE_HEIGHT);

        let per = |v, c| ((v as f64) / ((c - 1) as f64));

        let rcon = |v| 255.999 * v;
        let con = |v| rcon(v) as u8;

        for j in 0..IMAGE_HEIGHT {
            for i in 0..IMAGE_WIDTH {
                let r = con(per(i, IMAGE_WIDTH));
                let g = con(per(255 - j, IMAGE_HEIGHT));
                let b = con(0.25);

                px[j * IMAGE_HEIGHT + i] = Color::new(r as f64, g as f64, b as f64);

                res.push_str(&format!("{} {} {}\n", r, g, b));
            }
        }

        let img = Image::new(&px, IMAGE_HEIGHT, IMAGE_WIDTH);

        let tmp_file1 = tempfile::Builder::new().suffix(".ppm").tempfile()?;
        let mut tmp_file2 = tmp_file1.reopen()?;

        // write the file back
        save(img, tmp_file1)?;

        let mut buf = String::new();
        tmp_file2.read_to_string(&mut buf)?;

        let buf = buf.trim();
        let res = res.trim();

        assert_eq!(buf.len(), res.len());

        const SIZE: usize = 100;

        for i in (0..buf.len()).step_by(SIZE) {
            let b = &buf[i..min(buf.len(), i + SIZE)];
            let r = &res[i..min(res.len(), i + SIZE)];
            assert_eq!(b, r);
        }

        Ok(())
    }
}
