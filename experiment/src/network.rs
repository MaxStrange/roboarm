//! Code pertaining to simple MLP-style neural networks

extern crate nalgebra;
extern crate rand;

use nalgebra as na;

pub fn relu(x: f64) -> f64 {
    if x < 0.0 {
        0.0
    } else {
        x
    }
}

/// MLP Neural Network
///
/// This struct contains a dirt-simple implementation of a
/// strictly feedforward, fully-connected, multilayer neural network.
///
/// This implementation offers no stochastic gradient descent or
/// backpropagation. You build it and then run it.
///
/// To train it, change its weight matrix.
///
pub struct MultilayerPerceptron {
    /// The layers present in this network in order from input to output.
    layers: Vec<Layer>,
}

/// A layer of an MLP
///
/// Layers in the MLP are always feedforward and fully-connected.
/// They may contain any activation function.
///
pub struct Layer {
    /// The number of nodes in this layer
    nnodes: usize,
    /// The activation function. Takes an input, applies a nonlinearity to it, and then returns the result.
    activation_function: Box<Fn(f64) -> f64>,
    /// Matrix of weights going *out of* this Layer. Matrix is N_thislayer x N_nextlayer.
    weights: na::Matrix<f64, na::Dynamic, na::Dynamic, na::MatrixVec<f64, na::Dynamic, na::Dynamic>>,
    /// Are we the output layer?
    output: bool,
}

impl MultilayerPerceptron {
    /// Returns an empty network. Use the builder methods to make it how you like.
    pub fn new() -> Self {
        MultilayerPerceptron {
            layers: Vec::new(),
        }
    }

    pub fn add_layer(&mut self, layer: Layer) -> &mut Self {
        self.layers.push(layer);
        self
    }

    pub fn finalize(&self) -> Self {
        MultilayerPerceptron {
            layers: self.layers.clone(),
        }
    }
}

impl Layer {
    /// Returns an empty Layer. Use the builder methods to make it how you like.
    pub fn new() -> Self {
        Layer {
            nnodes: 0,
            activation_function: Box::new(|_| 0.0),
            weights: na::DMatrix::<f64>::identity(10, 10),
            output: false,
        }
    }

    /// Makes the layer length `nnodes`
    pub fn length(&mut self, nnodes: usize) -> &mut Self {
        self.nnodes = nnodes;
        self
    }

    /// Makes this layer's activation function `f`
    pub fn activation(&mut self, f: Box<Fn(f64) -> f64>) -> &mut Self {
        self.activation_function = f;
        self
    }

    /// Adjusts the weights of this layer to the appropriate dimensions, given the next layer's size.
    pub fn connect(&mut self, next_layer_nnodes: usize) -> &mut Self {
        self.weights = na::DMatrix::<f64>::identity(self.nnodes, next_layer_nnodes);
        self
    }

    /// Alerts this layer that it is the output layer. Call this instead of `connect` for the output layer.
    pub fn make_output(&mut self) -> &mut Self {
        self.output = true;
        self.weights = na::DMatrix::<f64>::identity(self.nnodes, 2);
        self
    }

    /// Initializes the weights via random uniform distribution to values in the interval [low, high].
    pub fn initialize_weights(&mut self, low: f64, high: f64, rng: &mut rand::ThreadRng) -> &mut Self {
        // TODO
        self
    }

    pub fn finalize(&self) -> Self {
        Layer {
            nnodes: self.nnodes,
            activation_function: self.activation_function.clone(),
            weights: self.weights.clone(),
            output: self.output,
        }
    }
}
