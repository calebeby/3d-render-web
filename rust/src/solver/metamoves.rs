use crate::twisty_puzzle::PuzzleState;
use crate::{face_map::FaceMap, twisty_puzzle::TwistyPuzzle};
use std::cmp::Ordering;
use std::collections::{BinaryHeap, HashMap};

/// A metamove is a set of moves that combines to one large "move"
/// that ends up (hopefully) moving only a small number of pieces.
#[derive(Debug)]
pub struct MetaMove {
    pub turns: Vec<usize>,
    // The indices of this vector are the new face indexes.
    // The values are the old face indexes to pull colors from.
    pub face_map: FaceMap,
    num_affected_pieces: usize,
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

pub fn discover_metamoves(
    puzzle: &TwistyPuzzle,
    max_turns: usize,
    // Pass 0 for no limit
    num_metamoves: usize,
) -> Vec<MetaMove> {
    let solved_state = puzzle.get_initial_state();
    let mut best_metamoves = BinaryHeap::new();
    // Each face map is stored with the fewest number of moves to achieve that face map.
    let mut face_map_optimal_num_moves: HashMap<FaceMap, usize> = HashMap::new();
    let mut debug_map: HashMap<FaceMap, Vec<usize>> = HashMap::new();
    face_map_optimal_num_moves.insert(FaceMap::identity(puzzle.turns.len()), 0);
    debug_map.insert(FaceMap::identity(puzzle.turns.len()), vec![]);
    for (turn_index, turn) in puzzle.turns.iter().enumerate() {
        face_map_optimal_num_moves.insert(turn.face_map.clone(), 1);
        debug_map.insert(turn.face_map.clone(), vec![turn_index]);
    }
    let mut fringe_stack: Vec<StateToExpand> = vec![StateToExpand {
        puzzle_state: solved_state.clone(),
        turn_index: 0,
    }];

    while let Some(state_to_expand) = fringe_stack.last() {
        if fringe_stack.len() < max_turns + 1 {
            let derived_state = puzzle.get_derived_state_turn_index(
                &state_to_expand.puzzle_state,
                state_to_expand.turn_index,
            );
            let num_affected_pieces =
                puzzle.get_num_pieces() - puzzle.get_num_solved_pieces(&derived_state);
            // Ignore move sequences that cancel themselves out and have no effect
            // Also ignore "move sequences" if they are just one move
            if fringe_stack.len() > 1 && num_affected_pieces > 0 {
                // We want to maximize the number of solved pieces:
                // minimize the number of pieces that were affected.
                let turns: Vec<_> = fringe_stack
                    .iter()
                    .map(|state_to_expand| state_to_expand.turn_index)
                    .collect();
                let face_map = turns.iter().fold(
                    FaceMap::identity(solved_state.len()),
                    |face_map, turn_index| face_map.apply(&puzzle.turns[*turn_index].face_map),
                );
                // A set of moves that has the same outcome as the current one.
                let saved_equivalent_metamove = face_map_optimal_num_moves.get(&face_map);
                match saved_equivalent_metamove {
                    Some(&saved_num_turns) if saved_num_turns <= turns.len() => {
                        // If the saved metamove is better than the current metamove,
                        // We don't need to expand this state,
                        // because every derived state will also be more optimally solved
                        // using the saved solutions than the current solution
                        increment(&mut fringe_stack, puzzle.turns.len());
                        continue;
                    }
                    _ => {
                        face_map_optimal_num_moves.insert(face_map.clone(), turns.len());
                        debug_map.insert(face_map.clone(), turns.clone());
                        let new_metamove = MetaMove {
                            face_map,
                            num_affected_pieces,
                            turns,
                        };
                        if num_metamoves == 0 || best_metamoves.len() < num_metamoves {
                            // There is space in the best_metamoves so add it
                            best_metamoves.push(new_metamove);
                        } else {
                            // There is not space so we have to decide whether to kick one out
                            let worst_saved_metamove = best_metamoves.peek().unwrap();
                            if new_metamove < *worst_saved_metamove {
                                // kick it out/replace it
                                best_metamoves.pop();
                                best_metamoves.push(new_metamove);
                            }
                        }
                    }
                }
            }
            fringe_stack.push(StateToExpand {
                puzzle_state: derived_state,
                turn_index: 0,
            })
        } else {
            increment(&mut fringe_stack, puzzle.turns.len());
        }
    }

    best_metamoves.into_sorted_vec()
}

struct StateToExpand {
    puzzle_state: PuzzleState,
    turn_index: usize,
}

fn increment(fringe_stack: &mut Vec<StateToExpand>, num_turns: usize) {
    while let Some(solution_to_increment) = fringe_stack.last_mut() {
        if solution_to_increment.turn_index < num_turns - 1 {
            solution_to_increment.turn_index += 1;
            break;
        } else {
            fringe_stack.pop();
        }
    }
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
        let all_metamoves_2_moves = discover_metamoves(&puzzle, 2, 0);

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

        let metamoves_2_moves_limit_5 = discover_metamoves(&puzzle, 2, 5);
        assert_eq!(metamoves_2_moves_limit_5.len(), 5);
        assert_eq!(all_metamoves_2_moves[0..5], metamoves_2_moves_limit_5);
        let metamoves_4_moves_limit_1 = discover_metamoves(&puzzle, 4, 1);
        assert_eq!(metamoves_4_moves_limit_1.len(), 1);
        assert_eq!(metamoves_4_moves_limit_1[0].turns, [4, 4]);
        assert_eq!(metamoves_4_moves_limit_1[0].num_affected_pieces, 4);
    }

    #[test]
    fn test_discover_metamoves_pyraminx() {
        let puzzle = puzzles::pyraminx();
        let all_metamoves_4_moves = discover_metamoves(&puzzle, 4, 0);
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
        let all_metamoves_4_moves = discover_metamoves(&puzzle, 4, 0);
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
