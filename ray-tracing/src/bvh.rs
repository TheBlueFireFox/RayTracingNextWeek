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
        Self::from_list(&list.objects(), 0, list.objects().len(), time0, time1)
    }

    pub fn from_list(
        objects: &[HittableObject],
        start: usize,
        end: usize,
        time0: f64,
        time1: f64,
    ) -> Self {
        // Create a modifable array of the source scene objects
        Self::inner_from_list(&mut objects.to_vec(), start, end, time0, time1)
    }

    fn inner_from_list(
        objects: &mut [HittableObject],
        start: usize,
        end: usize,
        time0: f64,
        time1: f64,
    ) -> Self {
        let comparator: &dyn Fn(&HittableObject, &HittableObject) -> Ordering;
        comparator = match rand_range(0..=2) {
            0 => &box_x_compare,
            1 => &box_y_compare,
            _ => &box_z_compare,
        };

        assert!(start < end, "start should be smaller then the end");

        let object_span = end - start;

        let (left, right) = match object_span {
            1 => (objects[start].clone(), objects[start].clone()),
            2 => match comparator(&objects[start], &objects[start + 1]) {
                Ordering::Greater => (objects[start + 1].clone(), objects[start].clone()),
                _ => (objects[start].clone(), objects[start + 1].clone()),
            },
            _ => {
                // Don't care about reordering here
                objects.sort_unstable_by(comparator);

                let mid = start + object_span / 2;

                let left = Arc::new(Self::inner_from_list(objects, start, mid, time0, time1));
                let right = Arc::new(Self::inner_from_list(objects, mid, end, time0, time1));

                (left as Arc<dyn Hittable>, right as Arc<dyn Hittable>)
            }
        };

        let (box_left, box_right) = Self::check_cond(&left, &right, (time0, time1))
            .expect("No bounding box in bvh_node constructor.");

        let ibox = Aabb::surrounding_box(&box_left, &box_right);

        Self { left, right, ibox }
    }

    fn check_cond(
        left: &HittableObject,
        right: &HittableObject,
        time: (f64, f64),
    ) -> Option<(Aabb, Aabb)> {
        let l = left.bounding_box(time.0, time.1);
        let r = right.bounding_box(time.0, time.1);

        match (l, r) {
            (Some(left), Some(right)) => Some((left, right)),
            _ => None,
        }
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
    fn bounding_box(&self, _time0: f64, _time1: f64) -> Option<Aabb> {
        Some(self.ibox.clone())
    }

    fn hit(&self, r: &Ray, t_min: f64, t_max: f64, rec: &mut HitRecord) -> bool {
        if !self.ibox.hit(r, t_min, t_max) {
            return false;
        }
        let hit_left = self.left.hit(r, t_min, t_max, rec);

        let using = if hit_left { rec.t } else { t_max };

        let hit_right = self.right.hit(r, t_min, using, rec);

        return hit_left || hit_right;
    }
}

fn box_compare(a: &HittableObject, b: &HittableObject, axis: usize) -> Ordering {
    let (box_a, box_b) = BvhNode::check_cond(a, b, (0.0, 0.0)).unwrap_or_else(|| {
        panic!(
            "No bounding box in bvh_node box compare found during a box compare with axis {}.",
            axis
        )
    });

    // As there currently is no total_cmp implemented
    // we are going to assume that no illegal values
    // are used here
    let left = box_a.min().data()[axis];
    let right = box_b.min().data()[axis];

    f64::partial_cmp(&left, &right).expect("illegal f64 value was used here, no NaN or Inf allowed")
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
