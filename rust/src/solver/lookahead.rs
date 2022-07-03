use std::rc::Rc;

use crate::twisty_puzzle::{PuzzleState, TwistyPuzzle};

use super::ScrambleSolver;

pub struct LookaheadSolver {
    state: PuzzleState,
    puzzle: Rc<TwistyPuzzle>,
    turns: Vec<String>,
    opts: LookaheadSolverOpts,
}

#[derive(Clone)]
pub struct LookaheadSolverOpts {
    pub depth: usize,
}

impl ScrambleSolver for LookaheadSolver {
    type Opts = LookaheadSolverOpts;

    fn new(puzzle: Rc<TwistyPuzzle>, initial_state: PuzzleState, opts: Self::Opts) -> Self {
        Self {
            state: initial_state,
            turns: puzzle.turn_names_iter().cloned().collect(),
            puzzle,
            opts,
        }
    }

    fn get_state(&self) -> &PuzzleState {
        &self.state
    }
}

impl Iterator for LookaheadSolver {
    type Item = usize;

    fn next(&mut self) -> Option<Self::Item> {
        let initial_state = StateWithScore {
            puzzle_state: self.state.clone(),
            score: self.puzzle.get_num_solved_pieces(&self.state),
            initial_turn: None,
            most_recent_turn: None,
        };
        let mut fringe: Vec<StateWithScore> = vec![initial_state.clone()];
        let solved_score = self.puzzle.get_num_pieces();

        if initial_state.score == solved_score {
            return None;
        }

        let mut best = initial_state;
        let num_turns = self.turns.len();

        let mut i = 0;
        while i < self.opts.depth || (best.initial_turn.is_none() && i < self.opts.depth + 1) {
            i += 1;
            let mut new_fringe: Vec<StateWithScore> = Vec::with_capacity(fringe.len() * num_turns);
            for state in &fringe {
                for (turn_index, turn_name) in self.turns.iter().enumerate() {
                    if let Some(most_recent_turn) = state.most_recent_turn {
                        let most_recent_turn_name = &self.turns[most_recent_turn];
                        if (most_recent_turn_name.ends_with('\'')
                            && &most_recent_turn_name[0..most_recent_turn_name.len() - 1]
                                == turn_name)
                            || (turn_name.ends_with('\'')
                                && &turn_name[0..turn_name.len() - 1] == most_recent_turn_name)
                        {
                            continue;
                        }
                    }
                    let new_state = self
                        .puzzle
                        .get_derived_state_turn_index(&state.puzzle_state, turn_index);
                    let new_score = self.puzzle.get_num_solved_pieces(&new_state);
                    let new_state_with_score = StateWithScore {
                        initial_turn: match state.initial_turn {
                            None => Some(turn_index),
                            v => v,
                        },
                        most_recent_turn: Some(turn_index),
                        puzzle_state: new_state,
                        score: new_score,
                    };
                    if new_score == solved_score {
                        return new_state_with_score.initial_turn;
                    }
                    if new_score > best.score {
                        best = new_state_with_score.clone();
                    }
                    new_fringe.push(new_state_with_score);
                }
            }
            fringe = new_fringe;
        }

        self.state = best.puzzle_state;

        best.initial_turn
    }
}

#[derive(Debug, Clone)]
struct StateWithScore {
    puzzle_state: PuzzleState,
    score: usize,
    initial_turn: Option<usize>,
    most_recent_turn: Option<usize>,
}
