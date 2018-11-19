use rand;
use super::network::{Layer, MultilayerPerceptron, relu, linear};
use std::cmp::Ordering::Equal;

/// A struct to maintain state across the whole experiment
pub struct ExperimentState {
    /// Which generation we are on (starts from 0 as the first)
    generation: usize,
    /// The networks in the current generation
    pub networks: Vec<MultilayerPerceptron>,
    /// How fit each network is. The ith evaluation is the evaluations for the ith network.
    evaluations: Vec<f64>,
}

impl ExperimentState {
    pub fn new() -> Self {
        ExperimentState {
            generation: 0,
            networks: Vec::new(),
            evaluations: Vec::new(),
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
        self.networks = if self.generation == 0 {
            self.spawn_n_networks(gensize, low, high, rng)
        } else {
            // sort the networks along with their indexes by how well they did
            let mut idx_val_nets: Vec<(usize, (&f64, &MultilayerPerceptron))> =
                (0..self.evaluations.len())
                    .zip(
                        self.evaluations
                            .iter()
                            .zip(&self.networks)
                    )
                    .collect();
            idx_val_nets.sort_unstable_by(|a, b| (a.1).0.partial_cmp((b.1).0).unwrap_or(Equal));

            // Now keep only the top-performing nkeep nets from self.networks
            let mut nets_to_keep = Vec::new();
            for i in 0..nkeep {
                let idx = idx_val_nets[i].0;
                nets_to_keep.push(self.networks[idx].clone());
            }

            // Spawn the rest of the nets as random mutations of the others
            // TODO
            let mut rest = self.spawn_n_networks(gensize - nkeep, low, high, rng);
            for net in rest {
                nets_to_keep.push(net);
            }

            nets_to_keep
        };
        self.generation += 1;
    }

    fn spawn_n_networks(&self, n: usize, low: f64, high: f64, rng: &mut rand::ThreadRng) -> Vec<MultilayerPerceptron> {
        let mut v = Vec::<MultilayerPerceptron>::new();
        for _netidx in 0..n {
            let net = self.build_network(low, high, rng);
            v.push(net);
        }
        v
    }

    fn build_network(&self, low: f64, high: f64, rng: &mut rand::ThreadRng) -> MultilayerPerceptron {
        let net = {
            MultilayerPerceptron::new()
                .add_layer(
                    Layer::new()
                        .length(3)
                        .activation(linear)
                        .connect(125)
                        .initialize_weights(low, high, rng)
                        .finalize()
                )
                .add_layer(
                    Layer::new()
                        .length(125)
                        .activation(relu)
                        .connect(75)
                        .initialize_weights(low, high, rng)
                        .finalize()
                )
                .add_layer(
                    Layer::new()
                        .length(75)
                        .activation(relu)
                        .connect(3)
                        .initialize_weights(low, high, rng)
                        .finalize()
                )
                .add_layer(
                    Layer::new()
                        .length(3)
                        .activation(linear)
                        .make_output()
                        .initialize_weights(low, high, rng)
                        .finalize()
                )
                .finalize()
        };
        match net {
            Err(msg) => { println!("{}", msg); panic!(); },
            Ok(n) => n,
        }
    }
}
