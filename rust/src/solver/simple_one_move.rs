use std::rc::Rc;

use crate::twisty_puzzle::{PuzzleState, TwistyPuzzle};

use super::ScrambleSolver;

pub struct OneMoveSolver {
    puzzle: Rc<TwistyPuzzle>,
    state: PuzzleState,
}

impl ScrambleSolver for OneMoveSolver {
    type Opts = ();

    fn new(puzzle: Rc<TwistyPuzzle>, initial_state: PuzzleState, opts: Self::Opts) -> Self {
        Self {
            puzzle,
            state: initial_state,
        }
    }

    fn get_state(&self) -> &PuzzleState {
        &self.state
    }
}

impl Iterator for OneMoveSolver {
    type Item = String;

    fn next(&mut self) -> Option<Self::Item> {
        let next_turn = self
            .puzzle
            .turns_iter()
            .map(|turn_name| {
                let next_state = self.puzzle.get_derived_state(&self.state, turn_name);
                let next_state_score = self.puzzle.get_num_solved_pieces(&next_state);
                (turn_name, next_state_score)
            })
            .max_by_key(|(_, score)| *score);

        match next_turn {
            None => None,
            Some((next_turn_name, next_turn_score)) => {
                let current_score = self.puzzle.get_num_solved_pieces(&self.state);
                if current_score >= next_turn_score {
                    None
                } else {
                    Some(next_turn_name.clone())
                }
            }
        }
    }
}
