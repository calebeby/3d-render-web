use super::{
    metamoves::{discover_metamoves, MetaMove},
    ScrambleSolver,
};
use crate::{
    solver::metamoves::combine_metamoves,
    traverse_combinations::{traverse_combinations, TraverseResult},
    twisty_puzzle::{PieceType, PuzzleState, TwistyPuzzle},
};
use std::{
    collections::{hash_map::Entry, HashMap, VecDeque},
    rc::Rc,
};

pub struct MetaMovePhasedSolver {
    puzzle: Rc<TwistyPuzzle>,
    state: PuzzleState,
    depth: usize,
    metamoves: Vec<MetaMove>,
    queued_turns: VecDeque<usize>,
}

macro_rules! console_log {
    ($($t:tt)*) => {
        #[cfg(target_arch = "wasm32")] {
            web_sys::console::log_1(&format!($($t)*).into());
        }
        #[cfg(not(target_arch = "wasm32"))] {
            println!($($t)*);
        }
    };
}

// fn discover_three_cycle(puzzle: &TwistyPuzzle, base_metamoves: &[MetaMove], target_piece_type: &PieceType, preserve_piece_type: &PieceType) -> MetaMove {
// }

impl ScrambleSolver for MetaMovePhasedSolver {
    type Opts = ();

    fn new(puzzle: Rc<TwistyPuzzle>, initial_state: PuzzleState, _opts: Self::Opts) -> Self {
        let edges = &puzzle.piece_types[1];
        let corners = &puzzle.piece_types[2];
        let piece_type = corners;

        let turn_num_affected_pieces =
            MetaMove::new_infer_face_map(Rc::clone(&puzzle), vec![0]).num_affected_pieces;
        let metamoves = discover_metamoves(
            Rc::clone(&puzzle),
            |mm| mm.num_affected_pieces < turn_num_affected_pieces,
            4,
        );

        console_log!("num metamoves: {}", metamoves.len());
        let best = metamoves.iter().min().unwrap();
        console_log!(
            "best metamove: {} turns affecting {} pieces",
            best.turns.len(),
            best.num_affected_pieces
        );

        let metamoves: Vec<_> = combine_metamoves(
            Rc::clone(&puzzle),
            |mm| {
                let derived_state =
                    puzzle.get_derived_state(&puzzle.get_initial_state(), &mm.face_map);
                let pieces_outside_type =
                    puzzle.get_num_pieces() - puzzle.get_num_pieces_of_type(piece_type);
                let solved_pieces_outside_type = puzzle.get_num_solved_pieces(&derived_state)
                    - puzzle.get_num_solved_pieces_of_type(&derived_state, piece_type);
                pieces_outside_type == solved_pieces_outside_type
            },
            &metamoves,
            3,
        );
        console_log!("after combining, num metamoves: {}", metamoves.len());
        let metamoves = filter_duplicates(metamoves);
        let best = metamoves.iter().min().unwrap();
        console_log!(
            "best metamove: {} turns affecting {} pieces",
            best.turns.len(),
            best.num_affected_pieces
        );
        console_log!("deduped num metamoves: {}", metamoves.len());

        let metamoves: Vec<_> = combine_metamoves(
            Rc::clone(&puzzle),
            |mm| {
                let derived_state =
                    puzzle.get_derived_state(&puzzle.get_initial_state(), &mm.face_map);
                let pieces_outside_type =
                    puzzle.get_num_pieces() - puzzle.get_num_pieces_of_type(piece_type);
                let solved_pieces_outside_type = puzzle.get_num_solved_pieces(&derived_state)
                    - puzzle.get_num_solved_pieces_of_type(&derived_state, piece_type);
                pieces_outside_type == solved_pieces_outside_type
            },
            &metamoves,
            3,
        );
        console_log!("after combining, num metamoves: {}", metamoves.len());
        let metamoves = filter_duplicates(metamoves);
        let best = metamoves.iter().min().unwrap();
        console_log!(
            "best metamove: {} turns affecting {} pieces",
            best.turns.len(),
            best.num_affected_pieces
        );
        let best_num_affected_pieces = best.num_affected_pieces;
        let metamoves = metamoves
            .into_iter()
            .filter(|mm| mm.num_affected_pieces <= best_num_affected_pieces + 1)
            .collect::<Vec<_>>();
        console_log!("deduped num metamoves: {}", metamoves.len());

        Self {
            depth: 2,
            metamoves,
            puzzle,
            state: initial_state,
            queued_turns: VecDeque::new(),
        }
    }

