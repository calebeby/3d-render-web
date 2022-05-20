use rand::{thread_rng, Rng};
use web_sys::console;

use crate::twisty_puzzle::{PuzzleState, TwistyPuzzle};

use super::Solver;

pub struct LookaheadSolver {
    depth: usize,
    turns: Vec<String>,
}

impl Solver for LookaheadSolver {
    type Opts = usize;
    fn new(puzzle: &TwistyPuzzle, depth: Self::Opts) -> Self {
        Self {
            depth,
            turns: puzzle.turns_iter().cloned().collect(),
        }
    }

    fn get_next_move(&self, puzzle: &TwistyPuzzle, state: &PuzzleState) -> Option<String> {
        let initial_state = StateWithScore {
            puzzle_state: state.clone(),
            score: puzzle.get_num_solved_pieces(state),
            initial_turn: None,
            most_recent_turn: None,
        };
        let mut fringe: Vec<StateWithScore> = vec![initial_state.clone()];
        let solved_score = puzzle.get_num_pieces();

        if initial_state.score == solved_score {
            return None;
        }

        let mut best = initial_state;
        let num_turns = self.turns.len();

        let mut i = 0;
        while i < self.depth || (best.initial_turn.is_none() && i < self.depth + 1) {
            i += 1;
            let mut new_fringe: Vec<StateWithScore> = Vec::with_capacity(fringe.len() * num_turns);
            for state in &fringe {
                for (turn_index, turn_name) in self.turns.iter().enumerate() {
                    if let Some(most_recent_turn) = state.most_recent_turn {
                        let most_recent_turn_name = &self.turns[most_recent_turn];
                        if (most_recent_turn_name.ends_with("'")
                            && &most_recent_turn_name[0..most_recent_turn_name.len() - 1]
                                == turn_name)
                            || (turn_name.ends_with("'")
                                && &turn_name[0..turn_name.len() - 1] == most_recent_turn_name)
                        {
                            continue;
                        }
                    }
                    let new_state = puzzle.get_derived_state(&state.puzzle_state, turn_name);
                    let new_score = puzzle.get_num_solved_pieces(&new_state);
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
                        return Some(self.turns[new_state_with_score.initial_turn?].clone());
                    }
                    if new_score > best.score {
                        best = new_state_with_score.clone();
                    }
                    new_fringe.push(new_state_with_score);
                }
            }
            fringe = new_fringe
        }

        if best.initial_turn.is_none() {
            console::log_1(&"Random turn!".into());
            let mut rng = thread_rng();
            let i = rng.gen_range(0..num_turns);
            return Some(self.turns[i].clone());
        }

        console::log_1(&format!("Fringe len: {}", fringe.len()).into());
        Some(self.turns[best.initial_turn?].clone())
    }
}

#[derive(Debug, Clone)]
struct StateWithScore {
    puzzle_state: PuzzleState,
    score: usize,
    initial_turn: Option<usize>,
    most_recent_turn: Option<usize>,
}
