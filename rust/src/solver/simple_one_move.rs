use crate::twisty_puzzle::{PuzzleState, TwistyPuzzle};

use super::Solver;

pub struct OneMoveSolver {}

impl Solver for OneMoveSolver {
    type Opts = ();
    fn new(_puzzle: &TwistyPuzzle, _opts: Self::Opts) -> Self {
        Self {}
    }

    fn get_next_move(&self, puzzle: &TwistyPuzzle, state: &PuzzleState) -> Option<String> {
        let next_turn = puzzle
            .turns_iter()
            .map(|turn_name| {
                let next_state = puzzle.get_derived_state(state, turn_name);
                let next_state_score = puzzle.get_num_solved_pieces(&next_state);
                (turn_name, next_state_score)
            })
            .max_by_key(|(_, score)| *score);

        match next_turn {
            None => None,
            Some((next_turn_name, next_turn_score)) => {
                let current_score = puzzle.get_num_solved_pieces(state);
                if current_score >= next_turn_score {
                    None
                } else {
                    Some(next_turn_name.clone())
                }
            }
        }
    }
}
