use ndarray::Ix2;
use neuronika::{
    data::DataLoader,
    nn::{loss, Dropout, Learnable, Linear, ModelStatus},
    optim, Backward, Data, Forward, Gradient, MatMatMulT, Overwrite, Param, VarDiff,
};

struct NeuralNetwork {
    lin1: Linear,
    lin2: Linear,
    lin3: Linear,
    lin4: Linear,
    dropout: Dropout,
    status: ModelStatus,
}

impl NeuralNetwork {
    fn new() -> Self {
        let mut status = ModelStatus::default();

        Self {
            lin1: status.register(Linear::new(24, 16)),
            lin2: status.register(Linear::new(16, 16)),
            lin3: status.register(Linear::new(16, 16)),
            lin4: status.register(Linear::new(16, 1)),
            dropout: status.register(Dropout::new(0.0001)),
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
        let out2 = self.lin2.forward(out1).relu();
        let out3 = self.lin3.forward(out2).relu();
        let out4 = self.dropout.forward(out3).relu();
        self.lin4.forward(out4)
    }
}

fn main() {
    let mut data = DataLoader::default()
        .with_labels(&[24])
        .with_delimiter(',')
        .from_csv("2x2_training_data.csv", 24, 1);

    let model = NeuralNetwork::new();

    // let optimizer = optim::SGD::new(model.parameters(), 0.00001, optim::L2::new(0.00));
    let optimizer = optim::Adam::new(
        model.parameters(),
        0.00003,
        (0.9, 0.999),
        optim::L2::new(0.000005),
        1e-8,
    );

    model.status.train();
    for epoch in 0..200 {
        let batched_data = data.shuffle().batch(16).drop_last();
        let mut total_loss: f32 = 0.0;

        for (input_array, target_array) in batched_data {
            let input = neuronika::from_ndarray(input_array.to_owned());
            let target = neuronika::from_ndarray(target_array.to_owned());

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

    model.status.eval();
    let entries: Vec<_> = data.shuffle().batch(1).into_iter().collect();

    let (input_array, target_array) = entries[0];
    let input = neuronika::from_ndarray(input_array.to_owned());
    let target = neuronika::from_ndarray(target_array.to_owned());
    let result = model.forward(input);

    println!("row! {:#?} {:#?}", input_array, target_array);

    let loss = loss::mse_loss(result.clone(), target.clone(), loss::Reduction::Mean);
    loss.forward();
    println!("Loss! {:#?}", loss.data());
    println!("Data! {:#?}", result.data());
}
