use crate::twisty_puzzle::{PuzzleState, TwistyPuzzle};
mod lookahead;
mod simple_one_move;
pub use lookahead::LookaheadSolver;
pub use simple_one_move::OneMoveSolver;

pub trait Solver {
    fn new(puzzle: &TwistyPuzzle) -> Self
    where
        Self: Sized;
    fn get_next_move(&self, puzzle: &TwistyPuzzle, state: &PuzzleState) -> Option<String>;
    fn next_move_iter<'a>(
        &'a self,
        puzzle: &'a TwistyPuzzle,
        state: &PuzzleState,
    ) -> SolveIterator<'a>
    where
        Self: Sized,
    {
        SolveIterator {
            puzzle,
            puzzle_state: state.clone(),
            solver: self,
        }
    }
}

pub struct SolveIterator<'a> {
    puzzle: &'a TwistyPuzzle,
    puzzle_state: PuzzleState,
    solver: &'a dyn Solver,
}

impl Iterator for SolveIterator<'_> {
    type Item = String;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(turn_name) = self.solver.get_next_move(&self.puzzle, &self.puzzle_state) {
            self.puzzle_state = self
                .puzzle
                .get_derived_state(&self.puzzle_state, &turn_name);
            Some(turn_name)
        } else {
            None
        }
    }
}
