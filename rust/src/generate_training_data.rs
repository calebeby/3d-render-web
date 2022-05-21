mod plane;
mod polyhedron;
mod puzzles;
mod quaternion;
mod ray;
mod solver;
mod twisty_puzzle;
mod vector3d;

use csv;
use rand::thread_rng;
use rand::Rng;
use serde::Deserialize;
use serde::Serialize;
use solver::LookaheadSolver;
use solver::Solver;
use std::io::Write;

#[derive(Serialize, Deserialize)]
struct Record {
    scramble: twisty_puzzle::PuzzleState,
    turns_to_solve: usize,
}

fn main() {
    let puzzle = puzzles::rubiks_cube_2x2();
    let depth = 14;

    let solver = LookaheadSolver::new(&puzzle, depth);
    let mut rng = thread_rng();
    let mut file = std::fs::OpenOptions::new()
        .append(true)
        .open("2x2_training_data.csv")
        .unwrap();

    for i in 0..10000 {
        let mut csv_writer = csv::WriterBuilder::new()
            .has_headers(false)
            .from_writer(vec![]);
        println!("\n\niteration {}", i);

        let scramble_turns = rng.gen_range(20..=30);

        let initial_state = puzzle.scramble(&puzzle.get_initial_state(), scramble_turns);
        println!(
            "Initial: {:.1}% solved",
            puzzle.get_num_solved_pieces(&initial_state) as f64 / puzzle.get_num_pieces() as f64
                * 100.0
        );
        let turns: Vec<_> = solver.next_move_iter(&puzzle, &initial_state).collect();
        println!("turns: {:?}", turns);
        println!(
            "scramble: {} turns, solve: {} turns",
            scramble_turns,
            turns.len()
        );
        let end_state = puzzle.get_derived_state_from_turns_iter(&initial_state, turns.iter());
        println!(
            "Final:   {:.1}% solved",
            puzzle.get_num_solved_pieces(&end_state) as f64 / puzzle.get_num_pieces() as f64
                * 100.0
        );

        csv_writer
            .serialize(Record {
                scramble: initial_state.clone(),
                turns_to_solve: turns.len(),
            })
            .unwrap();

        let mut num_turns_left = turns.len();
        let mut state = initial_state;
        for turn in &turns {
            state = puzzle.get_derived_state(&state, &turn);
            num_turns_left -= 1;
            csv_writer
                .serialize(Record {
                    scramble: state.clone(),
                    turns_to_solve: num_turns_left,
                })
                .unwrap();
        }
        file.write(&csv_writer.into_inner().unwrap()).unwrap();
    }
}
