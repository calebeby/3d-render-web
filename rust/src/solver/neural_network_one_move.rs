use std::rc::Rc;

use web_sys::console;

use crate::{
    neural_network::{load_model, NeuralNetwork},
    twisty_puzzle::{PuzzleState, TwistyPuzzle},
};

use super::ScrambleSolver;

pub struct NNOneMoveSolver {
    puzzle: Rc<TwistyPuzzle>,
    state: PuzzleState,
    model: NeuralNetwork,
}

impl ScrambleSolver for NNOneMoveSolver {
    type Opts = ();

    fn new(puzzle: Rc<TwistyPuzzle>, initial_state: PuzzleState, _opts: Self::Opts) -> Self {
        Self {
            puzzle,
            state: initial_state,
            model: load_model(),
        }
    }

    fn get_state(&self) -> &PuzzleState {
        &self.state
    }
}

impl Iterator for NNOneMoveSolver {
    type Item = usize;

    fn next(&mut self) -> Option<Self::Item> {
        let current_score = self.model.plugin(&self.state);
        // console::log_1(&format!("Current state score: {}", current_score).into());
        println!("Current state score: {}", current_score);
        // None
        let next_turn = self
            .puzzle
            .turn_names_iter()
            .enumerate()
            .map(|(turn_index, _turn_name)| {
                let next_state = self.puzzle.get_derived_state(&self.state, turn_index);
                let next_state_score = self.model.plugin(&next_state);
                // console::log_1(&format!("Next state score: {}", next_state_score).into());
                println!("Next state {} score: {}", turn_index, next_state_score);
                (turn_index, next_state_score)
            })
            .max_by(|(_, a_score), (_, b_score)| a_score.partial_cmp(b_score).unwrap());

        match next_turn {
            None => None,
            Some((next_turn_name, next_turn_score)) => {
                if current_score >= next_turn_score {
                    None
                } else {
                    Some(next_turn_name.clone())
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::puzzles;

    use super::*;

    #[test]
    fn test_nn_one_move_solver() {
        let puzzle = puzzles::pyraminx();
        let initial_state = puzzle.get_initial_state();
        let mut solver = NNOneMoveSolver::new(Rc::new(puzzle), initial_state, ());
        let solution = solver.collect::<Vec<_>>();
        println!("Solution: {:#?}", solution);
    }
}
