use nalgebra as na;
use rand::prelude::*;
use std::collections::hash_map::{self, HashMap};
use std::fmt::{self, Write};
use std::path::Path;
use std::str::FromStr;

#[derive(Debug)]
/// An experiment configuration
pub struct ExperimentConfig {
    /// Number of steps in a single episode
    pub nsteps_per_episode: u64,
    /// Number of episodes in the experiment. If mode is genetic, this is also the number of generations.
    pub nepisodes: u64,
    /// Mode of the experiment.
    pub mode: Mode,
    /// The COM port to find the Robot on.
    pub comstr: String,
    /// The number of networks in a generation. Only parsed if mode is Genetic.
    pub generation_size: u64,
    /// A random new network will have weights in the interval [low, high]
    pub low: f64,
    /// A random new network will have weights in the interval [low, high]
    pub high: f64,
    /// The number of networks to use to seed a new generation
    pub nkeep: u64,
    /// Rough percentage of the weights in each network to mutate to create a mutant. Should be in interval [0.0, 100.0].
    pub percent_mutate: f64,
    /// Mutant weights are formed by drawing from a Gaussian of mu=weight_i, stdev=mutation_stdev
    pub mutation_stdev: f64,
    /// Target location for the gripper. Orientation doesn't matter to us, hence, Translation rather than Isometry. Values are in meters.
    pub target: na::Translation3<f64>,
    /// Path to the Arm URDF file
    pub urdfpath: String,
    /// Seed for the random number generator - the file may specify "none", in which case a random number is used to seed the RNG.
    pub seed: u64,
    /// Path to the weights file (if doing inference).
    pub weights: String,
}

#[derive(Debug)]
/// The different modes the experiment can be run in.
pub enum Mode {
    /// Joint deltas are generated within limits randomly each step.
    Random,
    /// Use a genetic algorithm to train a neural network to control an arm from a start point to an end point.
    Genetic,
    /// Use an already-trained network to control an arm.
    Inference,
}

