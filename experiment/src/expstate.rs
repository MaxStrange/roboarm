use rand;
use super::network::{Layer, MultilayerPerceptron, relu, linear};
use super::expconfig::{ExperimentConfig, Mode};
use std::cmp::Ordering::Equal;

/// A struct to maintain state across the whole experiment
pub struct ExperimentState {
    /// Which generation we are on (starts from 0 as the first)
    generation: usize,
    /// The networks in the current generation
    pub networks: Vec<MultilayerPerceptron>,
    /// How fit each network is. The ith evaluation is the evaluation for the ith network.
    /// This vector will be cleared of values each time we move to a new generation.
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
            self.spawn_from_networks(gensize, nkeep, rng, experiment.percent_mutate, experiment.mutation_stdev)
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

    fn spawn_from_networks(&self, gensize: usize, nkeep: usize, rng: &mut rand::ThreadRng, percent_mutate: f64, mutation_stdev: f64) -> Vec<MultilayerPerceptron> {
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
            let mutant = nets_to_keep[i % nkeep].mutate(rng, percent_mutate / 100.0, mutation_stdev);
            rest.push(mutant);
        }

        // Move the rest into nets_to_keep
        for net in rest {
            nets_to_keep.push(net);
        }

        assert!(nets_to_keep.len() == gensize);
        nets_to_keep
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

#[cfg(test)]
mod tests {
    extern crate nalgebra;

    use super::*;

    use nalgebra as na;
    use rand::prelude::*;

    fn approx_equal(a: f64, b: f64, decimal_places: u8) -> bool {
        let factor = 10.0f64.powi(decimal_places as i32);
        let a = (a * factor).trunc();
        let b = (b * factor).trunc();
        a == b
    }

    fn build_input(input_length: usize) -> na::DVector<f64> {
        let mut v = Vec::<f64>::new();
        for i in 0..input_length {
            v.push(i as f64);
        }
        na::DVector::<f64>::from_vec(input_length, v)
    }

    fn create_experiment_config(gensize: u64, nkeep: u64) -> ExperimentConfig {
        ExperimentConfig {
            nsteps_per_episode: 30,
            nepisodes: 2,
            mode: Mode::Genetic,
            comstr: "simulation".to_string(),
            generation_size: gensize,
            low: -1.0,
            high: 1.0,
            nkeep: nkeep,
            percent_mutate: 2.0,
            mutation_stdev: 0.25,
            target: na::Translation3::new(0.0, 0.0, 0.0),
            urdfpath: "".to_string(),
        }
    }

    #[test]
    fn test_next_generation_is_based_on_best_networks() {
        let gensize = 100;
        let nkeep = 1;
        let mut config = create_experiment_config(gensize, nkeep);
        config.percent_mutate = 0.0;
        let mut state = ExperimentState::new();
        let mut rng = thread_rng();

        // Make a generation of networks
        state.create_next_generation(&config, &mut rng);

        // Assign a fitness to each network equal to its forward pass on the same input
        let input = build_input(state.networks[0].input_length());
        let mut fitnesses = Vec::new();
        for net in state.networks.iter() {
            let fitness = net.forward(&input).iter().sum();
            fitnesses.push(fitness);
        }
        for fitness in fitnesses {
            state.add_fitness(fitness);
        }
        let maxfitness = state.evaluations.iter().cloned().fold(-1.0/0.0, f64::max);

        // Derive a second generation with no mutation and nkeep = 1
        state.create_next_generation(&config, &mut rng);

        // Assert that the output of each network is equal to the max of the fitnesses from the last generation
        let ndecimals = 3;
        for net in state.networks.iter() {
            let output = net.forward(&input).iter().sum();
            assert!(approx_equal(output, maxfitness, ndecimals));
        }
    }
}
