use crate::twisty_puzzle::{PuzzleState, TwistyPuzzle};
mod lookahead;
mod simple_one_move;
pub use lookahead::LookaheadSolver;
pub use simple_one_move::OneMoveSolver;

pub trait Solver {
    fn new(puzzle: &TwistyPuzzle) -> Self;
    fn get_next_move(&self, puzzle: &TwistyPuzzle, state: &PuzzleState) -> Option<String>;
}
