pub(crate) use std::error::Error;
use std::iter;

use plotlib::page::Page;
use plotlib::repr::Plot;
use plotlib::style::PointStyle;
use plotlib::view::ContinuousView;

use corgi::array::Array;
use corgi::numbers::Float;
use neural_network::{load_parameters, normalize_output, save_parameters, use_model};
use rand::{prelude::SliceRandom, Rng};

mod neural_network;
mod plane;
mod polyhedron;
mod quaternion;
mod ray;
mod twisty_puzzle;
mod vector3d;

#[derive(Debug, Clone)]
struct Scramble {
    state: Vec<Float>,
    turns_to_solve: usize,
}

fn train() -> Result<(), Box<dyn Error>> {
    let rng = &mut rand::thread_rng();
    let mut reader = csv::ReaderBuilder::new()
        .has_headers(false)
        .from_path("training_data/pyraminx.csv")
        .unwrap();

    let input_size = 28;
    let output_size = 16;

    let mut turns_to_solve_distribution = vec![0; 40];

    println!("loading data");
    let raw_data: Vec<Scramble> = reader
        .records()
        .filter_map(|line| line.ok())
        .map(|line| {
            let turns_to_solve = line.get(input_size).unwrap().parse::<_>().unwrap();
            turns_to_solve_distribution[turns_to_solve] += 1;
            Scramble {
                state: line
                    .iter()
                    .map(|s| s.parse::<Float>().unwrap())
                    .take(input_size)
                    .collect(),
                turns_to_solve,
            }
        })
        .collect();

    let highest_turns_to_solve = *turns_to_solve_distribution.iter().max().unwrap();

    // Normalized by the number of turns to solve
    let data: Vec<Scramble> = raw_data
        .into_iter()
        .map(|scramble| {
            // This scramble should appear N times
            // so that each number of turns to solve
            // happens roughly the same number of times
            let n = highest_turns_to_solve / turns_to_solve_distribution[scramble.turns_to_solve];
            iter::repeat(scramble).take(n)
        })
        .flatten()
        .collect();

    let batch_size = 300;
    let batches_per_epoch = 25;

    for num_to_solve in 0..10 {
        let n = data
            .iter()
            .filter(|s| s.turns_to_solve == num_to_solve)
            .count();
        println!("{num_to_solve} to solve: {n}");
    }
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
            let mut loss_over_time = vec![];
            for epoch in 0..2500 {
                let mut plot_points = vec![];
                println!("Shuffling");
                let (shuffled, _) = data_ptrs.partial_shuffle(rng, batches_per_epoch * batch_size);
                println!("Done shuffling");

                let mut total_loss = 0.0;

                for batch in shuffled.chunks_exact(batch_size).take(batches_per_epoch) {
                    let mut input = vec![];
                    let mut target = vec![];
                    for scramble in batch {
                        input.extend_from_slice(&scramble.state);
                        let mut scramble_target = vec![0.0; output_size];
                        scramble_target[scramble.turns_to_solve] = 1.0;
                        target.extend_from_slice(&scramble_target);
                    }

                    let input = Array::from((vec![batch_size, input_size], input));
                    let target = Array::from((vec![batch_size, output_size], target));
                    let result = model.forward(input);
                    for i in 0..batch_size {
                        let mut target_row = vec![];
                        let mut result_row = vec![];
                        for j in 0..output_size {
                            target_row.push(target[vec![i, j]]);
                            result_row.push(result[vec![i, j]]);
                        }
                        plot_points
                            .push((normalize_output(target_row), normalize_output(result_row)));
                    }
                    total_loss += model.backward(target);
                    model.update();
                }
                plot(plot_points, "scatter.svg");
                println!(
                    "loss from epoch {epoch}: {}",
                    total_loss / batches_per_epoch as f64
                );
                loss_over_time.push((epoch as f64, total_loss / batches_per_epoch as f64));
                plot(loss_over_time.clone(), "loss_over_time.svg");
            }
        },
        |layers| save_parameters(layers),
    )?;

    Ok(())
}

fn plot(data: Vec<(Float, Float)>, name: &str) -> Option<()> {
    if data.len() < 2 {
        return None;
    }
    let max_x = data
        .iter()
        .max_by(|(x1, _), (x2, _)| x1.partial_cmp(x2).unwrap_or(std::cmp::Ordering::Equal))?
        .0;
    let max_y = data
        .iter()
        .max_by(|(_, y1), (_, y2)| y1.partial_cmp(y2).unwrap_or(std::cmp::Ordering::Equal))?
        .1;
    let s1: Plot = Plot::new(data).point_style(PointStyle::new().size(0.8));
    let v = ContinuousView::new()
        .add(s1)
        .x_range(0.0, max_x * 1.1)
        .y_range(0.0, max_y * 1.1);
    Page::single(&v).save(name).unwrap_or(());
    Some(())
}

fn main() {
    train().unwrap();
}