    fn get_state(&self) -> &PuzzleState {
        &self.state
    }
}

impl Iterator for MetaMovePhasedSolver {
    type Item = usize;

    fn next(&mut self) -> Option<Self::Item> {
        if !self.queued_turns.is_empty() {
            let next_turn = self.queued_turns.pop_front().unwrap();
            self.state = self
                .puzzle
                .get_derived_state_turn_index(&self.state, next_turn);
            return Some(next_turn);
        }

        let edges = &self.puzzle.piece_types[1];
        let corners = &self.puzzle.piece_types[2];
        let piece_type = corners;

        let best_metamove = find_best_metamove(
            Rc::clone(&self.puzzle),
            &self.state,
            &self.metamoves,
            piece_type,
            self.depth,
        );
        console_log!("adding metamove with {} turns", best_metamove.turns.len());
        if best_metamove.turns.is_empty() {
            console_log!(
                "done solving, solved {} / {}",
                self.puzzle
                    .get_num_solved_pieces_of_type(&self.state, piece_type),
                self.puzzle.get_num_pieces_of_type(piece_type)
            );
        }
        if best_metamove.turns.is_empty() {
            None
        } else {
            self.queued_turns.extend(best_metamove.turns);
            self.next()
        }
    }
}

fn find_best_metamove(
    puzzle: Rc<TwistyPuzzle>,
    state: &PuzzleState,
    metamoves: &[MetaMove],
    piece_type: &PieceType,
    depth: usize,
) -> MetaMove {
    let mut best_metamove = MetaMove::empty(Rc::clone(&puzzle));
    let mut best_score = puzzle.get_num_solved_pieces_of_type(state, piece_type);

    traverse_combinations(
        metamoves,
        depth,
        MetaMove::empty(Rc::clone(&puzzle)),
        |previous_metamove: &MetaMove, new_metamove: &MetaMove| {
            previous_metamove.apply(new_metamove)
        },
        &mut |mm| {
            let next_state = puzzle.get_derived_state(state, &mm.face_map);
            let next_state_score = puzzle.get_num_solved_pieces_of_type(&next_state, piece_type);
            if next_state_score > best_score {
                best_metamove = mm.clone();
                best_score = next_state_score;
                // Stop once we find _anything_ better, not the best one
                return TraverseResult::Break;
            }
            if next_state_score == puzzle.get_num_pieces_of_type(piece_type) {
                return TraverseResult::Break;
            }
            TraverseResult::Continue
        },
    );

    best_metamove
}

fn filter_duplicates(metamoves: Vec<MetaMove>) -> Vec<MetaMove> {
    let mut metamoves_reduced = HashMap::new();
    for mm in metamoves {
        let entry = metamoves_reduced.entry(mm.face_map.clone());

        match entry {
            Entry::Vacant(entry) => {
                entry.insert(mm);
            }
            Entry::Occupied(mut entry) => {
                if entry.get().turns.len() > mm.turns.len() {
                    entry.insert(mm);
                }
            }
        }
    }

    metamoves_reduced.into_values().collect()
}

#[cfg(test)]
mod tests {
    use rand::SeedableRng;
    use rand_chacha::ChaCha8Rng;

    use super::*;
    use crate::puzzles;

    #[test]
    fn solve_rubiks_3x3() {
        let puzzle = Rc::new(puzzles::rubiks_cube_3x3());

        let mut rng = ChaCha8Rng::seed_from_u64(1);

        let mut sum = 0;
        let mut num_solves = 0;
        let num_scrambles = 10;
        // Baseline: 30/50, took 1m 57s to run:
        //
        // 3x3 solution length: 260 turns
        // avg 3x3 solution length: 384.6 turns, (30 / 50)
        for _ in 0..num_scrambles {
            let scrambled_state = puzzle.scramble(&puzzle.get_initial_state(), 20, &mut rng);
            let solution: Vec<_> =
                MetaMovePhasedSolver::new(Rc::clone(&puzzle), scrambled_state.clone(), ())
                    .collect();

            let out = puzzle
                .get_derived_state_from_turn_sequence(&scrambled_state, solution.iter().cloned());

            if out != puzzle.get_initial_state() {
                println!("It did not solve it")
            } else {
                sum += solution.len();
                num_solves += 1;
            }

            println!("3x3 solution length: {} turns", solution.len());
        }

        println!(
            "avg 3x3 solution length: {} turns, ({} / {})",
            sum as f64 / num_solves as f64,
            num_solves,
            num_scrambles
        );
    }
}
