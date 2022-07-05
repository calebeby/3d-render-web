#[derive(Debug, Eq, PartialEq, Hash, Clone)]
pub struct FaceMap(
    // The indices of this vector are the new face indexes.
    // The values are the old face indexes to pull colors from.
    pub Vec<usize>,
);

impl FaceMap {
    /// Apply another FaceMap after this one.
    /// Returns a new FaceMap that represents the combination.
    pub fn apply(&self, other: &FaceMap) -> FaceMap {
        FaceMap(
            other
                .0
                .iter()
                .map(|index_from_other| self.0[*index_from_other])
                .collect(),
        )
    }
    pub fn invert(&self) -> FaceMap {
        let mut inverted_face_map = vec![0; self.0.len()];
        for (i, val) in self.0.iter().enumerate() {
            inverted_face_map[*val] = i;
        }
        FaceMap(inverted_face_map)
    }
    pub fn identity(count: usize) -> FaceMap {
        FaceMap((0..count).into_iter().collect())
    }
    pub fn is_inverse_of(&self, other: &FaceMap) -> bool {
        for (i, val) in self.0.iter().enumerate() {
            if other.0[*val] != i {
                println!("i={i}, val={val}");
                return false;
            }
        }
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_face_map() {
        let initial_face_map = FaceMap(vec![0, 2, 1]);
        let second_face_map = FaceMap(vec![2, 0, 1]);
        let combined_face_map = initial_face_map.apply(&second_face_map);

        assert_eq!(combined_face_map.0, vec![1, 0, 2]);
        assert_eq!(initial_face_map.invert().0, vec![0, 2, 1]);
        assert_eq!(second_face_map.invert().0, vec![1, 2, 0]);

        assert!(initial_face_map.invert().is_inverse_of(&initial_face_map));
        assert!(!initial_face_map.invert().is_inverse_of(&second_face_map));
        assert!(second_face_map.invert().is_inverse_of(&second_face_map));
        assert!(!second_face_map.invert().is_inverse_of(&initial_face_map));

        assert_eq!(FaceMap::identity(3).0, vec![0, 1, 2]);
    }
}
