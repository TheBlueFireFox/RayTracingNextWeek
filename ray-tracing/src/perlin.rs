use crate::{
    rand_range,
    ray::{Point, Vec3},
};

const POINT_COUNT: usize = 256;

pub struct Perlin {
    ranvec: Vec<Vec3>,
    perm_x: Vec<usize>,
    perm_y: Vec<usize>,
    perm_z: Vec<usize>,
}

impl Perlin {
    pub fn new() -> Self {
        Self {
            ranvec: (0..POINT_COUNT)
                .map(|_| Vec3::random_range(-1.0..=1.0).unit_vector())
                .collect(),
            perm_x: Self::generate_perm(),
            perm_y: Self::generate_perm(),
            perm_z: Self::generate_perm(),
        }
    }

    pub fn noise(&self, p: &Point) -> f64 {
        let calc = |v: f64| v - v.floor();

        let u = calc(p.x());
        let v = calc(p.y());
        let w = calc(p.z());

        let calc = |v: f64| v.floor() as isize;
        let i = calc(p.x());
        let j = calc(p.y());
        let k = calc(p.z());

        let mut c = [[[Vec3::default(); 2]; 2]; 2];

        let calc = |i, di| ((i + di as isize) & 255) as usize;
        for di in 0..2 {
            for dj in 0..2 {
                for dk in 0..2 {
                    c[di][dj][dk] = self.ranvec[self.perm_x[calc(i, di)]
                        ^ self.perm_y[calc(j, dj)]
                        ^ self.perm_z[calc(k, dk)]];
                }
            }
        }
        Self::perlin_interp(&c, u, v, w)
    }

    pub fn turb(&self, p: &Point) -> f64 {
        self.turb_with_depth(p, 7)
    }

    pub fn turb_with_depth(&self, p: &Point, depth: usize) -> f64 {
        let mut accum = 0.0;
        let mut temp_p = p.clone();
        let mut weight = 1.0;

        for _ in 0..depth {
            accum += weight * self.noise(&temp_p);
            weight *= 0.5;
            temp_p *= 2.0;
        }

        accum.abs()
    }

    fn perlin_interp(c: &[[[Vec3; 2]; 2]], u: f64, v: f64, w: f64) -> f64 {
        let calc = |v: f64| v * v * (3.0 - 2.0 * v);

        let uu = calc(u);
        let vv = calc(v);
        let ww = calc(w);

        let mut accum = 0.0;

        for ii in 0..2 {
            let i = ii as f64;

            for jj in 0..2 {
                let j = jj as f64;

                for kk in 0..2 {
                    let k = kk as f64;
                    let weight_v = [u - i, v - j, w - k].into();
                    accum += (i * uu + (1.0 - i) * (1.0 - uu))
                        * (j * vv + (1.0 - j) * (1.0 - vv))
                        * (k * ww + (1.0 - k) * (1.0 - ww))
                        * Vec3::dot(&c[ii][jj][kk], &weight_v);
                }
            }
        }

        accum
    }

    // from part 5.4
    #[allow(dead_code)]
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
