/// This module breaks out the network configuration, to make it easy to find mainly.

use super::network::{MultilayerPerceptron, Layer, linear, tanh};
use rand;

/// Builds the network for the experiment.
///
/// Panics if the network can't be finalized for some reason.
pub fn build_network(low: f64, high: f64, rng: &mut rand::StdRng) -> MultilayerPerceptron {
    let net = {
        MultilayerPerceptron::new()
            .add_layer(
                Layer::new()
                    .length(3)
                    .activation(linear)
                    .connect(25)
                    .initialize_weights(low, high, rng)
                    .finalize()
            )
            .add_layer(
                Layer::new()
                    .length(25)
                    .activation(tanh)
                    .connect(25)
                    .initialize_weights(low, high, rng)
                    .finalize()
            )
            .add_layer(
                Layer::new()
                    .length(25)
                    .activation(tanh)
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
        Err(msg) => { println!("Problem building the network: {}", msg); panic!(); },
        Ok(n) => n,
    }
}
