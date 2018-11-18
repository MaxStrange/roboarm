use std::collections::hash_map::{self, HashMap};
use std::fmt::{self, Write};
use std::path::Path;

#[derive(Debug)]
pub struct ExperimentConfig {
    pub nsteps_per_episode: u64,
    pub nepisodes: u64,
    pub mode: Mode,
    pub comstr: String,
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

        // Now print out the settings as we interpreted them
        Ok(ExperimentConfig {
            nsteps_per_episode: nsteps_per_episode,
            nepisodes: nepisodes,
            mode: mode,
            comstr: comstr,
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
