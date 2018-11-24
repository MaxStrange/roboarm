use rand;
use super::network::{Layer, MultilayerPerceptron, relu, linear};
use super::expconfig::ExperimentConfig;
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

    /// Adds the next fitness value
    pub fn add_fitness(&mut self, fitness: f64) {
        self.evaluations.push(fitness);
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
    pub fn create_next_generation<'a>(&mut self, experiment: &'a ExperimentConfig, rng: &mut rand::ThreadRng) {
        let gensize = experiment.generation_size as usize;
        let nkeep = experiment.nkeep as usize;

        self.networks = if self.generation == 0 {
            self.spawn_n_networks(gensize, experiment.low, experiment.high, rng)
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
            idx_val_nets.reverse();

            // Now keep only the top-performing nkeep nets from self.networks
            let mut nets_to_keep = Vec::new();
            for i in 0..nkeep {
                let idx = idx_val_nets[i].0;
                nets_to_keep.push(self.networks[idx].clone());
            }

            // Spawn the rest of the nets as random mutations of the others
            let mut rest = Vec::new();
            for i in 0..(gensize - nkeep) {
                let mutant = nets_to_keep[i % nkeep].mutate(rng, experiment.percent_mutate / 100.0, experiment.mutation_stdev);
                rest.push(mutant);
            }

            // Move the rest into nets_to_keep
            for net in rest {
                nets_to_keep.push(net);
            }

            assert!(nets_to_keep.len() == gensize);
            nets_to_keep
        };
        self.generation += 1;
        self.evaluations.clear();
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
