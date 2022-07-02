#[derive(Debug)]
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
        for (val, i) in self.0.iter().enumerate() {
            inverted_face_map[*i] = val;
        }
        FaceMap(inverted_face_map)
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
    }
}
