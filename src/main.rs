use std::{
    collections::{BTreeMap, BTreeSet},
    fs::File,
    path::PathBuf,
};

use anyhow::Context;
use rayon::prelude::*;
use lmers::workflow::{
    input_data::WorkflowInput,
    runner::{cached_read_stack, RunnerOutput},
    step::Step,
    workflow_data::{LayerStorage, Window},
};

use clap::Parser;

/// Start a LME modeling process
#[derive(Parser, Debug)]
struct Args {
    /// Specify the entrypoint file path.
    ///
    /// The parent directory of the file will be the working directory
    #[clap(short = 'i')]
    input_file: String,
    /// Specify the checkpoint name for restart.
    ///
    /// The LME will find the checkpoint file under `.checkpoint` folder
    /// in the same directory of the entrypoint file, load the status and
    /// start from the step after the checkpoint in step sequence.
    #[clap(short = 'c')]
    checkpoint: Option<String>,
    /// Speicify the stop before a checkpoint/bookmark
    ///
    /// For a normal step without `load` property, the LME won't execute the step,
    /// but for step with property, the steps in `load` will be executed and then
    /// stopped.
    #[clap(short = 's')]
    stop_at: Option<String>,
    /// Display details of the step before execute it.
    #[clap(long)]
    verbose: bool,
    /// Remove unused layers in the on-disk database each time create a checkpoint.
    #[clap(long)]
    clean: bool,
}

fn main() {
    let args = Args::parse();
    let entrypoint = PathBuf::from(args.input_file);
    let entrypoint = std::fs::canonicalize(entrypoint)
        .with_context(|| "Unable to get absolute path of the entrypoint file, does it exists?")
        .unwrap();
    let working_directory = entrypoint.parent().expect("Invalid entrypoint file path");
    std::env::set_current_dir(working_directory).expect(&format!(
        "Unable to set {:?} as working directory",
        working_directory
    ));
    let entrypoint_filename = entrypoint
        .file_name()
        .expect("Invalid entrypoint file path");
    let input: WorkflowInput = serde_yaml::from_reader(
        File::open(entrypoint_filename)
            .with_context(|| {
                format!(
                    "Failed to open {:?} in {:?}",
                    entrypoint_filename, working_directory
                )
            })
            .unwrap(),
    )
    .unwrap();

    let checkpoint_list = input
        .steps
        .0
        .iter()
        .filter_map(|step| step.name.as_ref().map(|item| item.to_string()))
        .collect::<Vec<_>>();

    set_path(input.binaries).unwrap();

    let (mut current_window, steps) = if let Some(checkpoint) = &args.checkpoint {
        let num_of_steps = input.steps.0.len();
        let steps = input
            .steps
            .0
            .into_iter()
            .skip_while(|step| step.name.as_ref() != Some(checkpoint))
            .skip(1)
            .collect::<Vec<Step>>();
        println!(
            "Try to start from checkpoint {}, {} steps will be skipped",
            checkpoint,
            num_of_steps - steps.len()
        );
        let checkpoint = PathBuf::from(".checkpoint").join(checkpoint);
        let checkpoint = File::open(&checkpoint)
            .with_context(|| format!("Unable to open the checkpoint file {:?}", checkpoint))
            .unwrap();
        let checkpoint: Window = serde_json::from_reader(checkpoint)
            .with_context(|| format!("Failed to deserialize the file of given checkpoint"))
            .unwrap();
        (checkpoint, steps)
    } else {
        std::fs::create_dir_all(".checkpoint")
            .with_context(|| "Unable to prepare checkpoint direcotry")
            .unwrap();
        (BTreeMap::from([("LME".to_string(), vec![])]), input.steps.0)
    };

    let steps = if let Some(stop_at) = args.stop_at {
        let current_steps = steps.len();
        let steps = steps
            .into_iter()
            .take_while(|step| {
                step.bookmark.as_ref() != Some(&stop_at) && step.name.as_ref() != Some(&stop_at)
            })
            .collect::<Vec<_>>();
        println!(
            "Will stop before checkpoint/bookmark {}, {} steps won't execute",
            stop_at,
            current_steps - steps.len()
        );
        steps
    } else {
        steps
    };

    let num_of_steps = steps.len();

    let layer_storage = LayerStorage::new(PathBuf::from(".checkpoint").join(".layers.db"));

    for (idx, step) in steps.into_iter().enumerate() {
        if let Some(from) = step.from.as_ref() {
            let checkpoint = PathBuf::from(".checkpoint").join(from);
            let checkpoint = File::open(&checkpoint)
                .with_context(|| format!("Unable to open the checkpoint file {:?}", checkpoint))
                .unwrap();
            current_window = serde_json::from_reader(checkpoint)
                .with_context(|| {
                    format!("Failed to deserialize the checkpoint file for the {}", from)
                })
                .unwrap();
        };
        println!(
            "Step {}/{}, input {} structures",
            idx + 1,
            num_of_steps,
            current_window.len()
        );
        if args.verbose {
            println!("{:#?}", step)
        }
        let result = step
            .run
            .execute(&input.base, &current_window, &layer_storage)
            .unwrap();

        let cache_generated_stacks = |generated_stacks: &BTreeMap<String, Vec<u64>>| {
            generated_stacks
                .par_iter()
                .map(|(_, stack_path)| cached_read_stack(&input.base, &layer_storage, &stack_path))
                .collect::<Result<Vec<_>, _>>()
        };

        match result {
            RunnerOutput::None => {}
            RunnerOutput::SingleWindow(window) => {
                cache_generated_stacks(&window).unwrap();
                current_window = window;
            }
            RunnerOutput::MultiWindow(windows) => {
                if let Some(name) = step.name.as_ref() {
                    for (window_name, window) in &windows {
                        cache_generated_stacks(window).unwrap();
                        let name = format!("{}_{}", name, window_name);
                        let checkpoint = File::create(PathBuf::from(".checkpoint").join(&name))
                            .with_context(|| format!("Failed to create checkpoint {}", name))
                            .unwrap();
                        serde_json::to_writer(checkpoint, &window)
                            .with_context(|| {
                                format!("Failed to serialize the checkpoint information")
                            })
                            .unwrap();
                        println!("Checkpoint {} created", &name);
                    }
                }
                current_window = BTreeMap::new();
                for (_, window) in windows {
                    current_window.extend(window);
                }
            }
        }
        if let Some(name) = step.name {
            let checkpoint = File::create(PathBuf::from(".checkpoint").join(&name))
                .with_context(|| format!("Failed to create checkpoint {}", name))
                .unwrap();
            serde_json::to_writer(checkpoint, &current_window)
                .with_context(|| format!("Failed to serialize the checkpoint information"))
                .unwrap();
            println!("Checkpoint {} created", &name);
        }
    }
    if args.clean {
        clean_unused_layers(&checkpoint_list, &layer_storage);
    }
    println!("finished");
}

