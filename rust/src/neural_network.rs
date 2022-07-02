use std::cell::RefCell;
pub(crate) use std::error::Error;
use std::rc::Rc;

use corgi::layer::Layer;
use corgi::numbers::Float;
use corgi::{array::Array, layer::dense::Dense, model::Model};

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
    let i = RefCell::new(0);
    let max_learning_rate = 3.0;
    let min_learning_rate = 0.01;
    let mut get_learning_rate = || {
        {
            let mut s = i.borrow_mut();
            *s += 1;
            let lr = (-(*s as Float / 2000.0).cos() + 1.0) / 2.0
                * (max_learning_rate - min_learning_rate)
                + min_learning_rate;
            lr
        }
        .sin()
    };
    let input_size = 28;
    let hidden_size = 50;
    let output_size = 16;
    let initializer = corgi::initializer::he();
    let relu = corgi::activation::relu();
    let mse = corgi::cost::mse();
    let optimizer = GradientDescent::new(&get_learning_rate);
    let l1 = Dense::new(input_size, hidden_size, &initializer, Some(&relu));
    let l2 = Dense::new(hidden_size, hidden_size, &initializer, Some(&relu));
    let l3 = Dense::new(hidden_size, hidden_size, &initializer, Some(&relu));
    let l4 = Dense::new(hidden_size, hidden_size, &initializer, Some(&relu));
    let l5 = Dense::new(hidden_size, output_size, &initializer, Some(&relu));
    let mut layers: Vec<_> = vec![l1, l2, l3, l4, l5];
    initial_layers_hook(&mut layers)?;
    let model = Model::new(
        layers.iter_mut().map(|l| l as &mut dyn Layer).collect(),
        &optimizer,
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

use corgi::array::*;
use corgi::numbers::*;
use corgi::optimizer::Optimizer;

// #[cfg(feature = "openblas")]
// use corgi::blas::daxpy_blas;

/// A gradient descent optimizer, which stores the parameters it updates, and the learning rate.
pub struct GradientDescent<'a> {
    get_learning_rate: &'a dyn Fn() -> Float,
}

impl<'a> GradientDescent<'a> {
    /// Creates a new gradient descent optimizer, which updates based on the learning rate.
    pub fn new(get_learning_rate: &'a dyn Fn() -> Float) -> GradientDescent {
        GradientDescent { get_learning_rate }
    }
}

impl<'a> Optimizer for GradientDescent<'a> {
    fn update(&self, parameters: Vec<&mut Array>) {
        let learning_rate = (self.get_learning_rate)();
        let mut frozen = Vec::new();
        let mut parameter_values = Vec::new();
        let mut parameter_gradients = Vec::new();
        parameters
            .iter()
            .filter(|p| {
                let gradient = p.gradient();
                frozen.push(gradient.is_none());
                gradient.is_some()
            })
            .for_each(|p| {
                parameter_values.extend(p.values());
                parameter_gradients.extend(p.replace_gradient().unwrap().values());
            });

        #[cfg(not(feature = "openblas"))]
        parameter_values
            .iter_mut()
            .zip(parameter_gradients)
            .for_each(|(x, g): (&mut Float, Float)| {
                *x -= learning_rate * g;
            });
        #[cfg(feature = "openblas")]
        daxpy_blas(-learning_rate, &parameter_gradients, &mut parameter_values);

        parameters
            .into_iter()
            .zip(frozen)
            .filter(|(_, f)| !f)
            .for_each(|(p, _)| {
                *p = Array::from((
                    p.dimensions().to_vec(),
                    parameter_values
                        .drain(0..p.values().len())
                        .collect::<Vec<Float>>(),
                ))
                .tracked();
            });
    }
}

#[cfg(feature = "openblas")]
use cblas_sys::{cblas_daxpy, cblas_dgemm};

#[cfg(feature = "openblas")]
use cblas_sys::CBLAS_LAYOUT;
#[cfg(feature = "openblas")]
use cblas_sys::CBLAS_TRANSPOSE;

#[cfg(feature = "openblas")]
use corgi::numbers::*;

#[cfg(feature = "openblas")]
use std::convert::TryInto;

#[cfg(feature = "openblas")]
use self::CBLAS_LAYOUT::*;
#[cfg(feature = "openblas")]
use self::CBLAS_TRANSPOSE::*;
#[cfg(feature = "openblas")]
pub(crate) fn daxpy_blas(alpha: Float, x: &[Float], y: &mut [Float]) {
    unsafe {
        cblas_daxpy(
            y.len().try_into().unwrap(),
            alpha,
            x.as_ptr(),
            1,
            y.as_mut_ptr(),
            1,
        );
    }
}
