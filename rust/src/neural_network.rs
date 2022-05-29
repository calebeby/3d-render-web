pub(crate) use std::error::Error;

use corgi::layer::Layer;
use corgi::numbers::Float;
use corgi::{array::Array, layer::dense::Dense, model::Model, optimizer::gd::GradientDescent};

pub fn use_model<F1, F2, F3, T>(
    initial_layers_hook: F1,
    model_hook: F2,
    final_layers_hook: F3,
) -> Result<T, Box<dyn Error>>
where
    F1: FnOnce(&mut [Dense]) -> Result<(), Box<dyn Error>>,
    F2: FnOnce(Model) -> T,
    F3: FnOnce(&mut [Dense]) -> Result<(), Box<dyn Error>>,
{
    let learning_rate = 5.0;
    let input_size = 28;
    let hidden_size = 50;
    let output_size = 16;
    let initializer = corgi::initializer::he();
    let relu = corgi::activation::relu();
    let mse = corgi::cost::mse();
    let gd = GradientDescent::new(learning_rate);
    let l1 = Dense::new(input_size, hidden_size, &initializer, Some(&relu));
    let l2 = Dense::new(hidden_size, hidden_size, &initializer, Some(&relu));
    let l3 = Dense::new(hidden_size, hidden_size, &initializer, Some(&relu));
    let l4 = Dense::new(hidden_size, hidden_size, &initializer, Some(&relu));
    let l5 = Dense::new(hidden_size, output_size, &initializer, Some(&relu));
    let mut layers: Vec<_> = vec![l1, l2, l3, l4, l5];
    initial_layers_hook(&mut layers)?;
    let model = Model::new(
        layers.iter_mut().map(|l| l as &mut dyn Layer).collect(),
        &gd,
        &mse,
    );
    let result = model_hook(model);
    final_layers_hook(&mut layers)?;

    Ok(result)
}

pub fn save_parameters<T: Layer>(layers: &mut [T]) -> Result<(), Box<dyn Error>> {
    let serialized: Vec<Vec<&[f64]>> = layers
        .into_iter()
        .map(|layer| {
            let parameters = layer.parameters();
            parameters
                .into_iter()
                .map(|parameter| parameter.values())
                .collect()
        })
        .collect();
    std::fs::write(
        "learning/pyraminx.json",
        serde_json::to_string_pretty(&serialized).unwrap(),
    )?;
    Ok(())
}

pub fn load_parameters<T: Layer>(layers: &mut [T], saved_text: &str) -> Result<(), Box<dyn Error>> {
    let serialized: Vec<Vec<Vec<f64>>> = serde_json::from_str(saved_text)?;
    for (layer, serialized_layer) in layers.iter_mut().zip(serialized.iter()) {
        let mut parameters = layer.parameters();
        for (parameter, serialized_parameter) in parameters.iter_mut().zip(serialized_layer.iter())
        {
            **parameter = Array::from((
                parameter.dimensions().to_vec(),
                serialized_parameter.clone(),
            ))
            .tracked();
        }
    }
    Ok(())
}

pub fn normalize_output(row: Vec<Float>) -> Float {
    // row.iter()
    //     .enumerate()
    //     .max_by(|(_, prob_a), (_, prob_b)| prob_a.partial_cmp(prob_b).unwrap())
    //     .unwrap()
    //     .0 as Float
    let mut output = 0.0;
    let mut sum = 0.0;
    for (i, val) in row.iter().enumerate() {
        sum += val;
        output += val * i as Float;
    }
    output / sum
}

pub fn evaluate_state(state: Vec<Float>, model: &mut Model) -> f64 {
    let input = Array::from((vec![1, state.len()], state));
    let row = model.forward(input).values().to_vec();
    normalize_output(row)
}
