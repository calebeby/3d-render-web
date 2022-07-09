use crate::vector3d::Vector3D;
use num::traits::Float;
use std::collections::HashMap;

const THRESHOLD: f64 = 1e-8;

#[derive(PartialEq, Eq, Hash, Debug)]
struct SplitUpFloat(u64, i16, i8);
impl SplitUpFloat {
    pub fn new(f: f64) -> Self {
        let (mantissa, exponent, sign) = Float::integer_decode(f);
        Self(mantissa, exponent, sign)
    }
}

#[derive(PartialEq, Eq, Hash, Debug)]
struct VectorKey(SplitUpFloat, SplitUpFloat, SplitUpFloat);
impl VectorKey {
    pub fn new(v: &Vector3D) -> Self {
        Self(
            SplitUpFloat::new((v.x / THRESHOLD).round() * THRESHOLD),
            SplitUpFloat::new((v.y / THRESHOLD).round() * THRESHOLD),
            SplitUpFloat::new((v.z / THRESHOLD).round() * THRESHOLD),
        )
    }
}

/// HashMap that is keyed by Vector3d's, and which deals with floating-point errors
pub struct PointInSpaceMap<V> {
    hash_map: HashMap<VectorKey, Vec<(Vector3D, V)>>,
}

impl<V> PointInSpaceMap<V> {
    pub fn new() -> Self {
        Self {
            hash_map: HashMap::new(),
        }
    }
    pub fn insert(&mut self, key: Vector3D, value: V) {
        let key_bucket = VectorKey::new(&key);
        let entry = self.hash_map.entry(key_bucket).or_insert_with(Vec::new);
        let existing_entry = entry
            .iter_mut()
            .find(|(entry_key, _)| entry_key.approx_equals(&key));
        match existing_entry {
            Some((_existing_key, existing_value)) => {
                *existing_value = value;
            }
            None => entry.push((key, value)),
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
    fn test() {
        let mut f = PointInSpaceMap::new();
        f.insert(Vector3D::new(0.0, 0.0, 0.0), 1);
        assert_eq!(f.get(&Vector3D::new(0.0, 0.0, 0.0)), Some(&1));
        assert_eq!(
            f.get(&Vector3D::new(
                0.0000000000001,
                0.0000000000001,
                0.0000000000001
            )),
            Some(&1)
        );
        assert_eq!(f.get(&Vector3D::new(0.01, 0.0, 0.0)), None);
    }
}
