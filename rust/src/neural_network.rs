use crate::twisty_puzzle::PuzzleState;
use ndarray::{Array2, ArrayBase, Dim, Ix2, OwnedRepr};
use neuronika::{
    nn::{loss, Learnable, Linear, ModelStatus},
    Backward, Data, Forward, Gradient, MatMatMulT, Overwrite, Param, VarDiff,
};
use serde::{Deserialize, Serialize};

pub const OUT_SIZE: usize = 12;
const NETWORK_JSON: &str = include_str!("../learning/pyraminx.json");

#[derive(Serialize, Deserialize)]
pub struct NeuralNetwork {
    lin1: Linear,
    // lin2: Linear,
    // lin3: Linear,
    // dropout: Dropout,
    lin4: Linear,
    #[serde(skip)]
    status: ModelStatus,
}

impl NeuralNetwork {
    pub fn new() -> Self {
        let mut status = ModelStatus::default();

        Self {
            lin1: status.register(Linear::new(28, 32)),
            // lin2: status.register(Linear::new(16, 16)),
            // lin3: status.register(Linear::new(16, 16)),
            // dropout: status.register(Dropout::new(0.0001)),
            lin4: status.register(Linear::new(32, OUT_SIZE)),
            status,
        }
    }

    pub fn parameters(&self) -> Vec<Param> {
        self.status.parameters()
    }

    pub fn train(&self) {
        self.status.train();
    }

    pub fn eval(&self) {
        self.status.eval();
    }

    pub fn forward<I, T, U>(
        &self,
        input: I,
    ) -> VarDiff<impl Data<Dim = Ix2> + Forward, impl Gradient<Dim = Ix2> + Overwrite + Backward>
    where
        I: MatMatMulT<Learnable<Ix2>>,
        I::Output: Into<VarDiff<T, U>>,
        T: Data<Dim = Ix2> + Forward,
        U: Gradient<Dim = Ix2> + Backward + Overwrite,
    {
        let out1 = self.lin1.forward(input).relu();
        // let out2 = self.lin2.forward(out1).relu();
        // let out3 = self.lin3.forward(out2).relu();
        // let out4 = self.dropout.forward(out3).relu();
        self.lin4.forward(out1)
    }

    pub fn plugin(&self, input: &PuzzleState) -> f32 {
        self.eval();
        let mut input_array: ArrayBase<OwnedRepr<f32>, Dim<[usize; 2]>> =
            Array2::zeros((1, input.len()));
        for (i, v) in input.iter().enumerate() {
            input_array[(0, i)] = *v as f32;
        }
        println!("{:?}", input_array);
        let input = neuronika::from_ndarray(input_array.to_owned());

        let zero_target: ArrayBase<OwnedRepr<f32>, Dim<[usize; 2]>> = Array2::zeros((1, OUT_SIZE));
        let target = neuronika::from_ndarray(zero_target);
        let result = self.forward(input);

        let loss = loss::mae_loss(result.clone(), target.clone(), loss::Reduction::Mean);
        loss.forward();
        loss.data().sum();
        // println!("Loss! {:#?}", loss.data().sum());
        // println!("Data! {:#?}", result.data());
        // let s = loss.data()[()];
        let s = result.data().sum();
        // let s = result
        //     .data()
        //     .iter()
        //     .enumerate()
        //     .fold(0.0, |prob, (i, val)| prob + i as f32 * val)
        //     / result.data().sum();
        s
    }
}

pub fn load_model() -> NeuralNetwork {
    let NeuralNetwork {
        lin1,
        lin4,
        mut status,
    } = serde_json::from_str(NETWORK_JSON).unwrap();

    NeuralNetwork {
        lin1: status.register(lin1),
        lin4: status.register(lin4),
        status,
    }
}
