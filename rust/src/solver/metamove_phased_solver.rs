use super::{
    metamoves::{discover_metamoves, MetaMove},
    ScrambleSolver,
};
use crate::{
    solver::bijection_trie::BijectionTrie,
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
    solve_phases: Vec<SolvePhase>,
    current_phase: usize,
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

/// Checks that the metamove preserves the required "preserve piece types"
/// i.e. does not affect pieces of those types
fn metamove_preserves(metamove: &MetaMove, preserve_piece_types: &[&PieceType]) -> bool {
    let puzzle = &metamove.puzzle;
    let derived_state = puzzle.get_derived_state(&puzzle.get_initial_state(), &metamove.face_map);
    for preserve_piece_type in preserve_piece_types {
        let num_pieces_of_type = puzzle.get_num_pieces_of_type(preserve_piece_type);
        let num_unaffected_pieces_of_type =
            puzzle.get_num_solved_pieces_of_type(&derived_state, preserve_piece_type);
        if num_pieces_of_type != num_unaffected_pieces_of_type {
            // At least one piece of the "preserved" type was affected,
            // so it was not preserved
            return false;
        }
    }
    true
}

struct SolvePhase {
    puzzle: Rc<TwistyPuzzle>,
    three_cycle: MetaMove,
    parity_flipper: Option<MetaMove>,
    target_piece_type: PieceType,
    preserve_piece_types: Vec<PieceType>,
}
impl SolvePhase {
    #[inline]
    pub fn new(
        base_metamoves: &[MetaMove],
        target_piece_type: &PieceType,
        preserve_piece_types: &[&PieceType],
        solve_parity: bool,
    ) -> Option<Self> {
        let puzzle = Rc::clone(&base_metamoves[0].puzzle);
        console_log!(
            "puzzle face types {:#?}",
            puzzle
                .piece_types
                .iter()
                .map(|pt| pt.face_mask().iter().filter(|&&fm| fm).count())
                .collect::<Vec<_>>()
        );
        let subset: Vec<MetaMove> = base_metamoves
            .iter()
            .filter(|mm| metamove_preserves(mm, preserve_piece_types))
            .cloned()
            .collect();
        if subset.len() < base_metamoves.len() && !subset.is_empty() {
            console_log!("found shortcut");
            return SolvePhase::new(
                &subset,
                target_piece_type,
                preserve_piece_types,
                solve_parity,
            );
        }
        console_log!("Build trie");
        let mut trie = BijectionTrie::new();

        // TODO: Should the face mask be based on the union of the target and the preserve face types?
        for mm in base_metamoves {
            trie.insert(&mm.face_map.mask(target_piece_type.face_mask()), mm);
        }
        console_log!("Done build trie");
        console_log!("Find most similar");

        // let initial = &candidates[0];
        let mut three_cycle: Option<MetaMove> = None;
        let mut parity_flipper: Option<MetaMove> = None;
        let target_piece_types = [target_piece_type];
        for initial in base_metamoves {
            console_log!("initial {initial:#?}");
            // console_log!("trie size: {}", trie.len());
            // console_log!(
            //     "matches: {:#?}",
            //     trie.find_most_similar(&initial.face_map)
            //         .collect::<Vec<_>>()
            // );
            let combined = trie
                .find_most_similar(&initial.face_map.mask(target_piece_type.face_mask()))
                .find_map(|(differences, most_similar)| {
                    if differences == 0 {
                        return None;
                    }
                    console_log!("most similar {most_similar:#?}");
                    let combined = initial.apply(&most_similar.invert());
                    if metamove_preserves(&combined, preserve_piece_types)
                        && combined.get_num_affected_pieces_of_types(&target_piece_types) > 0
                    {
                        Some(combined)
                    } else {
                        None
                    }
                });
            if combined.is_none() {
                continue;
            }
            let combined = combined.unwrap();
            let num_affected_pieces_of_type =
                combined.get_num_affected_pieces_of_types(&target_piece_types);
            if three_cycle.is_none() && num_affected_pieces_of_type == 3 {
                three_cycle = Some(combined);
                console_log!("three cycle: {:#?}", three_cycle);
            } else if solve_parity
                && parity_flipper.is_none()
                && combined
                    .cycles()
                    .iter()
                    .filter(|cycle| {
                        // All of the faces in a cycle should belong to the same face type,
                        // so we are just using the first one.
                        let is_in_target_face_type = target_piece_type.face_mask()[cycle[0]];
                        // Count the number of even-length cycles in the target face type
                        is_in_target_face_type && cycle.len() % 2 == 0
                    })
                    .count()
                    == 1
            {
                console_log!(
                    "found even parity: {:#?} ({})",
                    combined,
                    num_affected_pieces_of_type
                );
                parity_flipper = Some(combined);
            }
            if three_cycle.is_some() && (!solve_parity || parity_flipper.is_some()) {
                break;
            }
        }

        Some(SolvePhase {
            puzzle,
            three_cycle: three_cycle?,
            parity_flipper,
            target_piece_type: target_piece_type.clone(),
            preserve_piece_types: preserve_piece_types.iter().cloned().cloned().collect(),
        })
    }

    fn next(&self, state: &PuzzleState) -> MetaMove {
        let puzzle = &self.puzzle;
        let mut best_metamove = MetaMove::empty(Rc::clone(puzzle));
        let solved_of_type = puzzle.get_num_solved_pieces_of_type(state, &self.target_piece_type);
        let unsolved_of_type = puzzle.get_num_pieces_of_type(&self.target_piece_type)
            - puzzle.get_num_solved_pieces_of_type(state, &self.target_piece_type);

        console_log!("unsolved_of_type: {}", unsolved_of_type);
        // Even parity; apply parity fix
        if unsolved_of_type == 2 && self.parity_flipper.is_some() {
            return self.parity_flipper.clone().unwrap();
        }

        let individual_turns_metamoves: Vec<MetaMove> = self
            .puzzle
            .turns
            .iter()
            .enumerate()
            .map(|(turn_index, turn)| {
                MetaMove::new(
                    Rc::clone(&self.puzzle),
                    vec![turn_index],
                    turn.face_map.clone(),
                )
            })
            .collect();

        let mut best_score = solved_of_type;

        traverse_combinations(
            &individual_turns_metamoves,
            4,
            MetaMove::empty(Rc::clone(puzzle)),
            |previous_metamove: &MetaMove, new_metamove: &MetaMove| {
                previous_metamove.apply(new_metamove)
            },
            &mut |mm| {
                // Of the form A, B, A', where A is the generated sequence of moves,
                // and B is the three-cycle
                let new_mm = mm.apply(&self.three_cycle).apply(&mm.invert());
                // Try both possibilities, A B A' or B A B'
                let next_state = puzzle.get_derived_state(state, &new_mm.face_map);
                let next_state_score =
                    puzzle.get_num_solved_pieces_of_type(&next_state, &self.target_piece_type);
                if next_state_score > best_score {
                    best_metamove = new_mm;
                    best_score = next_state_score;
                    console_log!("best score {}", best_score);
                    // Stop once we find _anything_ better, not the best one
                    // return TraverseResult::Break;
                }
                if next_state_score == puzzle.get_num_pieces_of_type(&self.target_piece_type) {
                    return TraverseResult::Break;
                }
                TraverseResult::Continue
            },
        );

        best_metamove
    }
}

impl ScrambleSolver for MetaMovePhasedSolver {
    type Opts = ();

    fn new(puzzle: Rc<TwistyPuzzle>, initial_state: PuzzleState, _opts: Self::Opts) -> Self {
        let edges = &puzzle.piece_types[0];
        let corners = &puzzle.piece_types[1];

        // let turn_num_affected_pieces =
        //     MetaMove::new_infer_face_map(Rc::clone(&puzzle), vec![0]).num_affected_pieces;
        console_log!("Initial traverse");
        let metamoves = discover_metamoves(Rc::clone(&puzzle), |_mm| true, 4);
        // TODO: delete
        // SolvePhase::new(&metamoves, corners, &[edges], false);
        let solve_phases = vec![
            SolvePhase::new(&metamoves, corners, &[], true).unwrap(),
            SolvePhase::new(&metamoves, edges, &[corners], false).unwrap(),
        ];
        // console_log!("Discovering metamoves affecting edges but not corners");
        // discover_three_cycle(&metamoves, edges, &[corners]);
        // console_log!("Discovering metamoves affecting corners (ignoring edges)");
        // discover_three_cycle(&metamoves, corners, &[]);
        // console_log!("Discovering metamoves affecting corners but not edges");
        // discover_three_cycle(&metamoves, corners, &[edges]);
        console_log!("Done");

        // console_log!("num metamoves: {}", metamoves.len());
        // let best = metamoves.iter().min().unwrap();
        // console_log!(
        //     "best metamove: {} turns affecting {} pieces",
        //     best.turns.len(),
        //     best.num_affected_pieces
        // );

        // let metamoves: Vec<_> = combine_metamoves(
        //     Rc::clone(&puzzle),
        //     |mm| {
        //         let derived_state =
        //             puzzle.get_derived_state(&puzzle.get_initial_state(), &mm.face_map);
        //         let pieces_outside_type =
        //             puzzle.get_num_pieces() - puzzle.get_num_pieces_of_type(piece_type);
        //         let solved_pieces_outside_type = puzzle.get_num_solved_pieces(&derived_state)
        //             - puzzle.get_num_solved_pieces_of_type(&derived_state, piece_type);
        //         pieces_outside_type == solved_pieces_outside_type
        //     },
        //     &metamoves,
        //     3,
        // );

        Self {
            depth: 2,
            puzzle,
            solve_phases,
            current_phase: 0,
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

        let current_phase = &self.solve_phases[self.current_phase];
        let best_metamove = current_phase.next(&self.state);
        console_log!("adding metamove with {} turns", best_metamove.turns.len());
        if best_metamove.turns.is_empty() && self.current_phase < self.solve_phases.len() - 1 {
            let num_solved = self
                .puzzle
                .get_num_solved_pieces_of_type(&self.state, &current_phase.target_piece_type);
            let num = self
                .puzzle
                .get_num_pieces_of_type(&current_phase.target_piece_type);
            console_log!("Phase {}: {} / {}", self.current_phase, num_solved, num);
            return if num_solved == num {
                self.current_phase += 1;
                console_log!("Phase is now {}", self.current_phase);
                self.next()
            } else {
                console_log!("Done");
                None
            };
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
            // let scrambled_state = puzzle.scramble(&puzzle.get_initial_state(), 20, &mut rng);
            let scrambled_state = puzzle.get_initial_state();
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
