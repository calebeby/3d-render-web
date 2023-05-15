#[derive(Debug, Eq, PartialEq, Hash, Clone)]
pub struct Bijection(
    // The indices of this vector are the new (output side) indexes.
    // The values are the old (input side) indexes to pull from.
    pub Vec<usize>,
);

impl Bijection {
    /// Apply another Bijection after this one.
    /// Returns a new Bijection that represents the combination.
    pub fn apply(&self, other: &Bijection) -> Bijection {
        Bijection(
            other
                .0
                .iter()
                .map(|index_from_other| self.0[*index_from_other])
                .collect(),
        )
    }
    pub fn invert(&self) -> Bijection {
        let mut inverted = vec![0; self.0.len()];
        for (i, val) in self.0.iter().enumerate() {
            inverted[*val] = i;
        }
        Bijection(inverted)
    }
    pub fn identity(count: usize) -> Bijection {
        Bijection((0..count).collect())
    }
    pub fn is_inverse_of(&self, other: &Bijection) -> bool {
        for (i, val) in self.0.iter().enumerate() {
            if other.0[*val] != i {
                return false;
            }
        }
        true
    }
    pub fn mask(&self, mask: &[bool]) -> Bijection {
        assert_eq!(mask.len(), self.0.len());
        Bijection(
            self.0
                .iter()
                .enumerate()
                .map(|(i, mapping)| {
                    if mask[i] {
                        *mapping
                    } else {
                        // map to itself (since it is not in the mapping)
                        i
                    }
                })
                .collect(),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bijection() {
        let initial = Bijection(vec![0, 2, 1]);
        let second = Bijection(vec![2, 0, 1]);
        let combined = initial.apply(&second);

        assert_eq!(combined.0, vec![1, 0, 2]);
        assert_eq!(initial.invert().0, vec![0, 2, 1]);
        assert_eq!(second.invert().0, vec![1, 2, 0]);

        assert!(initial.invert().is_inverse_of(&initial));
        assert!(!initial.invert().is_inverse_of(&second));
        assert!(second.invert().is_inverse_of(&second));
        assert!(!second.invert().is_inverse_of(&initial));

        assert_eq!(Bijection::identity(3).0, vec![0, 1, 2]);
    }
}
