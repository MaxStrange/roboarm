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
extern crate num;
extern crate rand;

/* Modules */
mod expconfig;
mod expresults;

/* Uses */
use rand::prelude::*;
use std::env;
use std::path;
use std::process;
use std::collections::{hash_map, HashMap};
use std::fmt::Write;
use std::io::Write as _w; // have to include two different Writes
use std::fs::File;

/* Selfs */
use self::expconfig::{Mode, ExperimentConfig};
use self::expresults::ExperimentResults;

/* Consts */
const ANGLE_START_BASE: isize = 90;
const ANGLE_START_SHOULDER: isize = 10;
const ANGLE_START_ELBOW: isize = 155;

const ANGLE_LOWER_LIMIT_BASE: isize = 0;
const ANGLE_LOWER_LIMIT_SHOULDER: isize = 0;
const ANGLE_LOWER_LIMIT_ELBOW: isize = 100;

const ANGLE_UPPER_LIMIT_BASE: isize = 180;
const ANGLE_UPPER_LIMIT_SHOULDER: isize = 50;
const ANGLE_UPPER_LIMIT_ELBOW: isize = 180;

const BASENUM: usize = 0;
const SHOULDERNUM: usize = 1;
const ELBOWNUM: usize = 2;

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
    }.get().parse::<u64>().expect("Could not convert nsteps_per_episode to integer.");

    let nepisodes = match setting_strings.entry("nepisodes".to_string()) {
        hash_map::Entry::Occupied(o) => o,
        hash_map::Entry::Vacant(_) => { println!("Missing nepisodes in config file."); process::exit(3) },
    }.get().parse::<u64>().expect("Could not convert nepisodes to integer.");

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

    println!("Running Experiment with Configuration:\n{}", experiment);

    let results = run_experiment(&experiment);
    println!("{}", results);
}

fn run_experiment<'a>(experiment: &'a ExperimentConfig) -> ExperimentResults<'a> {
    // Create the random number generator
    let mut rng = thread_rng();

    // Create the results to record to
    let mut results = ExperimentResults::new(&experiment);

    // Run the whole experiment (each episode)
    for episode in 0..experiment.nepisodes {
        println!("=== Starting episode {} ===", episode);
        results.set_episode(episode);
        run_episode(episode, experiment, &mut results, &mut rng);
    }

    results
}

fn run_episode<'a>(episode: u64, experiment: &'a ExperimentConfig, results: &mut ExperimentResults, rng: &mut rand::ThreadRng) {
    // Starting angles
    let mut base: isize = ANGLE_START_BASE;
    let mut shoulder: isize = ANGLE_START_SHOULDER;
    let mut elbow: isize = ANGLE_START_ELBOW;

    // Create the script for this episode
    let mut scriptname = String::new();
    write!(scriptname, "tmpscript_episode_{}.txt", episode);
    let mut f = match File::create(scriptname) {
        Err(e) => {
            let mut msg = String::new();
            writeln!(msg, "Could not run episode {} due to error in opening script file: {:?}", episode, e);

            write!(results, "{}", msg);
            print!("{}", msg);
            return;
        },
        Ok(file) => file,
    };

    // For each step, do a random step of up to 15 degrees in either direction on each servo
    for _step in 0..experiment.nsteps_per_episode {
        // Generate a bunch of values
        let randbase = rng.gen_range(-15, 16);
        let randshoulder = rng.gen_range(-15, 16);
        let randelbow = rng.gen_range(-15, 16);

        // Add the values to the joints
        base += randbase;
        shoulder += randshoulder;
        elbow += randelbow;

        // Clamp the joints to between min and max for each joint
        base = num::clamp(base, ANGLE_LOWER_LIMIT_BASE, ANGLE_UPPER_LIMIT_BASE);
        shoulder = num::clamp(shoulder, ANGLE_LOWER_LIMIT_SHOULDER, ANGLE_UPPER_LIMIT_SHOULDER);
        elbow = num::clamp(elbow, ANGLE_LOWER_LIMIT_ELBOW, ANGLE_UPPER_LIMIT_ELBOW);

        // Write to the file
        writeln!(f, "servo {} {}", BASENUM, base);
        writeln!(f, "servo {} {}", SHOULDERNUM, shoulder);
        writeln!(f, "servo {} {}", ELBOWNUM, elbow);

        // Also write to results
        writeln!(results, "servo {} {}", BASENUM, base);
        writeln!(results, "servo {} {}", SHOULDERNUM, shoulder);
        writeln!(results, "servo {} {}", ELBOWNUM, elbow);
    }

    // Make sure to go to home after every episode and spend a few cycles there.
    writeln!(f, "home");
    writeln!(f, "home");
    writeln!(f, "home");

    // Execute the script
    let mut cmd = if cfg!(target_os = "windows") {
        process::Command::new("target/debug/teleop.exe")
    } else {
        process::Command::new("target/debug/teleop")
    };
    let s = match cmd.status() {
        Ok(status) => {
            writeln!(results, "Executed episode {} successfully. Exit status: {}", episode, status);
            status
        },
        Err(e) => {
            writeln!(results, "Could not execute episode {}. Error: {:?}", episode, e);
            return;
        },
    };
    match s.code() {
        Some(0) => writeln!(results, "Script executed successfully.").unwrap(),
        Some(v) => writeln!(results, "Script exited abnormally with exit status {}", v).unwrap(),
        None => (),
    };
}
