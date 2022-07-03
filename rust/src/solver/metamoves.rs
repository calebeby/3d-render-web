use crate::traverse_combinations::{traverse_combinations, TraverseResult};
use crate::twisty_puzzle::{PuzzleState, Turn};
use crate::{face_map::FaceMap, twisty_puzzle::TwistyPuzzle};
use std::cmp::Ordering;
use std::collections::{BinaryHeap, HashMap};

/// A metamove is a set of moves that combines to one large "move"
/// that ends up (hopefully) moving only a small number of pieces.
#[derive(Debug, Clone)]
pub struct MetaMove {
    pub turns: Vec<usize>,
    // The indices of this vector are the new face indexes.
    // The values are the old face indexes to pull colors from.
    pub face_map: FaceMap,
    pub num_affected_pieces: usize,
}

impl MetaMove {
    pub fn new(puzzle: &TwistyPuzzle, turns: Vec<usize>, face_map: FaceMap) -> Self {
        let derived_state = puzzle.get_derived_state(&puzzle.get_initial_state(), &face_map);
        let num_affected_pieces =
            puzzle.get_num_pieces() - puzzle.get_num_solved_pieces(&derived_state);

        MetaMove {
            turns,
            face_map,
            num_affected_pieces,
        }
    }
    pub fn empty(puzzle: &TwistyPuzzle) -> Self {
        MetaMove {
            turns: vec![],
            face_map: FaceMap::identity(puzzle.get_num_faces()),
            num_affected_pieces: 0,
        }
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
            .then(self.turns.cmp(&other.turns).reverse())
    }
}

pub fn discover_metamoves(puzzle: &TwistyPuzzle, max_turns: usize) -> Vec<MetaMove> {
    let mut best_metamoves: BinaryHeap<MetaMove> = BinaryHeap::new();
    // Each face map is stored with the fewest number of moves to achieve that face map.
    let mut face_map_optimal_num_moves: HashMap<FaceMap, usize> = HashMap::new();
    face_map_optimal_num_moves.insert(FaceMap::identity(puzzle.turns.len()), 0);
    for turn in &puzzle.turns {
        face_map_optimal_num_moves.insert(turn.face_map.clone(), 1);
    }

    let turns: Vec<_> = puzzle.turns.iter().enumerate().collect();

    traverse_combinations(
        &turns,
        max_turns,
        MetaMove::empty(puzzle),
        &|previous_metamove: &MetaMove, (turn_index, turn): &(usize, &Turn)| {
            let face_map = previous_metamove.face_map.apply(&turn.face_map);
            let derived_state = puzzle.get_derived_state(&puzzle.get_initial_state(), &face_map);
            let num_affected_pieces =
                puzzle.get_num_pieces() - puzzle.get_num_solved_pieces(&derived_state);

            MetaMove {
                num_affected_pieces,
                face_map,
                turns: previous_metamove
                    .turns
                    .iter()
                    .chain(std::iter::once(turn_index))
                    .cloned()
                    .collect(),
            }
        },
        &mut |metamove| {
            // Ignore "move sequences" if they are just one move
            // Also ignore move sequences that cancel themselves out and have no effect
            if metamove.turns.len() <= 1 || metamove.num_affected_pieces == 0 {
                return TraverseResult::Continue;
            }

            // We want to maximize the number of solved pieces:
            // minimize the number of pieces that were affected.

            // A set of moves that has the same outcome as the current one.
            let saved_equivalent_metamove = face_map_optimal_num_moves.get(&metamove.face_map);

            match saved_equivalent_metamove {
                Some(&saved_num_turns) if saved_num_turns <= metamove.turns.len() => {
                    // If the saved metamove is better than the current metamove,
                    // We don't need to expand this state,
                    // because every derived state will also be more optimally solved
                    // using the saved solutions than the current solution
                    TraverseResult::Skip
                }
                _ => {
                    face_map_optimal_num_moves
                        .insert(metamove.face_map.clone(), metamove.turns.len());
                    best_metamoves.push(metamove.clone());
                    TraverseResult::Continue
                }
            }
        },
    );

    best_metamoves.into_sorted_vec()
}

#[cfg(test)]
mod tests {
    use crate::puzzles;
    use insta::assert_debug_snapshot;

    use super::*;

