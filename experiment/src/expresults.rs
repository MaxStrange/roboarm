use std::fmt;
use super::expconfig::ExperimentConfig;

#[derive(Debug)]
pub struct ExperimentResults<'a> {
    config: &'a ExperimentConfig,
}

impl<'a> ExperimentResults<'a> {
    pub fn new(experiment: &'a ExperimentConfig) -> Self {
        ExperimentResults::<'a> {
            config: experiment,
        }
    }
}

impl<'a> fmt::Display for ExperimentResults<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "Experiment Results");
        writeln!(f, "===================");
        writeln!(f, "Experiment Results Generated for Experiment with configuration:");
        writeln!(f, "{}", self.config)
    }
}