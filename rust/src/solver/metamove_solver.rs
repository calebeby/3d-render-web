use super::{
    metamoves::{discover_metamoves, MetaMove},
    ScrambleSolver,
};
use crate::twisty_puzzle::{PuzzleState, TwistyPuzzle};
use std::{collections::VecDeque, rc::Rc};
use web_sys::console;

pub struct MetaMoveSolver {
    puzzle: Rc<TwistyPuzzle>,
    state: PuzzleState,
    metamoves: Vec<MetaMove>,
    buffered_turns: VecDeque<usize>,
}

impl ScrambleSolver for MetaMoveSolver {
    type Opts = ();

    fn new(puzzle: Rc<TwistyPuzzle>, initial_state: PuzzleState, _opts: Self::Opts) -> Self {
        Self {
            metamoves: discover_metamoves(&puzzle, 4, 200),
            puzzle,
            state: initial_state,
            buffered_turns: VecDeque::new(),
        }
    }

    fn get_state(&self) -> &PuzzleState {
        &self.state
    }
}

impl Iterator for MetaMoveSolver {
    type Item = usize;

    fn next(&mut self) -> Option<Self::Item> {
        console::log_1(&format!("num metamoves: {}", self.metamoves.len()).into());
        if !self.buffered_turns.is_empty() {
            let next_turn = self.buffered_turns.pop_front().unwrap();
            self.state = self
                .puzzle
                .get_derived_state_turn_index(&self.state, next_turn);
            return Some(next_turn);
        }

        let current_score = self.puzzle.get_num_solved_pieces(&self.state);

        let best_metamove = self
            .metamoves
            .iter()
            .filter_map(|mm| {
                let next_state = self.puzzle.get_derived_state(&self.state, &mm.face_map);
                let next_state_score = self.puzzle.get_num_solved_pieces(&next_state);
                if next_state_score > current_score {
                    Some((mm, next_state_score))
                } else {
                    None
                }
            })
            .max_by_key(|(_, score)| *score);

        let best_metamove_score = match best_metamove {
            Some((_, score)) => score,
            None => 0,
        };

        let best_single_turn = self
            .puzzle
            .turns
            .iter()
            .enumerate()
            .filter_map(|(turn_index, turn)| {
                let next_state = self.puzzle.get_derived_state(&self.state, &turn.face_map);
                let next_state_score = self.puzzle.get_num_solved_pieces(&next_state);
                if next_state_score > current_score && next_state_score > best_metamove_score {
                    Some((turn_index, next_state_score))
                } else {
                    None
                }
            })
            .max_by_key(|(_, score)| *score);

        if let Some((best_single_turn_index, _)) = best_single_turn {
            console::log_1(&"applying single turn".into());
            self.state = self
                .puzzle
                .get_derived_state_turn_index(&self.state, best_single_turn_index);
            Some(best_single_turn_index)
        } else if let Some((best_metamove, _)) = best_metamove {
            console::log_1(
                &format!("applying metamoves {} turns", best_metamove.turns.len()).into(),
            );
            let first_turn = best_metamove.turns[0];
            self.state = self
                .puzzle
                .get_derived_state_turn_index(&self.state, first_turn);
            self.buffered_turns.clear();
            for turn in &best_metamove.turns[1..] {
                self.buffered_turns.push_back(*turn)
            }
            Some(first_turn)
        } else {
            None
        }
    }
}
