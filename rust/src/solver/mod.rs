use std::rc::Rc;

use crate::twisty_puzzle::{PuzzleState, TwistyPuzzle};
mod bijection_trie;
mod full_search_solve;
mod lookahead;
mod metamove_solver;
mod metamoves;
mod simple_one_move;
pub use full_search_solve::{FullSearchSolver, FullSearchSolverOpts};
pub use lookahead::{LookaheadSolver, LookaheadSolverOpts};
pub use metamove_solver::MetaMoveSolver;
pub use simple_one_move::OneMoveSolver;

pub struct Solver<T: ScrambleSolver> {
    opts: T::Opts,
    puzzle: Rc<TwistyPuzzle>,
}

impl<T: ScrambleSolver> Solver<T> {
    pub fn new(puzzle: Rc<TwistyPuzzle>, opts: T::Opts) -> Self {
        Self { opts, puzzle }
    }
    pub fn solve(&self, initial_state: PuzzleState) -> T {
        T::new(self.puzzle.clone(), initial_state, self.opts.clone())
    }
}

pub trait ScrambleSolver: Iterator<Item = usize> {
    type Opts: Clone;
    fn new(puzzle: Rc<TwistyPuzzle>, initial_state: PuzzleState, opts: Self::Opts) -> Self;
    fn get_state(&self) -> &PuzzleState;
}
