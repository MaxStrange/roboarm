use rand;
use super::network::{Layer, MultilayerPerceptron, relu};

/// A struct to maintain state across the whole experiment
pub struct ExperimentState {
    /// Which generation we are on (starts from 0 as the first)
    generation: usize,
    /// The networks in the current generation
    pub networks: Vec<MultilayerPerceptron>,
}

impl ExperimentState {
    pub fn new() -> Self {
        ExperimentState {
            generation: 0,
            networks: Vec::new(),
        }
    }

    /// Creates the next generation of neural networks
    ///
    /// If the current generation does not contain any networks,
    /// a brand new generation is created with random weights between
    /// the low and high parameters.
    ///
    /// If the current generation does contain networks,
    /// the top `nkeep` networks are kept and mutated, while the others are
    /// discarded.
    ///
    /// **Parameters**
    ///
    /// * `gensize` The number of networks in a single generation
    /// * `low` The first generation's networks will have weights in the interval [low, high]
    /// * `high` The first generation's networks will have weights in the interval [low, high]
    /// * `nkeep` The number of networks to keep between generations
    ///
    pub fn create_next_generation(&mut self, gensize: usize, low: f64, high: f64, nkeep: usize, rng: &mut rand::ThreadRng) {
        self.networks = Vec::new();
        for _netidx in 0..gensize {
            let net = {
                MultilayerPerceptron::new()
                    .add_layer(
                        Layer::new()
                            .length(3)
                            .activation(relu)
                            .connect(125)
                            .initialize_weights(low, high, rng)
                            .finalize()
                    )
                    .finalize()
            };
            self.networks.push(net);
        }
    }
}
