use std::{cmp::Ordering, sync::Arc};

use crate::{
    aabb::Aabb,
    hittable::{HitRecord, Hittable, HittableList, HittableObject},
    rand_range,
    ray::Ray,
};

pub struct BvhNode {
    left: HittableObject,
    right: HittableObject,
    ibox: Aabb,
}

impl BvhNode {
    pub fn from_hittable_list(list: &HittableList, time0: f64, time1: f64) -> Self {
        // Create a modifable array of the source scene objects
        let mut c = list.objects().to_vec();

        Self::from_list(&mut c, 0, list.objects().len(), time0, time1)
    }

    pub fn from_list(
        objects: &mut [HittableObject],
        start: usize,
        end: usize,
        time0: f64,
        time1: f64,
    ) -> Self {

        let axis = rand_range(0..=2);
        let comparator: &dyn Fn(&HittableObject, &HittableObject) -> Ordering = match axis {
            0 => &box_x_compare,
            1 => &box_y_compare,
            _ => &box_z_compare,
        };

        let object_span = end - start;

        let left;
        let right;
        match object_span {
            1 => {
                left = objects[start].clone();
                right = objects[start].clone();
            }
            2 => match comparator(&objects[start], &objects[start]) {
                Ordering::Less => {
                    left = objects[start].clone();
                    right = objects[start + 1].clone();
                }
                _ => {
                    left = objects[start + 1].clone();
                    right = objects[start].clone();
                }
            },
            _ => {
                objects.sort_by(comparator);
                let mid = start + object_span / 2;
                left = Arc::new(Self::from_list(objects, start, mid, time0, time1));
                right = Arc::new(Self::from_list(objects, mid, end, time0, time1));

            }
        }

        let mut box_left = Aabb::default();
        let mut box_right = Aabb::default();
        assert!(
            !left.bounding_box(time0, time1, &mut box_left)
                || !right.bounding_box(time0, time1, &mut box_right),
            "No bounding box in bvh_node constructor.\n"
        );

        let ibox = Aabb::surrounding_box(&box_left, &box_right);

        Self { left, right, ibox }
    }

    /// Get a clone to the bvh node's left.
    pub fn left(&self) -> HittableObject {
        self.left.clone()
    }

    /// Get a clone to the bvh node's right.
    pub fn right(&self) -> HittableObject {
        self.right.clone()
    }

    /// Get a reference to the bvh node's ibox.
    pub fn ibox(&self) -> &Aabb {
        &self.ibox
    }
}

impl Hittable for BvhNode {
    fn bounding_box(&self, _time0: f64, _time1: f64, output: &mut Aabb) -> bool {
        *output = self.ibox.clone();
        true
    }

    fn hit(&self, r: &Ray, t_min: f64, t_max: f64, rec: &mut HitRecord) -> bool {
        if self.ibox.hit(r, t_min, t_max) {
            return false;
        }
        let hit_left = self.left.hit(r, t_min, t_max, rec);
        let hit_right = self.right.hit(r, t_min, t_max, rec);

        return hit_left || hit_right;
    }
}

#[inline]
fn box_compare(a: &HittableObject, b: &HittableObject, axis: usize) -> Ordering {
    let box_a = Aabb::default();
    let box_b = Aabb::default();

    assert!(
        !a.bounding_box(0.0, 0.0, &mut box_a) || !b.bounding_box(0.0, 0.0, &mut box_a),
        "No bounding box in bvh_node constructor.\n"
    );

    // As there currently is no total_cmp implemented
    // we are going to assume that no illegal values
    // are used here
    box_a.min().data()[axis]
        .partial_cmp(&box_b.min().data()[axis])
        .expect("illegal f64 value was used here")
}

fn box_x_compare(a: &HittableObject, b: &HittableObject) -> Ordering {
    box_compare(a, b, 0)
}

fn box_y_compare(a: &HittableObject, b: &HittableObject) -> Ordering {
    box_compare(a, b, 1)
}

fn box_z_compare(a: &HittableObject, b: &HittableObject) -> Ordering {
    box_compare(a, b, 2)
}
