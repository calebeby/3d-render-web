use crate::traverse_combinations::{traverse_combinations, TraverseResult};
use crate::twisty_puzzle::{Symmetry, Turn};
use crate::{bijection::Bijection, twisty_puzzle::TwistyPuzzle};
use std::cmp::Ordering;
use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::fmt::Debug;
use std::hash::Hash;
use std::rc::Rc;

/// A metamove is a set of moves that combines to one large "move"
/// that ends up (hopefully) moving only a small number of pieces.
#[derive(Clone)]
pub struct MetaMove {
    pub turns: Vec<usize>,
    // The indices of this vector are the new face indexes.
    // The values are the old face indexes to pull colors from.
    pub face_map: Bijection,
    pub num_affected_pieces: usize,
    pub puzzle: Rc<TwistyPuzzle>,
}

impl MetaMove {
    #[inline]
    pub fn new(puzzle: Rc<TwistyPuzzle>, turns: Vec<usize>, face_map: Bijection) -> Self {
        let derived_state = puzzle.get_derived_state(&puzzle.get_initial_state(), &face_map);
        let num_affected_pieces =
            puzzle.get_num_pieces() - puzzle.get_num_solved_pieces(&derived_state);

        MetaMove {
            turns,
            face_map,
            num_affected_pieces,
            puzzle,
        }
    }
    #[inline]
    pub fn new_infer_face_map(puzzle: Rc<TwistyPuzzle>, turns: Vec<usize>) -> Self {
        let face_map = turns.iter().fold(
            Bijection::identity(puzzle.get_num_faces()),
            |face_map, &turn_index| face_map.apply(&puzzle.turns[turn_index].face_map),
        );
        Self::new(puzzle, turns, face_map)
    }
    #[inline]
    pub fn empty(puzzle: Rc<TwistyPuzzle>) -> Self {
        MetaMove {
            turns: vec![],
            face_map: Bijection::identity(puzzle.get_num_faces()),
            num_affected_pieces: 0,
            puzzle,
        }
    }
    #[inline]
    pub fn apply(&self, other: &MetaMove) -> Self {
        MetaMove::new(
            Rc::clone(&self.puzzle),
            self.turns
                .iter()
                .chain(other.turns.iter())
                .cloned()
                .collect(),
            self.face_map.apply(&other.face_map),
        )
    }
    #[inline]
    pub fn apply_symmetry(&self, symmetry: &Symmetry) -> Self {
        MetaMove {
            turns: self
                .turns
                .iter()
                .map(|turn_index| symmetry.turn_map.0[*turn_index])
                .collect(),
            face_map: symmetry
                .face_map
                .apply(&self.face_map)
                .apply(&symmetry.face_map.invert()),
            num_affected_pieces: self.num_affected_pieces,
            puzzle: Rc::clone(&self.puzzle),
        }
    }
    #[inline]
    pub fn invert(&self) -> Self {
        let inverted_turns = self
            .turns
            .iter()
            .rev()
            .map(|&turn_index| self.puzzle.inverted_turn_index(turn_index))
            .collect();
        MetaMove::new(
            Rc::clone(&self.puzzle),
            inverted_turns,
            self.face_map.invert(),
        )
    }
}

impl Debug for MetaMove {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let turns: Vec<String> = self
            .turns
            .iter()
            .map(|turn_index| self.puzzle.turn_names[*turn_index].clone())
            .collect();

        struct TurnSequence(Vec<String>);
        impl Debug for TurnSequence {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "[{}] ({})", &self.0.join(", "), self.0.len())
            }
        }

        f.debug_struct("MetaMove")
            .field("turns", &TurnSequence(turns))
            .field("num_affected_pieces", &self.num_affected_pieces)
            .finish_non_exhaustive()
    }
}

impl PartialEq for MetaMove {
    fn eq(&self, other: &Self) -> bool {
        // comparing by turns instead of by face_map
        // because different sequences of turns can have the same effect,
        // but those should be represented by different (non-equal) MetaMoves
        self.turns == other.turns
    }
}
impl Hash for MetaMove {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        // comparing by turns instead of by face_map
        // because different sequences of turns can have the same effect,
        // but those should be represented by different (non-equal) MetaMoves
        self.turns.hash(state)
    }
}
impl Eq for MetaMove {}

