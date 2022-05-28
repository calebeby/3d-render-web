use std::rc::Rc;

use web_sys::console;

use crate::neural_network::{load_parameters_static, use_model};
use crate::twisty_puzzle::{PuzzleState, TwistyPuzzle};
use corgi::array::Array;

use super::ScrambleSolver;

pub struct NNOneMoveSolver {
    puzzle: Rc<TwistyPuzzle>,
    state: PuzzleState,
}

impl NNOneMoveSolver {
    fn evaluate_state(&self, state: &PuzzleState) -> f64 {
        use_model(
            |layers| load_parameters_static(layers),
            |mut model| {
                let input = Array::from((
                    vec![1, state.len()],
                    state.iter().map(|x| *x as f64).collect::<Vec<f64>>(),
                ));
                model.forward(input).values()[0]
            },
            |_| Ok(()),
        )
        .unwrap()
    }
}

impl ScrambleSolver for NNOneMoveSolver {
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

impl Iterator for NNOneMoveSolver {
    type Item = usize;

    fn next(&mut self) -> Option<Self::Item> {
        let current_score = self.evaluate_state(&self.state);
        console::log_1(&format!("Current state: {:?}", &self.state).into());
        console::log_1(&format!("Current state score: {}", current_score).into());
        let next_turn = self
            .puzzle
            .turn_names_iter()
            .enumerate()
            .map(|(turn_index, _turn_name)| {
                let next_state = self.puzzle.get_derived_state(&self.state, turn_index);
                let next_state_score = self.evaluate_state(&next_state);
                console::log_1(&format!("Next state score: {}", next_state_score).into());
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