    #[test]
    fn test_discover_metamoves_2x2() {
        let puzzle = puzzles::rubiks_cube_2x2();
        let solved_state = puzzle.get_initial_state();
        let all_metamoves_2_moves = discover_metamoves(&puzzle, 2);

        assert_eq!(all_metamoves_2_moves.len(), 27);

        assert_debug_snapshot!(all_metamoves_2_moves
            .iter()
            .map(|mm| (mm.num_affected_pieces, mm.turns.clone()))
            .collect::<Vec<_>>(), @r###"
        [
            (
                4,
                [
                    4,
                    4,
                ],
            ),
            (
                4,
                [
                    2,
                    2,
                ],
            ),
            (
                4,
                [
                    0,
                    0,
                ],
            ),
            (
                6,
                [
                    5,
                    3,
                ],
            ),
            (
                6,
                [
                    5,
                    2,
                ],
            ),
            (
                6,
                [
                    5,
                    1,
                ],
            ),
            (
                6,
                [
                    5,
                    0,
                ],
            ),
            (
                6,
                [
                    4,
                    3,
                ],
            ),
            (
                6,
                [
                    4,
                    2,
                ],
            ),
            (
                6,
                [
                    4,
                    1,
                ],
            ),
            (
                6,
                [
                    4,
                    0,
                ],
            ),
            (
                6,
                [
                    3,
                    5,
                ],
            ),
            (
                6,
                [
                    3,
                    4,
                ],
            ),
            (
                6,
                [
                    3,
                    1,
                ],
            ),
            (
                6,
                [
                    3,
                    0,
                ],
            ),
            (
                6,
                [
                    2,
                    5,
                ],
            ),
            (
                6,
                [
                    2,
                    4,
                ],
            ),
            (
                6,
                [
                    2,
                    1,
                ],
            ),
            (
                6,
                [
                    2,
                    0,
                ],
            ),
            (
                6,
                [
                    1,
                    5,
                ],
            ),
            (
                6,
                [
                    1,
                    4,
                ],
            ),
            (
                6,
                [
                    1,
                    3,
                ],
            ),
            (
                6,
                [
                    1,
                    2,
                ],
            ),
            (
                6,
                [
                    0,
                    5,
                ],
            ),
            (
                6,
                [
                    0,
                    4,
                ],
            ),
            (
                6,
                [
                    0,
                    3,
                ],
            ),
            (
                6,
                [
                    0,
                    2,
                ],
            ),
        ]
        "###);

        for metamove in &all_metamoves_2_moves {
            assert_eq!(
                puzzle.get_derived_state(&solved_state, &metamove.face_map),
                puzzle.get_derived_state_from_turns_iter(
                    &solved_state,
                    &mut metamove.turns.iter().cloned()
                )
            );
        }

        let all_metamoves_4_moves = discover_metamoves(&puzzle, 4);
        assert_eq!(all_metamoves_4_moves.len(), 693);
        assert_eq!(all_metamoves_4_moves[0].turns, [4, 4]);
        assert_eq!(all_metamoves_4_moves[0].num_affected_pieces, 4);
    }

    #[test]
    fn test_discover_metamoves_pyraminx() {
        let puzzle = puzzles::pyraminx();
        let all_metamoves_4_moves = discover_metamoves(&puzzle, 4);
        assert_eq!(all_metamoves_4_moves[0].num_affected_pieces, 3);
        assert_eq!(all_metamoves_4_moves[0].turns, [7, 5, 6, 4]);

        assert_eq!(all_metamoves_4_moves.len(), 2304);
        assert_eq!(
            all_metamoves_4_moves
                .iter()
                .filter(|mm| mm.num_affected_pieces == 3)
                .count(),
            48 // All 4-move (or less, but those don't exist) 3-cycles on the pyraminx
        );
    }

    #[test]
    fn test_discover_metamoves_3x3() {
        let puzzle = puzzles::rubiks_cube_3x3();
        let all_metamoves_4_moves = discover_metamoves(&puzzle, 4);
        assert_eq!(all_metamoves_4_moves[0].num_affected_pieces, 7);
        assert_eq!(all_metamoves_4_moves[0].turns, [11, 9, 10, 8]);

        assert_eq!(all_metamoves_4_moves.len(), 11280);
        assert_eq!(
            all_metamoves_4_moves
                .iter()
                .filter(|mm| mm.num_affected_pieces == 7)
                .count(),
            96
        );
    }
}
