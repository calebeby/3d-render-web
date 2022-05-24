mod neural_network;
mod plane;
mod polyhedron;
mod puzzles;
mod quaternion;
mod ray;
mod solver;
mod twisty_puzzle;
mod vector3d;

use crate::twisty_puzzle::PuzzleState;
use csv;

use serde::Deserialize;
use serde::Serialize;
use std::collections::HashMap;
use std::fs::File;
use std::io::Write;
use std::rc::Rc;

#[derive(Serialize, Deserialize)]
struct Record {
    scramble: twisty_puzzle::PuzzleState,
    turns_to_solve: usize,
}

fn main() {
    let puzzle = Rc::new(puzzles::pyraminx());

    let mut states: HashMap<PuzzleState, usize> = HashMap::new();
    let solved_state = puzzle.get_initial_state();

    let depth = 11;

    states.insert(solved_state.clone(), 0);
    let turns: Vec<_> = puzzle.turn_names_iter().collect();
    let mut fringe_stack: Vec<StateToExpand> = vec![StateToExpand {
        puzzle_state: solved_state,
        turn_index: 0,
    }];

    let mut i = 0;
    let mut num_turns = 0;
    let info = std::cmp::max(depth as isize - 6, 1) as usize;
    while let Some(state_to_expand) = fringe_stack.last() {
        if fringe_stack.len() == info {
            i += 1;
            println!(
                "{:.2}%",
                100.0 * i as f64 / turns.len().pow(info as _) as f64
            )
        }
        if fringe_stack.len() < depth + 1 {
            num_turns += 1;
            let derived_state =
                puzzle.get_derived_state(&state_to_expand.puzzle_state, state_to_expand.turn_index);
            let num_moves = fringe_stack.len();
            let saved_solution_turns = states.get(&derived_state);
            match saved_solution_turns {
                Some(saved_solution_turns) if *saved_solution_turns <= num_moves => {
                    // If the saved solution is better than the current solution,
                    // We don't need to expand this state,
                    // because every derived state will also be more optimally solved
                    // using the saved solutions than the current solution
                    increment(&mut fringe_stack, &turns);
                    continue;
                }
                _ => {
                    states.insert(derived_state.clone(), num_moves);
                }
            }
            fringe_stack.push(StateToExpand {
                puzzle_state: derived_state,
                turn_index: 0,
            })
        } else {
            increment(&mut fringe_stack, &turns);
        }
    }

    println!(
        "Actual states: {},\
        \nTheoretical total turn sequences: {},\
        \nActual total turn sequences: {}",
        states.len(),
        turns.len().pow(depth as _),
        num_turns
    );

    let mut file = File::create("training_data/pyraminx.csv").unwrap();
    let mut csv_writer = csv::WriterBuilder::new()
        .has_headers(false)
        .from_writer(vec![]);

    for (puzzle_state, turns_to_solve) in states.into_iter() {
        csv_writer
            .serialize(Record {
                scramble: puzzle_state,
                turns_to_solve,
            })
            .unwrap();
    }
    file.write(&csv_writer.into_inner().unwrap()).unwrap();
}

fn increment(fringe_stack: &mut Vec<StateToExpand>, turns: &Vec<&String>) {
    while let Some(solution_to_increment) = fringe_stack.last_mut() {
        if solution_to_increment.turn_index < turns.len() - 1 {
            solution_to_increment.turn_index += 1;
            break;
        } else {
            fringe_stack.pop();
        }
    }
}

#[derive(Debug)]
struct StateToExpand {
    puzzle_state: PuzzleState,
    turn_index: usize,
}