impl PartialOrd for MetaMove {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
impl Ord for MetaMove {
    fn cmp(&self, other: &Self) -> Ordering {
        self.num_affected_pieces
            .cmp(&other.num_affected_pieces)
            .then(self.turns.len().cmp(&other.turns.len()))
            .then(other.turns.cmp(&self.turns))
    }
}

pub fn discover_metamoves<Filter>(
    puzzle: Rc<TwistyPuzzle>,
    filter: Filter,
    max_turns: usize,
) -> Vec<MetaMove>
where
    Filter: Fn(&MetaMove) -> bool,
{
    let mut best_metamoves = HashMap::<Bijection, MetaMove>::new();

    let turns: Vec<_> = puzzle.turns.iter().enumerate().collect();

    traverse_combinations(
        &turns,
        max_turns - 1,
        // We'll start out with a single known turn,
        // and then copy the metamoves all over the puzzle at the end.
        MetaMove::new_infer_face_map(Rc::clone(&puzzle), vec![0]),
        &|previous_metamove: &MetaMove, (turn_index, turn): &(usize, &Turn)| {
            let face_map = previous_metamove.face_map.apply(&turn.face_map);

            let new_turns = previous_metamove
                .turns
                .iter()
                .chain(std::iter::once(turn_index))
                .cloned()
                .collect();

            MetaMove::new(Rc::clone(&puzzle), new_turns, face_map)
        },
        &mut |metamove| {
            // Ignore "move sequences" if they are just one move
            if metamove.turns.len() <= 1 {
                return TraverseResult::Continue;
            }

            // Ignore if last move inverts move before; that is useless
            if puzzle.turns[metamove.turns[metamove.turns.len() - 1]]
                .face_map
                .is_inverse_of(&puzzle.turns[metamove.turns[metamove.turns.len() - 2]].face_map)
            {
                return TraverseResult::Skip;
            }

            if metamove.num_affected_pieces > 0 && filter(metamove) {
                // Since we started out with a fixed single turn,
                // now we need to expand out all the symmetric versions
                for symmetry in puzzle.symmetries.values() {
                    let sym_metamove = metamove.apply_symmetry(symmetry);
                    let entry = best_metamoves.entry(sym_metamove.face_map.clone());
                    match entry {
                        Entry::Occupied(mut entry) => {
                            if entry.get() > &sym_metamove {
                                entry.insert(sym_metamove.clone());
                            }
                        }
                        Entry::Vacant(entry) => {
                            entry.insert(sym_metamove.clone());
                        }
                    }
                }
            }
            TraverseResult::Continue
        },
    );

    best_metamoves.into_values().collect()
}

pub fn combine_metamoves<Filter>(
    puzzle: Rc<TwistyPuzzle>,
    filter: Filter,
    metamoves: &[MetaMove],
    depth: usize,
) -> Vec<MetaMove>
where
    Filter: Fn(&MetaMove) -> bool,
{
    let mut combined_metamoves = vec![];

    traverse_combinations(
        metamoves,
        depth,
        MetaMove::empty(puzzle),
        &|previous_metamove: &MetaMove, new_metamove: &MetaMove| {
            previous_metamove.apply(new_metamove)
        },
        &mut |mm| {
            if mm.num_affected_pieces != 0 && filter(mm) {
                combined_metamoves.push(mm.clone());
            }
            TraverseResult::Continue
        },
    );

    combined_metamoves
}

#[cfg(test)]
mod tests {
    use crate::puzzles;
    use insta::assert_debug_snapshot;

    use super::*;

    #[test]
    fn invert_metamoves() {
        let puzzle = Rc::new(puzzles::rubiks_cube_3x3());
        let mm1 = MetaMove::new_infer_face_map(Rc::clone(&puzzle), vec![0]);
        assert_eq!(
            mm1.invert(),
            MetaMove::new_infer_face_map(Rc::clone(&puzzle), vec![1])
        );
        let mm2 = MetaMove::new_infer_face_map(Rc::clone(&puzzle), vec![1, 3, 5, 7]);
        assert_eq!(
            mm2.invert(),
            MetaMove::new_infer_face_map(Rc::clone(&puzzle), vec![6, 4, 2, 0])
        );
    }

