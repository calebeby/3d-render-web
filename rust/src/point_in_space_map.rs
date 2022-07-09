use crate::vector3d::Vector3D;
use num::traits::Float;
use std::collections::HashMap;

const THRESHOLD: f64 = 1e-10;
const HALF_THRESHOLD: f64 = THRESHOLD / 2.0;

#[derive(PartialEq, Eq, Hash, Debug)]
struct SplitUpFloat(u64, i16, i8);
impl SplitUpFloat {
    pub fn new(f: f64) -> Self {
        let (mantissa, exponent, sign) = Float::integer_decode(f);
        Self(mantissa, exponent, sign)
    }
}

fn round(f: f64) -> f64 {
    let rounded = (f / THRESHOLD).round() * THRESHOLD;
    if rounded == -0.0 {
        0.0
    } else {
        rounded
    }
}

#[derive(PartialEq, Eq, Hash, Debug)]
struct VectorKey(SplitUpFloat, SplitUpFloat, SplitUpFloat);
impl VectorKey {
    pub fn new(v: &Vector3D) -> Self {
        Self(
            SplitUpFloat::new(round(v.x)),
            SplitUpFloat::new(round(v.y)),
            SplitUpFloat::new(round(v.z)),
        )
    }

    /// If the point is near the edge of rounding buckets, it will return the adjacent buckets too
    /// This prevents errors where an insert and retrieval might barely round in different directions
    pub fn new_and_nearby(v: &Vector3D) -> Vec<Self> {
        let rounded_x = round(v.x);
        let rounded_y = round(v.y);
        let rounded_z = round(v.z);
        let mut output = vec![(rounded_x, rounded_y, rounded_z)];

        if rounded_x - v.x >= HALF_THRESHOLD {
            // rounded up by a lot; add the rounded down version
            let add_to_output: Vec<_> = output
                .iter()
                .map(|(x, y, z)| (x - THRESHOLD, *y, *z))
                .collect();
            output.extend_from_slice(&add_to_output);
        } else if v.x - rounded_x >= HALF_THRESHOLD {
            // rounded down by a lot; add the rounded up version
            let add_to_output: Vec<_> = output
                .iter()
                .map(|(x, y, z)| (x + THRESHOLD, *y, *z))
                .collect();
            output.extend_from_slice(&add_to_output);
        }

        if rounded_y - v.y >= HALF_THRESHOLD {
            // rounded up by a lot; add the rounded down version
            let add_to_output: Vec<_> = output
                .iter()
                .map(|(x, y, z)| (*x, y - THRESHOLD, *z))
                .collect();
            output.extend_from_slice(&add_to_output);
        } else if v.y - rounded_y >= HALF_THRESHOLD {
            // rounded down by a lot; add the rounded up version
            let add_to_output: Vec<_> = output
                .iter()
                .map(|(x, y, z)| (*x, y + THRESHOLD, *z))
                .collect();
            output.extend_from_slice(&add_to_output);
        }

        if rounded_z - v.z >= HALF_THRESHOLD {
            // rounded up by a lot; add the rounded down version
            let add_to_output: Vec<_> = output
                .iter()
                .map(|(x, y, z)| (*x, *y, z - THRESHOLD))
                .collect();
            output.extend_from_slice(&add_to_output);
        } else if v.z - rounded_z >= HALF_THRESHOLD {
            // rounded down by a lot; add the rounded up version
            let add_to_output: Vec<_> = output
                .iter()
                .map(|(x, y, z)| (*x, *y, z + THRESHOLD))
                .collect();
            output.extend_from_slice(&add_to_output);
        }

        output
            .into_iter()
            .map(|(x, y, z)| {
                Self(
                    SplitUpFloat::new(x),
                    SplitUpFloat::new(y),
                    SplitUpFloat::new(z),
                )
            })
            .collect()
    }
}

/// HashMap that is keyed by Vector3d's, and which deals with floating-point errors
pub struct PointInSpaceMap<V> {
    hash_map: HashMap<VectorKey, Vec<(Vector3D, V)>>,
}

impl<V: Clone> PointInSpaceMap<V> {
    pub fn new() -> Self {
        Self {
            hash_map: HashMap::new(),
        }
    }
    pub fn insert(&mut self, key: Vector3D, value: V) {
        let key_buckets = VectorKey::new_and_nearby(&key);
        for key_bucket in key_buckets {
            let entry = self.hash_map.entry(key_bucket).or_insert_with(Vec::new);
            let existing_entry = entry
                .iter_mut()
                .find(|(entry_key, _)| entry_key.approx_equals(&key));
            match existing_entry {
                Some((_existing_key, existing_value)) => {
                    *existing_value = value.clone();
                }
                None => entry.push((key, value.clone())),
            }
        }
    }
    pub fn get(&self, key: &Vector3D) -> Option<&V> {
        let key_bucket = VectorKey::new(key);
        let entry = self.hash_map.get(&key_bucket)?;
        let (_existing_key, existing_value) = entry
            .iter()
            .find(|(entry_key, _)| entry_key.approx_equals(key))?;
        Some(existing_value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[allow(clippy::assertions_on_constants)]
    fn test() {
        let mut f = PointInSpaceMap::new();
        assert!(f64::EPSILON < THRESHOLD);
        f.insert(Vector3D::new(0.0, 0.0, 0.0), 1);
        assert_eq!(f.get(&Vector3D::new(0.0, 0.0, 0.0)), Some(&1));

        // This is right on the edge of the threshold (inside)
        assert_eq!(
            f.get(&Vector3D::new(0.0 + THRESHOLD / 2.0 - 1e-20, 0.0, 0.0)),
            Some(&1)
        );

        // This is right on the other edge of the threshold (inside)
        assert_eq!(
            f.get(&Vector3D::new(0.0 - THRESHOLD / 2.0 + 1e-20, 0.0, 0.0)),
            Some(&1)
        );

        // This is right on the edge of the threshold (outside)
        assert_eq!(f.get(&Vector3D::new(0.0 + THRESHOLD / 2.0, 0.0, 0.0)), None);

        // This is right on the other edge of the threshold (outside)
        assert_eq!(f.get(&Vector3D::new(0.0 - THRESHOLD / 2.0, 0.0, 0.0)), None);

        // Fully outside
        assert_eq!(f.get(&Vector3D::new(0.01, 0.0, 0.0)), None);

        f.insert(Vector3D::new(0.0, 1.0 + THRESHOLD / 2.0, 0.0), 27);
        assert_eq!(
            f.get(&Vector3D::new(0.0, 1.0 + THRESHOLD / 2.0, 0.0)),
            Some(&27)
        );
        // These numbers are on the edge of being rounded either up or down
        // Both should work because we check adjacent buckets
        assert_eq!(
            f.get(&Vector3D::new(
                0.0,
                1.0 + THRESHOLD / 2.0 + f64::EPSILON,
                0.0
            )),
            Some(&27)
        );
        assert_eq!(
            f.get(&Vector3D::new(
                0.0,
                1.0 + THRESHOLD / 2.0 - f64::EPSILON,
                0.0
            )),
            Some(&27)
        );
    }
}
