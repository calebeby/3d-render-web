use crate::traverse_combinations::{traverse_combinations, TraverseResult};
use crate::twisty_puzzle::Turn;
use crate::{face_map::FaceMap, twisty_puzzle::TwistyPuzzle};
use std::cmp::Ordering;

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
    pub fn apply(&self, puzzle: &TwistyPuzzle, other: &MetaMove) -> Self {
        MetaMove::new(
            &puzzle,
            self.turns
                .iter()
                .chain(other.turns.iter())
                .cloned()
                .collect(),
            self.face_map.apply(&other.face_map),
        )
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

pub fn discover_metamoves<Filter>(
    puzzle: &TwistyPuzzle,
    filter: Filter,
    max_turns: usize,
) -> Vec<MetaMove>
where
    Filter: Fn(&MetaMove) -> bool,
{
    let mut best_metamoves: Vec<MetaMove> = vec![];

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

            if metamove.num_affected_pieces > 0 {
                if filter(metamove) {
                    best_metamoves.push(metamove.clone());
                }
            }
            TraverseResult::Continue
        },
    );

    best_metamoves.sort();
    best_metamoves
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
        // TODO: test filter
        let all_metamoves_2_moves = discover_metamoves(&puzzle, |_| true, 2);

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

        let all_metamoves_4_moves = discover_metamoves(&puzzle, |_| true, 4);
        assert_eq!(all_metamoves_4_moves.len(), 693);
        assert_eq!(all_metamoves_4_moves[0].turns, [4, 4]);
        assert_eq!(all_metamoves_4_moves[0].num_affected_pieces, 4);
    }

    #[test]
    fn test_discover_metamoves_pyraminx() {
        let puzzle = puzzles::pyraminx();
        let all_metamoves_4_moves = discover_metamoves(&puzzle, |_| true, 4);
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
        let all_metamoves_4_moves = discover_metamoves(&puzzle, |_| true, 4);
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
