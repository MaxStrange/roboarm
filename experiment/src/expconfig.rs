use std::fmt;

#[derive(Debug)]
pub struct ExperimentConfig {
    nsteps_per_episode: i64,
    nepisodes: i64,
    mode: Mode,
}

#[derive(Debug)]
pub enum Mode {
    Random,
    Genetic,
}

impl ExperimentConfig {
    pub fn new(mode: Mode, nepisodes: i64, nsteps_per_episode: i64) -> Self {
        ExperimentConfig{mode: mode, nepisodes: nepisodes, nsteps_per_episode: nsteps_per_episode}
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