fn set_path(user_specified_paths: Vec<PathBuf>) -> anyhow::Result<()> {
    let current_binary_directory = PathBuf::from(
        std::env::current_exe()?
            .parent()
            .expect("Binary file must have a parent directory"),
    );
    let working_directory_bin = std::env::current_dir()?.join("bin");
    let current_path_var = std::env::var_os("PATH").unwrap_or_default();
    let current_path_var = std::env::split_paths(&current_path_var);
    let mut paths = user_specified_paths;
    paths.extend([working_directory_bin, current_binary_directory]);
    paths.extend(current_path_var);
    let paths = std::env::join_paths(paths)?;
    std::env::set_var("PATH", paths);
    Ok(())
}

fn clean_unused_layers(checkpoint_list: &Vec<String>, storage: &LayerStorage) {
    let checkpoints = checkpoint_list
        .iter()
        .filter_map(|checkpoint_name| -> Option<Window> {
            let checkpoint = PathBuf::from(".checkpoint").join(checkpoint_name);
            let checkpoint = File::open(checkpoint).ok();
            if let Some(checkpoint) = checkpoint {
                Some(
                    serde_json::from_reader(checkpoint)
                        .with_context(|| {
                            format!("checkpoint {} is unable to load", checkpoint_name)
                        })
                        .unwrap(),
                )
            } else {
                None
            }
        });
    let mut retains = BTreeSet::new();
    for checkpoint in checkpoints {
        for structure in checkpoint.values() {
            retains.extend(structure.iter().copied());
        }
    }
    storage.retain(&retains);
}
