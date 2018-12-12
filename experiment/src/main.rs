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
#![allow(unused_must_use)]

/* Externs */
extern crate config;
extern crate k;
extern crate nalgebra;
extern crate num;
extern crate rand;

/* Modules */
mod expconfig;
mod expresults;
mod expstate;
mod netconfig;
mod network;

/* Uses */
use k::prelude::*;
use k::urdf::FromUrdf;
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
const ANGLE_START_BASE: f64 = 90.0;
const ANGLE_START_SHOULDER: f64 = 10.0;
const ANGLE_START_ELBOW: f64 = 155.0;

const ANGLE_LOWER_LIMIT_BASE: f64 = 0.0;
const ANGLE_LOWER_LIMIT_SHOULDER: f64 = 0.0;
const ANGLE_LOWER_LIMIT_ELBOW: f64 = 100.0;

const ANGLE_UPPER_LIMIT_BASE: f64 = 180.0;
const ANGLE_UPPER_LIMIT_SHOULDER: f64 = 50.0;
const ANGLE_UPPER_LIMIT_ELBOW: f64 = 180.0;

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

    // Parse the URDF file
    let robot = match k::LinkTree::<f64>::from_urdf_file(&experiment.urdfpath) {
        Ok(robot) => robot,
        Err(e) => {
            println!("Problem with loading the URDF file: {:?}", e);
            process::exit(4);
        },
    };
    let mut arm = match k::Manipulator::from_link_tree("hand", &robot) {
        Some(arm) => arm,
        None => {
            println!("Problem pulling out the 'hand' link from the URDF robot. Could not find 'hand' in tree.");
            process::exit(4);
        },
    };

    // Inform the user how we parsed their config file
    println!("Running Experiment with Configuration:\n{}", experiment);

    // Run the experiment
    let results = run_experiment(&experiment, &mut arm);

    // Save the results
    results.save("results.txt".to_string());
}

fn run_experiment<'a>(experiment: &'a ExperimentConfig, arm: &mut k::Manipulator<f64>) -> ExperimentResults<'a> {
    // Create the random number generator
    let mut rng: StdRng = SeedableRng::seed_from_u64(experiment.seed);

    // Create the results to record to
    let mut results = ExperimentResults::new(&experiment);

    // Create the Experiment state which will track anything that persists between episodes
    let mut state = ExperimentState::new();

    // Run the whole experiment (each episode)
    for episode in 0..experiment.nepisodes {
        println!("=== Starting episode {} ===", episode);
        results.set_episode(episode);
        if experiment.comstr == "simulate" {
            run_simulation(experiment, &mut results, &mut rng, &mut state, arm);
        } else {
            run_episode(episode, experiment, &mut results, &mut rng, &mut state, arm);
        }
    }

    // Save the best network if mode is genetic
    match experiment.mode {
        Mode::Genetic => match state.save_best_network(&"network_weights.wghts".to_string()) {
            Ok(_) => (),
            Err(e) => panic!("Could not save network: {:?}", e),
        },
        Mode::Inference | Mode::Random => (),
    }
    results.finish();
    results
}

fn run_simulation<'a>(experiment: &'a ExperimentConfig, results: &mut ExperimentResults, rng: &mut rand::StdRng, state: &mut ExperimentState, arm: &mut k::Manipulator<f64>) {
    println!("Running simulation");

    // Create a buffer to put the commands (the run_*_episode functions need a script to write to)
    let mut f = Vec::<u8>::new();
    match experiment.mode {
        Mode::Random => run_random_episode(experiment, rng, results, &mut f),
        Mode::Genetic => run_genetic_episode(experiment, rng, results, &mut f, state, arm),
        Mode::Inference => run_inference_episode(experiment, rng, results, &mut f),
    };
}