    #[test]
    fn test_apply_symmetry() {
        let puzzle = Rc::new(puzzles::rubiks_cube_3x3());
        let mm1 = MetaMove::new_infer_face_map(Rc::clone(&puzzle), vec![0, 2, 4]);
        for symmetry in &puzzle.symmetries {
            let mm2 = mm1.apply_symmetry(symmetry.1);
            assert_eq!(
                &mm2,
                &MetaMove::new_infer_face_map(Rc::clone(&puzzle), mm2.turns.clone())
            );
        }
    }

    #[test]
    fn test_discover_metamoves_2x2() {
        let puzzle = Rc::new(puzzles::rubiks_cube_2x2());
        let solved_state = puzzle.get_initial_state();
        let mut all_metamoves_2_moves = discover_metamoves(Rc::clone(&puzzle), |_| true, 2);

        for metamove in &all_metamoves_2_moves {
            assert_eq!(
                puzzle.get_derived_state(&solved_state, &metamove.face_map),
                puzzle.get_derived_state_from_turn_sequence(
                    &solved_state,
                    &mut metamove.turns.iter().cloned()
                )
            );
        }

        assert_eq!(all_metamoves_2_moves.len(), 27);

        all_metamoves_2_moves.sort_by(|mm1, mm2| {
            mm1.num_affected_pieces
                .cmp(&mm2.num_affected_pieces)
                .then(mm1.face_map.0.cmp(&mm2.face_map.0))
                .then(mm1.turns.cmp(&mm2.turns))
        });

        assert_debug_snapshot!(all_metamoves_2_moves
            .iter()
            .map(|mm| (mm.num_affected_pieces, mm.turns.clone()))
            .collect::<Vec<_>>());

        let all_metamoves_4_moves = discover_metamoves(Rc::clone(&puzzle), |_| true, 4);
        assert_eq!(all_metamoves_4_moves.len(), 687);
        assert_eq!(all_metamoves_4_moves[0].num_affected_pieces, 4);
    }

    #[test]
    fn test_discover_metamoves_pyraminx() {
        let puzzle = Rc::new(puzzles::pyraminx());
        let solved_state = puzzle.get_initial_state();
        let all_metamoves_4_moves = discover_metamoves(Rc::clone(&puzzle), |_| true, 4);
        assert_eq!(all_metamoves_4_moves[0].num_affected_pieces, 3);
        for mm in &all_metamoves_4_moves {
            if mm.num_affected_pieces == 3 {
                // It should be in the form [A, B, A', B']
                assert_eq!(mm.turns.len(), 4);
                assert_eq!(puzzle.inverted_turn_index(mm.turns[0]), mm.turns[2]);
                assert_eq!(puzzle.inverted_turn_index(mm.turns[1]), mm.turns[3]);
            }
        }

        assert_eq!(
            all_metamoves_4_moves
                .iter()
                .filter(|mm| mm.num_affected_pieces <= 3)
                .count(),
            48 // All 4-move (or less, but those don't exist) 3-cycles on the pyraminx
        );
        assert_eq!(all_metamoves_4_moves.len(), 2072);

        for metamove in &all_metamoves_4_moves {
            assert_eq!(
                puzzle.get_derived_state(&solved_state, &metamove.face_map),
                puzzle.get_derived_state_from_turn_sequence(
                    &solved_state,
                    &mut metamove.turns.iter().cloned()
                )
            );
        }
    }

    #[test]
    fn test_discover_metamoves_3x3() {
        let puzzle = Rc::new(puzzles::rubiks_cube_3x3());
        let solved_state = puzzle.get_initial_state();
        let all_metamoves_3_moves = discover_metamoves(Rc::clone(&puzzle), |_| true, 3);
        assert_eq!(all_metamoves_3_moves[0].num_affected_pieces, 8);
        assert_eq!(all_metamoves_3_moves[0].turns.len(), 2);
        // Two turns to affect 8 pieces, it is a double-turn on a single face
        assert_eq!(
            all_metamoves_3_moves[0].turns[0],
            all_metamoves_3_moves[0].turns[1]
        );
        assert_eq!(
            all_metamoves_3_moves
                .iter()
                .filter(|mm| mm.num_affected_pieces == 8)
                .count(),
            114
        );
        assert_eq!(all_metamoves_3_moves.len(), 1194);

        for metamove in &all_metamoves_3_moves {
            assert_eq!(
                puzzle.get_derived_state(&solved_state, &metamove.face_map),
                puzzle.get_derived_state_from_turn_sequence(
                    &solved_state,
                    &mut metamove.turns.iter().cloned()
                )
            );
        }
    }
}
