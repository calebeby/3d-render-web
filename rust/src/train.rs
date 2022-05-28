pub(crate) use std::error::Error;

use corgi::array::Array;
use corgi::numbers::Float;
use neural_network::{load_parameters, save_parameters, use_model};
use rand::prelude::SliceRandom;

mod neural_network;
mod plane;
mod polyhedron;
mod quaternion;
mod ray;
mod twisty_puzzle;
mod vector3d;

#[derive(Debug)]
struct Solution {
    state: Vec<Float>,
    turns_to_solve: Float,
}

fn train() -> Result<(), Box<dyn Error>> {
    let rng = &mut rand::thread_rng();
    let mut reader = csv::ReaderBuilder::new()
        .has_headers(false)
        .from_path("training_data/pyraminx.csv")
        .unwrap();

    let batch_size = 1024;
    let input_size = 28;
    let output_size = 1;

    println!("loading data");
    let data: Vec<Solution> = reader
        .records()
        .filter_map(|line| line.ok())
        .map(|line| Solution {
            state: line
                .iter()
                .map(|s| s.parse::<Float>().unwrap())
                .take(input_size)
                .collect(),
            turns_to_solve: line.get(input_size).unwrap().parse::<_>().unwrap(),
        })
        // .take(100000)
        .collect();
    println!("done loading data");

    let mut data_ptrs: Vec<_> = data.iter().collect();

    use_model(
        |layers| {
            let text = &std::fs::read_to_string("learning/pyraminx.json");
            if let Ok(text) = text {
                load_parameters(layers, text)
            } else {
                Ok(())
            }
        },
        |mut model| {
            for epoch in 0..5 {
                println!("Shuffling");
                data_ptrs.shuffle(rng);
                println!("Done shuffling");

                let mut total_loss = 0.0;

                for batch in data_ptrs.chunks_exact(batch_size) {
                    let mut input = vec![];
                    let mut target = vec![];
                    for solution in batch {
                        input.extend_from_slice(&solution.state);
                        target.push(solution.turns_to_solve);
                    }

                    let input = Array::from((vec![batch_size, input_size], input));
                    let target = Array::from((vec![batch_size, output_size], target));
                    let _result = model.forward(input);
                    total_loss += model.backward(target);
                    model.update();
                }
                println!("loss from epoch {epoch}: {}", total_loss);
            }
        },
        |layers| save_parameters(layers),
    )?;

    Ok(())
}

fn main() {
    train().unwrap();
}