fn run_episode<'a>(episode: u64, experiment: &'a ExperimentConfig, results: &mut ExperimentResults, rng: &mut rand::StdRng, state: &mut ExperimentState, arm: &mut k::Manipulator<f64>) {
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
        Mode::Genetic => run_genetic_episode(experiment, rng, results, &mut f, state, arm),
        Mode::Inference => run_inference_episode(experiment, rng, results, &mut f),
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

fn run_inference_episode<'a>(experiment: &'a ExperimentConfig, rng: &mut rand::StdRng, results: &mut ExperimentResults, f: &mut dyn std::io::Write) {
    // Go to random start position - but should be the same start position every time
    let mut rngcopy: StdRng = rand::SeedableRng::seed_from_u64(experiment.seed);
    let mut base: f64 = num::clamp(ANGLE_START_BASE as f64 + rngcopy.gen_range(-30.0, 30.0), ANGLE_LOWER_LIMIT_BASE as f64, ANGLE_UPPER_LIMIT_BASE as f64);
    let mut shoulder: f64 = num::clamp(ANGLE_START_SHOULDER as f64 + rngcopy.gen_range(-30.0, 30.0), ANGLE_LOWER_LIMIT_SHOULDER as f64, ANGLE_UPPER_LIMIT_SHOULDER as f64);
    let mut elbow: f64 = num::clamp(ANGLE_START_ELBOW as f64 + rngcopy.gen_range(-30.0, 30.0), ANGLE_LOWER_LIMIT_ELBOW as f64, ANGLE_UPPER_LIMIT_ELBOW as f64);
    writeln!(f, "servo {} {}", BASENUM, base);
    writeln!(f, "servo {} {}", SHOULDERNUM, shoulder);
    writeln!(f, "servo {} {}", ELBOWNUM, elbow);
    writeln!(results, "servo {} {}", BASENUM, base);
    writeln!(results, "servo {} {}", SHOULDERNUM, shoulder);
    writeln!(results, "servo {} {}", ELBOWNUM, elbow);

    // Create a network with the appropriate weights
    let mut network: network::MultilayerPerceptron = netconfig::build_network(0.0, 1.0, rng);
    match network.load_weights(&experiment.weights) {
        Err(e) => panic!("Could not load the network weights from file {}: {:?}", experiment.weights, e),
        Ok(_) => (),
    }

    // Write each step's actions to the results and script
    for _ in 0..experiment.nsteps_per_episode {
        run_step_using_network(&network, &mut base, &mut shoulder, &mut elbow, results, f);
    }
}

