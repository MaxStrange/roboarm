//! Code pertaining to simple MLP-style neural networks

extern crate nalgebra;
extern crate rand;

use nalgebra as na;
use rand::Rng;
use rand::distributions::Normal;
use std::io::Read;
use std::io::Write as w_;
use std::fmt::Write;

pub fn linear(x: f64) -> f64 {
    x
}

pub fn relu(x: f64) -> f64 {
    if x < 0.0 {
        0.0
    } else {
        x
    }
}

#[derive(Clone)]
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

pub type ActivationFunction = fn(f64) -> f64;

#[derive(Clone)]
/// A layer of an MLP
///
/// Layers in the MLP are always feedforward and fully-connected.
/// They may contain any activation function.
///
pub struct Layer {
    /// The number of nodes in this layer
    nnodes: usize,
    /// The activation function. Takes an input, applies a nonlinearity to it, and then returns the result.
    activation_function: ActivationFunction,
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

    /// Attempts to save this network's weights to `path`.
    pub fn save_weights(&self, path: &String) -> std::io::Result<()> {
        let mut f = std::fs::File::create(path)?;
        for layer in self.layers.iter() {
            writeln!(f, "{}", layer.serialize_weights());
        }
        Ok(())
    }

    /// Loads the weights into a network from the given file path, which should contain weights as saved by save_weights().
    pub fn load_weights(&mut self, path: &String) -> std::io::Result<()> {
        let mut f = std::fs::File::open(path)?;
        let mut contents = String::new();
        f.read_to_string(&mut contents)?;
        for (i, line) in contents.split('\n').enumerate() {
            match self.layers[i].deserialize_weights(&line.trim().to_string()) {
                Ok(_) => (),
                Err(e) => return Err(std::io::Error::new(std::io::ErrorKind::Other, e)),
            }
        }
        Ok(())
    }

    pub fn add_layer(&mut self, layer: Layer) -> &mut Self {
        self.layers.push(layer);
        self
    }

    /// Attempts to finalize the building pattern. May fail if configuration doesn't make sense.
    pub fn finalize(&self) -> Result<Self, String> {
        for i in 0..self.layers.len() {
            if i < self.layers.len() - 1 {
                // Check layer i's output weights against layer i+1's input weights
                if self.layers[i].weights.ncols() != self.layers[i + 1].nnodes {
                    let mut msg = String::new();
                    write!(msg, "Layer {} specifies {} connections, but there are {} nodes in layer {}.", i, self.layers[i].weights.ncols(), self.layers[i+1].nnodes, i + 1);
                    return Err(msg);
                }
            }
        }
        Ok(MultilayerPerceptron {
            layers: self.layers.clone(),
        })
    }

    /// Returns the total number of weights in this network.
    pub fn nweights(&self) -> usize {
        let mut total: usize = 0;
        for layer in self.layers.iter() {
            total += layer.weights.ncols() * layer.weights.nrows();
        }
        total
    }

    /// Clone the current network and mutate the offspring's weights.
    ///
    /// Weights are mutated by taking `percent_mutate` of the number of weights in the network
    /// and adjusting them to equal a value drawn from a Gaussian distribution of
    /// mu=current_weight, sigma=`stdev`.
    ///
    /// Note that `percent_mutate` is asserted to be less than or equal to 1.0.
    pub fn mutate(&self, rng: &mut rand::ThreadRng, percent_mutate: f64, stdev: f64) -> Self {
        // Assert that we don't try to mutate more weights than we have
        assert!(percent_mutate <= 1.0);

        // Create the mutant as a clone of us
        let mut mutant = self.clone();

        // Figure out how many weights we should mutate
        let nmutate = (self.nweights() as f64 * percent_mutate).round() as usize;

        // Mutate that many weights
        for _ in 0..nmutate {
            // Pick a weight at random. In the interest of speed and simplicity, let's not worry about
            // whether we have picked it already or not
            let layeridx = rng.gen_range(0, self.layers.len());
            let randrow = rng.gen_range(0, self.layers[layeridx].weights.nrows());
            let randcol = rng.gen_range(0, self.layers[layeridx].weights.ncols());
            let nrows = self.layers[layeridx].weights.nrows();
            let weight_idx = randcol * nrows + randrow;
            let original_weight = self.layers[layeridx].weights[weight_idx];

            let mutant_weight = rng.sample(Normal::new(original_weight, stdev));
            mutant.layers[layeridx].weights[weight_idx] = mutant_weight;
        }

        mutant
    }

    /// Does a forward pass through the MLP.
    ///
    /// Takes a vector, which must be of the same length as the input layer
    /// and returns a vector, which is of the same length as the output layer.
    pub fn forward(&self, input: &na::DVector<f64>) -> na::DVector<f64> {
        assert!(input.len() == self.layers[0].nnodes);

        let mut output = input.clone();
        for layer in self.layers.iter() {
            output.apply(layer.activation_function);
            output = (output.transpose() * &layer.weights).transpose();
        }

        // The final output is diagonal due to final layer's weights being ID
        let mut result = na::DVector::<f64>::from_element(self.layers.last().unwrap().nnodes, 0.0);
        for c in 0..output.ncols() {
            for r in 0..output.nrows() {
                if c == r {
                    result[c] = output[c * output.nrows() + r];
                }
            }
        }

        result
    }

    /// Returns the length of the input vectors this network expects
    pub fn input_length(&self) -> usize {
        if self.layers.len() > 0 {
            self.layers[0].nnodes
        } else {
            0
        }
    }
}

impl Layer {
    /// Returns an empty Layer. Use the builder methods to make it how you like.
    pub fn new() -> Self {
        Layer {
            nnodes: 0,
            activation_function: |_| 0.0,
            weights: na::DMatrix::<f64>::identity(10, 10),
            output: false,
        }
    }

    /// Serializes the Layer's weights into a string representation: a single line of numbers.
    pub fn serialize_weights(&self) -> String {
        let mut s = String::new();
        for w in self.weights.iter() {
            s.push_str(&w.to_string());
            s.push(' ');
        }
        s
    }

    /// Deserializes the given string of weights and fills this Layer's weights with the results.
    pub fn deserialize_weights(&mut self, line: &String) -> Result<(), String> {
        for (i, number) in line.trim().split(' ').enumerate() {
            let n = match number.parse::<f64>() {
                Ok(res) => res,
                Err(_) => {
                    let mut msg = String::new();
                    write!(msg, "Could not parse {} into an f64 while trying to deserialize some weights for a Layer.", number);
                    return Err(msg);
                },
            };
            self.weights[i] = n;
        }
        Ok(())
    }

    /// Makes the layer length `nnodes`
    pub fn length(&mut self, nnodes: usize) -> &mut Self {
        self.nnodes = nnodes;
        self
    }

    /// Makes this layer's activation function `f`
    pub fn activation(&mut self, f: ActivationFunction) -> &mut Self {
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
        self.weights = na::DMatrix::<f64>::identity(self.nnodes, self.nnodes);
        self
    }

    /// Initializes the weights via random uniform distribution to values in the interval [low, high].
    pub fn initialize_weights(&mut self, low: f64, high: f64, rng: &mut rand::ThreadRng) -> &mut Self {
        for c in 0..self.weights.ncols() {
            for r in 0..self.weights.nrows() {
                let val = rng.gen_range(low, high + 1E-9);
                let nrows = self.weights.nrows();
                self.weights[c * nrows + r] = val;
            }
        }
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
