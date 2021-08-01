use crate::ray::Point;


const POINT_COUNT : usize = 256;

pub struct Perlin {
    ranfloat: [f64; POINT_COUNT],
    perm_x : [usize; POINT_COUNT],
    perm_y : [usize; POINT_COUNT],
    perm_z : [usize; POINT_COUNT],
}

impl Perlin {
   pub fn new() -> Perlin {
    todo!()
   }

   pub fn noise(&self, p: &Point) -> f64 {
       let calc = |v| ((4.0 * v) as usize) & 255;
       let i = calc(p.x());
       let j = calc(p.y());
       let k = calc(p.z());

        self.ranfloat[
            self.perm_x[i] ^
            self.perm_y[j] ^
            self.perm_z[k]
        ]
   }
}


