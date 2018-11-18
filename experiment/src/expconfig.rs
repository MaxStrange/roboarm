use std::collections::hash_map::{self, HashMap};
use std::fmt::{self, Write};
use std::path::Path;

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
            Mode::Genetic => match setting_strings.entry("generation_size".to_string()) {
                    hash_map::Entry::Vacant(_) => return Err("Missing generation_size in config file.".to_string()),
                    hash_map::Entry::Occupied(o) => match o.get().clone().parse::<u64>() {
                        Err(_) => return Err("Could not convert generation_size into integer.".to_string()),
                        Ok(val) => val,
                }
            }
        };

        // Parse out 'low' if the mode is genetic
        let low = match mode {
            Mode::Random => 0.0,
            Mode::Genetic => match setting_strings.entry("low".to_string()) {
                hash_map::Entry::Vacant(_) => return Err("Missing 'low' in config file.".to_string()),
                hash_map::Entry::Occupied(o) => match o.get().clone().parse::<f64>() {
                    Err(_) => return Err("Could not convert 'low' into float.".to_string()),
                    Ok(val) => val,
                }
            }
        };

        // Parse out 'high' if the mode is genetic
        let high = match mode {
            Mode::Random => 0.0,
            Mode::Genetic => match setting_strings.entry("high".to_string()) {
                hash_map::Entry::Vacant(_) => return Err("Missing 'high' in config file.".to_string()),
                hash_map::Entry::Occupied(o) => match o.get().clone().parse::<f64>() {
                    Err(_) => return Err("Could not convert 'high' into float.".to_string()),
                    Ok(val) => val,
                }
            }
        };

        // Parse out 'nkeep' if the mode is genetic
        let nkeep = match mode {
            Mode::Random => 0,
            Mode::Genetic => match setting_strings.entry("nkeep".to_string()) {
                hash_map::Entry::Vacant(_) => return Err("Missing 'nkeep' in config file.".to_string()),
                hash_map::Entry::Occupied(o) => match o.get().clone().parse::<u64>() {
                    Err(_) => return Err("Could not convert 'nkeep' into integer.".to_string()),
                    Ok(val) => val,
                }
            }
        };

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
        })
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
