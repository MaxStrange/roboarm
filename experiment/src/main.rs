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
//! com: com_port_or_dev_path

/* Externs */
extern crate config;
extern crate nalgebra;
extern crate num;
extern crate rand;

/* Modules */
mod expconfig;
mod expresults;
mod expstate;
mod network;

/* Uses */
use nalgebra as na;
use rand::prelude::*;
use std::env;
use std::path;
use std::process;
use std::fmt::Write;
use std::io::Write as _w; // have to include two different Writes
use std::fs;

/* Selfs */
use self::expconfig::{Mode, ExperimentConfig};
use self::expresults::ExperimentResults;
use self::expstate::ExperimentState;

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
    let experiment = match ExperimentConfig::new(configpath) {
        Ok(exp) => exp,
        Err(msg) => {
            println!("{}", msg);
            process::exit(3);
        },
    };

    // Inform the user how we parsed their config file
    println!("Running Experiment with Configuration:\n{}", experiment);

    // Run the experiment
    let results = run_experiment(&experiment);

    // Print and save the results
    println!("{}", results);
    results.save("results.txt".to_string());
}

fn run_experiment<'a>(experiment: &'a ExperimentConfig) -> ExperimentResults<'a> {
    // Create the random number generator
    let mut rng = thread_rng();

    // Create the results to record to
    let mut results = ExperimentResults::new(&experiment);

    // Create the Experiment state which will track anything that persists between episodes
    let mut state = ExperimentState::new();

    // Run the whole experiment (each episode)
    for episode in 0..experiment.nepisodes {
        println!("=== Starting episode {} ===", episode);
        results.set_episode(episode);
        run_episode(episode, experiment, &mut results, &mut rng, &mut state);
    }

    results.finish();
    results
}

fn run_episode<'a>(episode: u64, experiment: &'a ExperimentConfig, results: &mut ExperimentResults, rng: &mut rand::ThreadRng, state: &mut ExperimentState) {
    // Create the script for this episode
    let mut scriptname = String::new();
    write!(scriptname, "tmpscript_episode_{}.txt", episode);
    let mut f = match fs::File::create(&scriptname) {
        Ok(file) => file,
        Err(e) => {
            let mut msg = String::new();
            writeln!(msg, "Could not run episode {} due to error in opening script file: {:?}", episode, e);

            write!(results, "{}", msg);
            print!("{}", msg);
            return;
        },
    };

    // Take a bunch of steps, with behavior dependent on the experiment configuration
    match experiment.mode {
        Mode::Random => run_random_episode(experiment, rng, results, &mut f),
        Mode::Genetic => run_genetic_episode(experiment, rng, results, &mut f, state),
    };

    // Make sure to go to home after every episode and spend a few cycles there.
    writeln!(f, "home");
    writeln!(f, "home");
    writeln!(f, "home");

    // Execute the script
    let status = if cfg!(target_os = "windows") {
        process::Command::new("target/debug/teleop.exe")
                            .arg(experiment.comstr.as_str())
                            .arg(scriptname.as_str())
                            .status()
    } else {
        process::Command::new("target/debug/teleop")
                            .arg(experiment.comstr.as_str())
                            .arg(scriptname.as_str())
                            .status()
    };

    // Check the results of running the script
    let s = match status {
        Ok(status) => {
            writeln!(results, "Executed episode {}", episode);
            status
        },
        Err(e) => {
            writeln!(results, "Could not execute episode {}. Error: {:?}", episode, e);
            return;
        },
    };

    // Check the status code
    match s.code() {
        Some(0) => writeln!(results, "Script executed successfully.").unwrap(),
        Some(v) => writeln!(results, "Script exited abnormally with exit status {}", v).unwrap(),
        None => (),
    };

    // Remove the temporary script
    match fs::remove_file(&scriptname) {
        Err(e) => writeln!(results, "Could not remove file {}. Error: {:?}", scriptname, e).unwrap(),
        Ok(_) => (),
    };
}

fn run_random_episode<'a>(experiment: &'a ExperimentConfig, rng: &mut rand::ThreadRng, results: &mut ExperimentResults, f: &mut fs::File) {
    // Starting angles
    let mut base: isize = ANGLE_START_BASE;
    let mut shoulder: isize = ANGLE_START_SHOULDER;
    let mut elbow: isize = ANGLE_START_ELBOW;

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
}

fn run_genetic_episode<'a>(experiment: &'a ExperimentConfig, rng: &mut rand::ThreadRng, results: &mut ExperimentResults, f: &mut fs::File, state: &mut ExperimentState) {
    // Crate a generation
    state.create_next_generation(experiment, rng);

    // Go to random start position
    let base_start: f64 = num::clamp(ANGLE_START_BASE as f64 + rng.gen_range(-30.0, 30.0), ANGLE_LOWER_LIMIT_BASE as f64, ANGLE_UPPER_LIMIT_BASE as f64);
    let shoulder_start: f64 = num::clamp(ANGLE_START_SHOULDER as f64 + rng.gen_range(-30.0, 30.0), ANGLE_LOWER_LIMIT_SHOULDER as f64, ANGLE_UPPER_LIMIT_SHOULDER as f64);
    let elbow_start: f64 = num::clamp(ANGLE_START_ELBOW as f64 + rng.gen_range(-30.0, 30.0), ANGLE_LOWER_LIMIT_ELBOW as f64, ANGLE_UPPER_LIMIT_ELBOW as f64);
    writeln!(f, "servo {} {}", BASENUM, base_start);
    writeln!(f, "servo {} {}", SHOULDERNUM, shoulder_start);
    writeln!(f, "servo {} {}", ELBOWNUM, elbow_start);
    writeln!(results, "servo {} {}", BASENUM, base_start);
    writeln!(results, "servo {} {}", SHOULDERNUM, shoulder_start);
    writeln!(results, "servo {} {}", ELBOWNUM, elbow_start);

    // Evaluate each network in the generation
    for network in state.networks.iter() {
        // For each step, get the values for each joint delta from a forward pass through the current network
        let (mut base, mut shoulder, mut elbow) = (base_start, shoulder_start, elbow_start);
        for _step in 0..experiment.nsteps_per_episode {
            let input = na::DVector::<f64>::from_vec(network.input_length(), vec!(base, shoulder, elbow));
            let output = network.forward(&input);
            base = output[0];
            shoulder = output[1];
            elbow = output[2];
        }

        // Now figure out how fit this network is based on how close the arm ended up to the object
        // TODO
    }
}