fn run_random_episode<'a>(experiment: &'a ExperimentConfig, rng: &mut rand::StdRng, results: &mut ExperimentResults, f: &mut dyn std::io::Write) {
    // Starting angles
    let mut base: f64 = ANGLE_START_BASE;
    let mut shoulder: f64 = ANGLE_START_SHOULDER;
    let mut elbow: f64 = ANGLE_START_ELBOW;

    // For each step, do a random step of up to 15 degrees in either direction on each servo
    for _step in 0..experiment.nsteps_per_episode {
        // Generate a bunch of values
        let randbase = rng.gen_range(-15.0, 16.0);
        let randshoulder = rng.gen_range(-15.0, 16.0);
        let randelbow = rng.gen_range(-15.0, 16.0);

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

fn run_genetic_episode<'a>(experiment: &'a ExperimentConfig, rng: &mut rand::StdRng, results: &mut ExperimentResults, f: &mut dyn std::io::Write, state: &mut ExperimentState, arm: &mut k::Manipulator<f64>) {
    // Crate a generation
    state.create_next_generation(experiment, rng);

    // Go to random start position - but should be the same start position every time
    let mut rngcopy: StdRng = rand::SeedableRng::seed_from_u64(experiment.seed);
    //let mut blahrng = thread_rng();
    //let seed = blahrng.next_u64();
    //let mut rngcopy: StdRng = rand::SeedableRng::seed_from_u64(seed);
    let base_start: f64 = num::clamp(ANGLE_START_BASE as f64 + rngcopy.gen_range(-10.0, 10.0), ANGLE_LOWER_LIMIT_BASE as f64, ANGLE_UPPER_LIMIT_BASE as f64);
    let shoulder_start: f64 = num::clamp(ANGLE_START_SHOULDER as f64 + rngcopy.gen_range(-10.0, 10.0), ANGLE_LOWER_LIMIT_SHOULDER as f64, ANGLE_UPPER_LIMIT_SHOULDER as f64);
    let elbow_start: f64 = num::clamp(ANGLE_START_ELBOW as f64 + rngcopy.gen_range(-10.0, 10.0), ANGLE_LOWER_LIMIT_ELBOW as f64, ANGLE_UPPER_LIMIT_ELBOW as f64);
    writeln!(f, "servo {} {}", BASENUM, base_start);
    writeln!(f, "servo {} {}", SHOULDERNUM, shoulder_start);
    writeln!(f, "servo {} {}", ELBOWNUM, elbow_start);
    writeln!(results, "servo {} {}", BASENUM, base_start);
    writeln!(results, "servo {} {}", SHOULDERNUM, shoulder_start);
    writeln!(results, "servo {} {}", ELBOWNUM, elbow_start);

    // Evaluate each network in the generation
    let mut evaluations = Vec::<f64>::new();
    for (networkidx, network) in state.networks.iter().enumerate() {
        writeln!(results, "network {}", networkidx);

        // For each step, get the values for each joint delta from a forward pass through the current network
        let (mut base, mut shoulder, mut elbow) = (base_start, shoulder_start, elbow_start);
        for _step in 0..experiment.nsteps_per_episode {
            run_step_using_network(&network, &mut base, &mut shoulder, &mut elbow, results, f);
        }

        // Now figure out how fit this network is based on how close the arm ended up to the goal position
        match arm.set_joint_angles(&vec![base, shoulder, elbow, 0.0]) {
            Ok(_) => (),
            Err(e) => panic!("Problem setting joint angles: {:?}", e),
        }
        let end = arm.end_transform().translation.vector;
        let (endx, endy, endz) = (end[0], end[1], end[2]);
        let (dx, dy, dz) = (endx - experiment.target.vector[0], endy - experiment.target.vector[1], endz - experiment.target.vector[2]);
        let fitness = calculate_fitness(dx, dy, dz);
        writeln!(results, "Fitness for network {} {}", networkidx, fitness);
        evaluations.push(fitness);

        // Put the joints back to their start positions for the next network
        writeln!(f, "servo {} {}", BASENUM, base_start);
        writeln!(f, "servo {} {}", SHOULDERNUM, shoulder_start);
        writeln!(f, "servo {} {}", ELBOWNUM, elbow_start);
        writeln!(results, "servo {} {}", BASENUM, base_start);
        writeln!(results, "servo {} {}", SHOULDERNUM, shoulder_start);
        writeln!(results, "servo {} {}", ELBOWNUM, elbow_start);
    }

    for fitness in evaluations {
        state.add_fitness(fitness);
    }
}

fn run_step_using_network(network: &network::MultilayerPerceptron, base: &mut f64, shoulder: &mut f64, elbow: &mut f64, results: &mut ExperimentResults, f: &mut dyn std::io::Write) {
    let input = na::DVector::<f64>::from_vec(network.input_length(), vec!(*base, *shoulder, *elbow));
    let mut output = network.forward(&input);

    // Clamp output deltas to -15, +15
    output[0] = num::clamp(output[0], -15.0, 15.0);
    output[1] = num::clamp(output[1], -15.0, 15.0);
    output[2] = num::clamp(output[2], -15.0, 15.0);

    // Add the resulting values from the network to the current angles
    *base += output[0];
    *shoulder += output[1];
    *elbow += output[2];

    // Clamp the joints to between min and max for each joint
    *base = num::clamp(*base, ANGLE_LOWER_LIMIT_BASE, ANGLE_UPPER_LIMIT_BASE);
    *shoulder = num::clamp(*shoulder, ANGLE_LOWER_LIMIT_SHOULDER, ANGLE_UPPER_LIMIT_SHOULDER);
    *elbow = num::clamp(*elbow, ANGLE_LOWER_LIMIT_ELBOW, ANGLE_UPPER_LIMIT_ELBOW);

    // Write to the file
    writeln!(f, "servo {} {}", BASENUM, base);
    writeln!(f, "servo {} {}", SHOULDERNUM, shoulder);
    writeln!(f, "servo {} {}", ELBOWNUM, elbow);

    // Also write to results
    writeln!(results, "servo {} {}", BASENUM, base);
    writeln!(results, "servo {} {}", SHOULDERNUM, shoulder);
    writeln!(results, "servo {} {}", ELBOWNUM, elbow);
}

/// Calculate a value that is higher the closer dx, dy, and dz are to zero without.
fn calculate_fitness(dx: f64, dy: f64, dz: f64) -> f64 {
    let distance = (dx * dx + dy * dy + dz * dz).sqrt();
    0.7 - distance // 0.7ish is the diameter of the sphere around the arm
    //1.0 / (distance + 1E-9)
}
