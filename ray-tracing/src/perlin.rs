use crate::{rand_range, ray::Point};

const POINT_COUNT: usize = 256;

pub struct Perlin {
    ranfloat: Vec<f64>,
    perm_x: Vec<usize>,
    perm_y: Vec<usize>,
    perm_z: Vec<usize>,
}

impl Perlin {
    pub fn new() -> Self {
        Self {
            ranfloat: (0..POINT_COUNT).map(|_| rand_range(0.0..1.0)).collect(),
            perm_x: Self::generate_perm(),
            perm_y: Self::generate_perm(),
            perm_z: Self::generate_perm(),
        }
    }

    pub fn noise(&self, p: &Point) -> f64 {
        let calc = |v| {
            let a = (4.0 * v) as usize;
            a & 255
        };

        let i = calc(p.x());
        let j = calc(p.y());
        let k = calc(p.z());

        self.ranfloat[self.perm_x[i] ^ self.perm_y[j] ^ self.perm_z[k]]
    }

    fn generate_perm() -> Vec<usize> {
        let mut p: Vec<_> = (0..POINT_COUNT).collect();
        Self::permutate(&mut p);
        p
    }

    fn permutate(p: &mut [usize]) {
        for i in (1..p.len()).rev() {
            let target = rand_range(0..=i);
            p.swap(i, target);
        }
    }
}
impl Default for Perlin {
    fn default() -> Self {
        Self::new()
    }
}
