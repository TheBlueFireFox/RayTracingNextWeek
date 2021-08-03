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
        // let calc = |v| {
        //     let a = (4.0 * v) as isize;
        //     (a & (POINT_COUNT as isize - 1)) as usize
        // };

        // let i = calc(p.x());
        // let j = calc(p.y());
        // let k = calc(p.z());

        // self.ranfloat[self.perm_x[i] ^ self.perm_y[j] ^ self.perm_z[k]]
        let calc = |v: f64| {
            let res = v - v.floor();
            res*res*(3.0-2.0*res)
        };

        let u = calc(p.x());
        let v = calc(p.y());
        let w = calc(p.z());

        let calc = |v: f64| v.floor() as isize;
        let i = calc(p.x());
        let j = calc(p.y());
        let k = calc(p.z());
        let mut c = [[[0.0;2];2];2];

        let calc = |i, di| ((i + di as isize) & 255) as usize;
        for di in 0..2 {
            for dj in 0..2 {
                for dk in 0..2 {
                    c[di][dj][dk] = self.ranfloat [
                        self.perm_x[calc(i, di)] ^
                        self.perm_y[calc(j, dj)] ^
                        self.perm_z[calc(k, dk)] 
                    ];
                }
            }
        }
        Self::trilinear_interp(&c, u, v, w)
    }

    fn trilinear_interp(c: &[[[f64; 2]; 2]], u: f64, v: f64, w: f64) -> f64 {
        let mut accum = 0.0;
        for ii in 0..2 {
            let i = ii as f64;
            for jj in 0..2 {
                let j = jj as f64;
                for kk in 0..2 {
                    let k = kk as f64;
                    accum += (i * u + (1.0 - i) * (1.0 - u))
                        * (j * v + (1.0 - j) * (1.0 - v))
                        * (k * w + (1.0 - k) * (1.0 - w))
                        * c[ii][jj][kk];
                }
            }
        }
        accum
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
