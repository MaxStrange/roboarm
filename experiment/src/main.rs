//! Run the experiment for CS 600: Independent Study: Developmental Robotics
//! 
//! This application runs in either random mode or GA mode.
//! If random mode (as specified in the config file), the robot arm
//! just moves to random locations within its allowed joint angles for the
//! length of one episode (some number of movements as determined by
//! the config file), then returns to its home location. It repeats this
//! some number of times (as specified in the config file).
//! 
//! The other mode (Genetic Algorithm mode), the robot arm will get its
//! joint angles from a genetic algorithm.
//! 
//! The config file must have the following items:
//! mode: 'random' or 'genetic'
//! nsteps_per_episode: integer value
//! nepisodes: interger value

/* Externs */
extern crate config;

/* Modules */
mod expconfig;

/* Use statements */
use std::env;
use std::path;
use std::process;
use std::collections::{hash_map, HashMap};
use self::expconfig::{Mode, ExperimentConfig};

fn main() {
    let usage = "Need a path to a valid configuration file.";

    // Get the config file from the user or give them the usage
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        println!("{}", usage);
        process::exit(1);
    }

    // Make sure the config file really exists
    let configpath = path::Path::new(&args[1]);
    if !configpath.exists() {
        println!("{}", usage);
        println!("{:?} does not exist.", configpath);
        process::exit(2);
    }

    // Try to parse the YAML file
    let mut configuration = config::Config::default();
    configuration.merge(config::File::with_name(configpath.to_str().unwrap())).unwrap();
    let mut setting_strings = configuration.try_into::<HashMap<String, String>>().unwrap();

    let nsteps_per_episode = match setting_strings.entry("nsteps_per_episode".to_string()) {
        hash_map::Entry::Occupied(o) => o,
        hash_map::Entry::Vacant(_) => { println!("Missing nsteps_per_episode in config file."); process::exit(3) },
    }.get().parse::<i64>().expect("Could not convert nsteps_per_episode to integer.");

    let nepisodes = match setting_strings.entry("nepisodes".to_string()) {
        hash_map::Entry::Occupied(o) => o,
        hash_map::Entry::Vacant(_) => { println!("Missing nepisodes in config file."); process::exit(3) },
    }.get().parse::<i64>().expect("Could not convert nepisodes to integer.");

    let modestr = match setting_strings.entry("mode".to_string()) {
        hash_map::Entry::Occupied(o) => o,
        hash_map::Entry::Vacant(_) => { println!("Missing mode in config file."); process::exit(3) },
    };

    // Try to convert the mode into one of either Mode::Random or Mode::Genetic
    let mode = match modestr.get().as_str() {
        "random" => Mode::Random,
        "genetic" => Mode::Genetic,
        m => { println!("Mode must be 'random' or 'genetic' but is {}", m); process::exit(3) },
    };

    // Now print out the settings as we interpreted them
    let experiment = ExperimentConfig::new(mode, nepisodes, nsteps_per_episode);

    println!("Experiment Configuration: {:?}", experiment);

    // If config.random, do the random experiment
    // If config.genetic, do the GA experiment
    // Report results
}
