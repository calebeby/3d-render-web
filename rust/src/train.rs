use ndarray::{Array2, Ix2};
use neuronika::{
    nn::{loss, Dropout, Learnable, Linear, ModelStatus},
    optim, Backward, Data, Forward, Gradient, MatMatMulT, Overwrite, Param, VarDiff,
};
use rand::Rng;

struct NeuralNetwork {
    lin1: Linear,
    lin2: Linear,
    lin3: Linear,
    dropout1: Dropout,
    status: ModelStatus,
}

impl NeuralNetwork {
    fn new() -> Self {
        let mut status = ModelStatus::default();

        Self {
            lin1: status.register(Linear::new(1, 5)),
            lin2: status.register(Linear::new(5, 5)),
            dropout1: status.register(Dropout::new(0.0)),
            lin3: status.register(Linear::new(5, 1)),
            status,
        }
    }

    fn parameters(&self) -> Vec<Param> {
        self.status.parameters()
    }

    fn forward<I, T, U>(
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
        let out2 = self.dropout1.forward(out1).relu();
        let out3 = self.lin2.forward(out2).relu();
        self.lin3.forward(out3)
    }
}

fn foo() {
    let model = NeuralNetwork::new();
    let optimizer = optim::SGD::new(model.parameters(), 0.000000001, optim::L2::new(0.0));

    model.status.train();

    // let data = [[10.0, 10.0, 3.0], [3.0, 5.0, 7.0], [2.0, 4.0, 8.0]];
    let mut rng = rand::thread_rng();
    let mut data = vec![];
    for _ in 0..200 {
        let i: f32 = rng.gen_range(1.0..10.0);
        data.push((i, i * i));
    }

    let mut get_row = || {
        let i = rng.gen_range(0..data.len());
        let row = &data[i % data.len()];
        let inputs = vec![row.0];
        let output = vec![row.1];
        (inputs, output)
    };

    // println!("{:#?}", model.parameters());

    // Trains the model.
    let n = 200;
    let mut tot_loss = f32::INFINITY;
    while tot_loss / n as f32 > 0.01 {
        tot_loss = 0.0;
        for _ in 0..n {
            let (inputs1, output1) = get_row();
            let (inputs2, output2) = get_row();
            // println!("row: {:#?}", row);
            let mut both_inputs = vec![];
            both_inputs.extend_from_slice(&inputs1);
            both_inputs.extend_from_slice(&inputs2);
            let mut both_outputs = vec![];
            both_outputs.extend_from_slice(&output1);
            both_outputs.extend_from_slice(&output2);
            let input_array = Array2::from_shape_vec((2, 1), both_inputs).unwrap();
            let output_array = Array2::from_shape_vec((2, 1), both_outputs).unwrap();

            let input = neuronika::from_ndarray(input_array);
            let output = neuronika::from_ndarray(output_array);

            let result = model.forward(input);

            let loss = loss::mse_loss(result.clone(), output.clone(), loss::Reduction::Mean);
            loss.forward();
            loss.backward(1.0);
            tot_loss += loss.data()[()];
            optimizer.step();
        }
        println!("Loss! {}", tot_loss / n as f32);
    }

    model.status.eval();

    let run_test = || {
        let test_input = Array2::from_shape_vec((2, 1), vec![3.5, 2.5]).unwrap();
        let input = neuronika::from_ndarray(test_input);
        let output = model.forward(input);
        let empty_output = Array2::from_shape_vec((2, 1), vec![0.0, 0.0]).unwrap();
        let loss = loss::mse_loss(
            output.clone(),
            neuronika::from_ndarray(empty_output.clone()),
            loss::Reduction::Mean,
        );

        loss.forward();

        println!("{:#?}", output.data());
    };
    run_test();
    run_test();
    run_test();
    run_test();
    run_test();
    run_test();
}

fn main() {
    foo();
}
