use super::ScrambleSolver;
use crate::twisty_puzzle::{PuzzleState, TwistyPuzzle};
use std::rc::Rc;

pub struct MetaMoveSolver {
    puzzle: Rc<TwistyPuzzle>,
    state: PuzzleState,
}

impl ScrambleSolver for MetaMoveSolver {
    type Opts = ();

    fn new(puzzle: Rc<TwistyPuzzle>, initial_state: PuzzleState, _opts: Self::Opts) -> Self {
        Self {
            puzzle,
            state: initial_state,
        }
    }

    fn get_state(&self) -> &PuzzleState {
        &self.state
    }
}

impl Iterator for MetaMoveSolver {
    type Item = usize;

    fn next(&mut self) -> Option<Self::Item> {
        let current_score = self.puzzle.get_num_solved_pieces(&self.state);
        let (next_turn_index, _, next_state) = self
            .puzzle
            .turns
            .iter()
            .enumerate()
            .filter_map(|(turn_index, turn)| {
                let next_state = self.puzzle.get_derived_state(&self.state, &turn.face_map);
                let next_state_score = self.puzzle.get_num_solved_pieces(&next_state);
                if next_state_score > current_score {
                    Some((turn_index, next_state_score, next_state))
                } else {
                    None
                }
            })
            .max_by_key(|(_, score, _)| *score)?;

        self.state = next_state;

        Some(next_turn_index)
    }
}
