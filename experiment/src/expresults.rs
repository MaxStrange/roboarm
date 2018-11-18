use super::expconfig::ExperimentConfig;
use std::collections::HashMap;
use std::fmt;
use std::io::Write;
use std::fs;

#[derive(Debug)]
pub struct ExperimentResults<'a> {
    config: &'a ExperimentConfig,   // The configuration for the experiment
    cur_episode: u64,               // The current episode
    episode_buffer: String,         // The log for the current episode
    logs: HashMap<u64, String>,     // The log for each episode
}

impl<'a> ExperimentResults<'a> {
    pub fn new(experiment: &'a ExperimentConfig) -> Self {
        ExperimentResults::<'a> {
            config: experiment,
            cur_episode: 0,
            episode_buffer: String::new(),
            logs: HashMap::<u64, String>::new(),
        }
    }

    /// Set the episode that a write! will record info to.
    pub fn set_episode(&mut self, ep: u64) {
        // Potentially switch episode log buffers
        if ep != self.cur_episode {
            self.logs.insert(self.cur_episode, self.episode_buffer.clone());
            self.episode_buffer.clear();
        }
        self.cur_episode = ep;
        self.episode_buffer = match self.logs.get(&ep) {
            Some(buf) => buf.clone(),
            None => String::new(),
        }
    }

    pub fn finish(&mut self) {
        self.logs.insert(self.cur_episode, self.episode_buffer.clone());
    }

    /// Save to disk
    pub fn save(&self, fname: String) {
        let mut f = match fs::File::create(&fname) {
            Err(e) => { println!("Could not save results due to error: {:?}", e); return; },
            Ok(file) => file,
        };

        write!(f, "{}", &self);
    }
}

impl<'a> fmt::Display for ExperimentResults<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "Experiment Results")?;
        writeln!(f, "===================")?;
        writeln!(f, "Experiment Results Generated for Experiment with configuration:")?;
        writeln!(f, "{}", self.config)?;

        // Collect all the keys into a vector in sorted order
        let mut sortedkeys: Vec<&u64> = self.logs.keys().clone().collect();
        sortedkeys.sort_unstable();

        for episode in sortedkeys {
            writeln!(f, "")?;
            writeln!(f, "Episode {}", episode)?;
            writeln!(f, "{}", self.logs.get(episode).unwrap())?;
        }

        Ok(())
    }
}

impl<'a> fmt::Write for ExperimentResults<'a> {
    fn write_str(&mut self, s: &str) -> Result<(), fmt::Error> {
        write!(self.episode_buffer, "{}", s)
    }
}