impl ExperimentConfig {
    /// Attempts to parse the given config file into a new ExperimentConfig instance.
    pub fn new(configpath: &Path) -> Result<Self, String> {
        let mut configuration = config::Config::default();
        configuration.merge(config::File::with_name(configpath.to_str().unwrap())).unwrap();
        let mut setting_strings = configuration.try_into::<HashMap<String, String>>().unwrap();

        // Parse out the mode
        let modestr = parse_parameter::<String>(&mut setting_strings, "mode".to_string())?;

        // Try to convert the mode into one of either Mode::Random, Mode::Genetic, or Mode::Inference
        let mode = match modestr.as_str() {
            "random" => Mode::Random,
            "genetic" => Mode::Genetic,
            "inference" => Mode::Inference,
            m => {
                let mut errmsg = String::new();
                writeln!(errmsg, "Mode must be 'random' or 'genetic' but is {}", m).unwrap();
                return Err(errmsg);
            },
        };

        // Check if 'seed' is present
        let seedstr = match setting_strings.entry("seed".to_string()) {
            hash_map::Entry::Occupied(o) => o,
            hash_map::Entry::Vacant(_) => return Err("Missing 'seed' in config file.".to_string()),
        }.get().clone();

        // Create a seed (chosen at random itself) if there is no seed in the config file
        // Otherwise just parse it into a u64
        let seed = if seedstr.to_ascii_lowercase() == "none" {
            let mut rng = rand::thread_rng();
            rng.next_u64()
        } else {
            match seedstr.parse::<u64>() {
                Ok(x) => x,
                Err(_) => return Err("Could not convert 'seed' to integer.".to_string()),
            }
        };

        // Parse out nsteps_per_episode if mode is genetic or random
        let nsteps_per_episode = parse_parameter::<u64>(&mut setting_strings, "nsteps_per_episode".to_string())?;

        // Parse out the number of episodes if mode is genetic or random
        let nepisodes = match mode {
            Mode::Inference => 1,
            Mode::Genetic | Mode::Random => parse_parameter::<u64>(&mut setting_strings, "nepisodes".to_string())?,
        };

        // Parse out 'com'
        let comstr = parse_parameter::<String>(&mut setting_strings, "com".to_string())?;

        // Parse out 'arm_urdf'
        let urdfpath = parse_parameter::<String>(&mut setting_strings, "arm_urdf".to_string())?;

        // Parse out the number of networks in a generation if the mode is genetic
        let generation_size = match mode {
            Mode::Random | Mode::Inference => 0,
            Mode::Genetic => parse_parameter::<u64>(&mut setting_strings, "generation_size".to_string())?,
        };

        // Parse out 'randomized_weights_low' if the mode is genetic
        let low = match mode {
            Mode::Random | Mode::Inference => 0.0,
            Mode::Genetic => parse_parameter::<f64>(&mut setting_strings, "randomized_weights_low".to_string())?,
        };

        // Parse out 'randomized_weights_high' if the mode is genetic
        let high = match mode {
            Mode::Random | Mode::Inference => 0.0,
            Mode::Genetic => parse_parameter::<f64>(&mut setting_strings, "randomized_weights_high".to_string())?,
        };

        // Parse out 'nkeep_between_generations' if the mode is genetic
        let nkeep = match mode {
            Mode::Random | Mode::Inference => 0,
            Mode::Genetic => parse_parameter::<u64>(&mut setting_strings, "nkeep_between_generations".to_string())?,
        };

        // Parse out 'mutation_stdev' if the mode is genetic
        let mutation_stdev = match mode {
            Mode::Random | Mode::Inference => 0.0,
            Mode::Genetic => parse_parameter::<f64>(&mut setting_strings, "mutation_stdev".to_string())?,
        };

        // Parse out 'percent_mutate' if the mode is genetic
        let percent_mutate = match mode {
            Mode::Random | Mode::Inference => 0.0,
            Mode::Genetic => parse_parameter::<f64>(&mut setting_strings, "percent_mutate".to_string())?,
        };

        // Check to make sure percent_mutate is within allowed bounds
        if percent_mutate < 0.0 || percent_mutate > 100.0 {
            let mut msg = String::new();
            write!(msg, "'percent_mutate' must be in interval [0, 100], but is {}", percent_mutate).unwrap();
            return Err(msg);
        };

        // Parse out 'weights' if the mode is inference
        let weights = match mode {
            Mode::Genetic | Mode::Random => String::new(),
            Mode::Inference => parse_parameter(&mut setting_strings, "weights".to_string())?,
        };

        // Parse out 'target_*' if the mode is genetic
        let target_x = match mode {
            Mode::Random | Mode::Inference => 0.0,
            Mode::Genetic => parse_parameter::<f64>(&mut setting_strings, "target_x".to_string())?,
        };
        let target_y = match mode {
            Mode::Random | Mode::Inference => 0.0,
            Mode::Genetic => parse_parameter::<f64>(&mut setting_strings, "target_y".to_string())?,
        };
        let target_z = match mode {
            Mode::Random | Mode::Inference => 0.0,
            Mode::Genetic => parse_parameter::<f64>(&mut setting_strings, "target_z".to_string())?,
        };
        let target = na::Translation3::new(target_x, target_y, target_z);

        // Now print out the settings as we interpreted them
        Ok(ExperimentConfig {
            nsteps_per_episode: nsteps_per_episode,
            nepisodes: nepisodes,
            mode: mode,
            comstr: comstr,
            generation_size: generation_size,
            low: low,
            high: high,
            nkeep: nkeep,
            mutation_stdev: mutation_stdev,
            percent_mutate: percent_mutate,
            target: target,
            urdfpath: urdfpath,
            seed: seed,
            weights: weights,
        })
    }
}

/// Attempts to parse the given string `s` into a new instance of type `T`.
fn parse_parameter<T: FromStr>(setting_strings: &mut HashMap<String, String>, s: String) -> Result<T, String> {
    match setting_strings.entry(s.clone()) {
        hash_map::Entry::Vacant(_) => {
            let mut msg = String::new();
            write!(msg, "Missing {} in config file", s).unwrap();
            Err(msg)
        },
        hash_map::Entry::Occupied(o) => match o.get().clone().parse::<T>() {
            Err(_) => {
                let mut msg = String::new();
                write!(msg, "Could not convert {} into appropriate type", s).unwrap();
                Err(msg)
            },
            Ok(val) => Ok(val),
        },
    }
}

impl fmt::Display for ExperimentConfig {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "Experiment Configuration:")?;
        writeln!(f, "Number of Steps per Episode: {}", self.nsteps_per_episode)?;
        writeln!(f, "Number of Episodes: {}", self.nepisodes)?;
        writeln!(f, "Mode: {:?}", self.mode)?;
        writeln!(f, "COM: {}", self.comstr)?;
        match self.mode {
            Mode::Genetic => {
                writeln!(f, "Generation size: {}", self.generation_size)?;
                writeln!(f, "Random weight low threshold: {}", self.low)?;
                writeln!(f, "Random weight high threshold: {}", self.high)?;
                writeln!(f, "Number of networks to keep between generations: {}", self.nkeep)?;
                writeln!(f, "Rough percentage of weights in a network to mutate: {}", self.percent_mutate)?;
                writeln!(f, "Mutation Standard Deviation: {}", self.mutation_stdev)?;
            },
            _ => (),
        }
        writeln!(f, "Target for gripper: {:?}", self.target)?;
        writeln!(f, "Path to URDF file: {}", self.urdfpath)?;
        writeln!(f, "Seed: {}", self.seed)?;
        writeln!(f, "Weights: {}", self.weights)
    }
}
