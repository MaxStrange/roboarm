use nalgebra as na;
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
}

#[derive(Debug)]
pub enum Mode {
    Random,
    Genetic,
}

impl ExperimentConfig {
    /// Attempts to parse the given config file into a new ExperimentConfig instance.
    pub fn new(configpath: &Path) -> Result<Self, String> {
        let mut configuration = config::Config::default();
        configuration.merge(config::File::with_name(configpath.to_str().unwrap())).unwrap();
        let mut setting_strings = configuration.try_into::<HashMap<String, String>>().unwrap();

        let nsteps_per_episode_str = match setting_strings.entry("nsteps_per_episode".to_string()) {
            hash_map::Entry::Occupied(o) => o,
            hash_map::Entry::Vacant(_) => return Err("Missing nsteps_per_episode in config file.".to_string()),
        };

        let nsteps_per_episode = match nsteps_per_episode_str.get().parse::<u64>() {
            Ok(x) => x,
            Err(_) => return Err("Could not convert nsteps_per_episode to integer.".to_string()),
        };

        let nepisodes_str = match setting_strings.entry("nepisodes".to_string()) {
            hash_map::Entry::Occupied(o) => o,
            hash_map::Entry::Vacant(_) => return Err("Missing nepisodes in config file.".to_string()),
        };

        let nepisodes = match nepisodes_str.get().parse::<u64>() {
            Ok(x) => x,
            Err(_) => return Err("Could not convert nepisodes to integer.".to_string()),
        };

        let modestr = match setting_strings.entry("mode".to_string()) {
            hash_map::Entry::Occupied(o) => o,
            hash_map::Entry::Vacant(_) => return Err("Missing mode in config file.".to_string()),
        }.get().clone();

        // Try to convert the mode into one of either Mode::Random or Mode::Genetic
        let mode = match modestr.as_str() {
            "random" => Mode::Random,
            "genetic" => Mode::Genetic,
            m => {
                let mut errmsg = String::new();
                writeln!(errmsg, "Mode must be 'random' or 'genetic' but is {}", m);
                return Err(errmsg);
            },
        };

        let comstr = match setting_strings.entry("com".to_string()) {
            hash_map::Entry::Occupied(o) => o,
            hash_map::Entry::Vacant(_) => return Err("Missing com in config file.".to_string()),
        }.get().clone();

        // Parse out the number of networks in a generation if the mode is genetic
        let generation_size = match mode {
            Mode::Random => 0,
            Mode::Genetic => match parse_genetic_parameter::<u64>(&mut setting_strings, "generation_size".to_string()) {
                Err(msg) => return Err(msg),
                Ok(val) => val,
            },
        };

        // Parse out 'low' if the mode is genetic
        let low = match mode {
            Mode::Random => 0.0,
            Mode::Genetic => match parse_genetic_parameter::<f64>(&mut setting_strings, "low".to_string()) {
                Err(msg) => return Err(msg),
                Ok(val) => val,
            },
        };

        // Parse out 'high' if the mode is genetic
        let high = match mode {
            Mode::Random => 0.0,
            Mode::Genetic => match parse_genetic_parameter::<f64>(&mut setting_strings, "high".to_string()) {
                Err(msg) => return Err(msg),
                Ok(val) => val,
            },
        };

        // Parse out 'nkeep' if the mode is genetic
        let nkeep = match mode {
            Mode::Random => 0,
            Mode::Genetic => match parse_genetic_parameter::<u64>(&mut setting_strings, "nkeep".to_string()) {
                Err(msg) => return Err(msg),
                Ok(val) => val,
            },
        };

        // Parse out 'mutation_stdev' if the mode is genetic
        let mutation_stdev = match mode {
            Mode::Random => 0.0,
            Mode::Genetic => match parse_genetic_parameter::<f64>(&mut setting_strings, "mutation_stdev".to_string()) {
                Err(msg) => return Err(msg),
                Ok(val) => val,
            },
        };

        // Parse out 'percent_mutate' if the mode is genetic
        let percent_mutate = match mode {
            Mode::Random => 0.0,
            Mode::Genetic => match parse_genetic_parameter::<f64>(&mut setting_strings, "percent_mutate".to_string()) {
                Err(msg) => return Err(msg),
                Ok(val) => val,
            },
        };

        // Check to make sure percent_mutate is within allowed bounds
        if percent_mutate < 0.0 || percent_mutate > 100.0 {
            let mut msg = String::new();
            write!(msg, "'percent_mutate' must be in interval [0, 100], but is {}", percent_mutate);
            return Err(msg);
        };

        // Parse out 'target_x' if the mode is genetic
        let target_x = match mode {
            Mode::Random => 0.0,
            Mode::Genetic => parse_genetic_parameter::<f64>(&mut setting_strings, "target_x".to_string())?
        };
        let target_y = match mode {
            Mode::Random => 0.0,
            Mode::Genetic => parse_genetic_parameter::<f64>(&mut setting_strings, "target_y".to_string())?
        };
        let target_z = match mode {
            Mode::Random => 0.0,
            Mode::Genetic => parse_genetic_parameter::<f64>(&mut setting_strings, "target_z".to_string())?
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
        })
    }
}

fn parse_genetic_parameter<T: FromStr>(setting_strings: &mut HashMap<String, String>, s: String) -> Result<T, String> {
    match setting_strings.entry(s.clone()) {
        hash_map::Entry::Vacant(_) => {
            let mut msg = String::new();
            write!(msg, "Missing {} in config file", s);
            Err(msg)
        },
        hash_map::Entry::Occupied(o) => match o.get().clone().parse::<T>() {
            Err(_) => {
                let mut msg = String::new();
                write!(msg, "Could not convert {} into appropriate type", s);
                Err(msg)
            },
            Ok(val) => Ok(val),
        },
    }
}

impl fmt::Display for ExperimentConfig {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "Experiment Configuration:");
        writeln!(f, "Number of Steps per Episode: {}", self.nsteps_per_episode);
        writeln!(f, "Number of Episodes: {}", self.nepisodes);
        writeln!(f, "Mode: {:?}", self.mode)
    }
}
