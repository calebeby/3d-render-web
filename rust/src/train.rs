mod neural_network;
mod plane;
mod polyhedron;
mod quaternion;
mod ray;
mod twisty_puzzle;
mod vector3d;

use ndarray::{Array2, ArrayBase, Dim, OwnedRepr, ViewRepr};
use neuronika::{data::DataLoader, nn::loss, optim};

use crate::neural_network::{NeuralNetwork, OUT_SIZE};

fn main() {
    println!("loading data");
    let data = DataLoader::default()
        .with_labels(&[28])
        .with_delimiter(',')
        .from_csv("training_data/pyraminx.csv", 28, 1);

    println!("done loading data");

    let model = NeuralNetwork::new();

    // let optimizer = optim::SGD::new(model.parameters(), 0.00001, optim::L2::new(0.00));
    let optimizer = optim::Adam::new(
        model.parameters(),
        0.00001,
        (0.9, 0.999),
        optim::L2::new(0.0),
        1e-8,
    );

    fn convert_target(
        target: &ArrayBase<ViewRepr<&f32>, Dim<[usize; 2]>>,
    ) -> ArrayBase<OwnedRepr<f32>, Dim<[usize; 2]>> {
        let mut out: ArrayBase<OwnedRepr<f32>, Dim<[usize; 2]>> =
            Array2::zeros((target.nrows(), OUT_SIZE));
        for (i, target_val) in target.iter().enumerate() {
            out[[i, *target_val as usize]] = 1.0;
        }
        out
    }

    model.train();
    for epoch in 0..10 {
        println!("Batching data");
        let batched_data = data.batch(50).drop_last();
        println!("Done batching data");
        let mut total_loss: f32 = 0.0;

        for (input_array, target_array) in batched_data {
            let input = neuronika::from_ndarray(input_array.to_owned());
            let converted_target = convert_target(&target_array);
            let target = neuronika::from_ndarray(converted_target);

            let result = model.forward(input);

            let loss = loss::mse_loss(result.clone(), target.clone(), loss::Reduction::Mean);
            loss.forward();
            total_loss += loss.data().mean().unwrap();
            // println!("Data! {:#?}", result.data());
            loss.backward(1.0);
            optimizer.step();
        }

        println!("Loss for epoch {} : {} ", epoch, total_loss);
    }

    // model.status.eval();
    // let entries: Vec<_> = data.shuffle().batch(1).into_iter().collect();

    // let (input_array, target_array) = entries[0];
    // let input = neuronika::from_ndarray(input_array.to_owned());
    // let converted_target = convert_target(&target_array);
    // let target = neuronika::from_ndarray(converted_target.to_owned());
    // let result = model.forward(input);

    // println!("row! {:#?} {:#?}", input_array, converted_target);

    // let loss = loss::mae_loss(result.clone(), target.clone(), loss::Reduction::Mean);
    // loss.forward();
    // println!("Loss! {:#?}", loss.data().sum());
    // println!("Data! {:#?}", result.data());
    // println!(
    //     "Data! {:#?}",
    //     result
    //         .data()
    //         .iter()
    //         .enumerate()
    //         .fold(0.0, |prob, (i, val)| prob + i as f32 * val)
    //         / result.data().sum()
    // );
    std::fs::write(
        "learning/pyraminx.json",
        serde_json::to_string_pretty(&model).unwrap(),
    )
    .unwrap();
}
