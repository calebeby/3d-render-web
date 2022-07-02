use std::{collections::VecDeque, rc::Rc};

use crate::twisty_puzzle::{PuzzleState, TwistyPuzzle};

use super::ScrambleSolver;

pub struct FullSearchSolver {
    puzzle: Rc<TwistyPuzzle>,
    state: PuzzleState,
    solution: VecDeque<usize>,
}

#[derive(Clone)]
pub struct FullSearchSolverOpts {
    pub depth: usize,
}

impl ScrambleSolver for FullSearchSolver {
    type Opts = FullSearchSolverOpts;

    fn new(puzzle: Rc<TwistyPuzzle>, initial_state: PuzzleState, opts: Self::Opts) -> Self {
        let num_turns = puzzle.turn_names_iter().count();
        let mut fringe_stack_max_size = opts.depth + 1;
        let mut fringe_stack: Vec<StateToExpand> = vec![StateToExpand {
            puzzle_state: initial_state.clone(),
            turn_index: 0,
        }];
        let solved_score = puzzle.get_num_pieces();

        let mut best = BestSolution {
            num_moves: 0,
            score: puzzle.get_num_solved_pieces(&initial_state),
            turns: vec![],
        };
        if best.score == solved_score {
            return Self {
                solution: VecDeque::new(),
                state: initial_state,
                puzzle,
            };
        }

        while let Some(state_to_expand) = fringe_stack.last() {
            if fringe_stack.len() < fringe_stack_max_size {
                let derived_state = puzzle
                    .get_derived_state(&state_to_expand.puzzle_state, state_to_expand.turn_index);
                let score = puzzle.get_num_solved_pieces(&derived_state);
                let num_moves = fringe_stack.len();
                if score > best.score || (score == best.score && num_moves < best.num_moves) {
                    best = BestSolution {
                        num_moves,
                        score,
                        turns: fringe_stack.iter().map(|t| t.turn_index).collect(),
                    }
                }
                if score == solved_score {
                    fringe_stack_max_size = fringe_stack.len();
                }
                fringe_stack.push(StateToExpand {
                    puzzle_state: derived_state,
                    turn_index: 0,
                })
            } else {
                while let Some(solution_to_increment) = fringe_stack.last_mut() {
                    if solution_to_increment.turn_index < num_turns - 1 {
                        solution_to_increment.turn_index += 1;
                        break;
                    } else {
                        fringe_stack.pop();
                    }
                }
            }
        }

        let solution: VecDeque<_> = best.turns.into();

        Self {
            solution,
            state: initial_state,
            puzzle,
        }
    }

    fn get_state(&self) -> &PuzzleState {
        &self.state
    }
}

impl Iterator for FullSearchSolver {
    type Item = usize;

    fn next(&mut self) -> Option<Self::Item> {
        let turn = self.solution.pop_front()?;
        self.state = self.puzzle.get_derived_state(&self.state, turn);
        Some(turn)
    }
}

#[derive(Debug)]
struct StateToExpand {
    puzzle_state: PuzzleState,
    turn_index: usize,
}

#[derive(Debug, Clone)]
struct BestSolution {
    num_moves: usize,
    score: usize,
    turns: Vec<usize>,
}
