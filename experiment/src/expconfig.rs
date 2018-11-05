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