use std::fmt;

#[derive(Debug)]
pub struct ExperimentConfig {
    pub nsteps_per_episode: u64,
    pub nepisodes: u64,
    pub mode: Mode,
    pub comstr: &'static str,
}

#[derive(Debug)]
pub enum Mode {
    Random,
    Genetic,
}

impl ExperimentConfig {
    pub fn new(mode: Mode, nepisodes: u64, nsteps_per_episode: u64, comstr: &'static str) -> Self {
        ExperimentConfig{mode: mode, nepisodes: nepisodes, nsteps_per_episode: nsteps_per_episode, comstr: comstr}
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