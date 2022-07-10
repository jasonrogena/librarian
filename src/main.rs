mod config;
mod error;
mod fs_notify;
mod library;
mod mime_type;
mod template;
use clap::{Args, Parser, Subcommand};
use std::collections::HashSet;
use std::path::Path;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::mpsc::channel;
use std::thread;
use std::time::Duration;

static GLOBAL_THREAD_COUNT: AtomicUsize = AtomicUsize::new(0);
static GLOBAL_FAILED_TREADS: AtomicUsize = AtomicUsize::new(0);

/// Runs pre-configured commands against a group of files that match a set of filters
#[derive(Debug, Parser)]
#[clap(name = "fs-librarian")]
#[clap(about = "Goes through file types inside directories and does with them as you wish", long_about = None)]
struct Cli {
    #[clap(subcommand)]
    command: Commands,
}

#[derive(Debug, Subcommand)]
enum Commands {
    /// Use OS specific mechanisms, like Linux's inotify, to watch for changes in directories that will trigger Librarian when new files are added or changed
    #[clap(arg_required_else_help = true)]
    Watch {
        /// Path to the configuration file to use
        config_path: String,
    },

    /// Run Librarian once
    #[clap(arg_required_else_help = true)]
    SingleShot {
        /// Path to the configuration file to use
        config_path: String,
    },

    /// Debugging tools to help you to better work with Librarian
    Test(Test),
}

#[derive(Debug, Args)]
#[clap(args_conflicts_with_subcommands = true)]
struct Test {
    #[clap(subcommand)]
    command: TestCommands,
}

#[derive(Debug, Subcommand)]
enum TestCommands {
    /// Check a file's MIME type
    Mime {
        #[clap(value_parser)]
        file_path: String,
    },
}

fn main() {
    let args = Cli::parse();
    match args.command {
        Commands::Watch { config_path } => {
            watch(&config_path);
        }
        Commands::SingleShot { config_path } => {
            single_shot(&config_path);
        }
        Commands::Test(t) => {
            test(&t);
        }
    }
}

fn get_config(config_path: &String) -> config::Config {
    match config::Config::new(config_path) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("{}", e);
            std::process::exit(exitcode::CONFIG);
        }
    }
}

fn watch(config_path: &String) {
    let conf = get_config(config_path);

    let mut paths: HashSet<String> = HashSet::new();
    // let mut libraries = Vec::new();
    for cur_lib_config in conf.libraries {
        for cur_dir in cur_lib_config.1.filter.directories {
            paths.insert(cur_dir);
        }
    }

    let (on_event_sender, on_event_receiver) = channel();
    let (mut notify_obj, _) =
        fs_notify::Notify::new(&conf.fs_watch, &paths, on_event_sender).unwrap();
    let config_path_clone = config_path.clone();
    thread::spawn(move || loop {
        let path = match on_event_receiver.recv() {
            Ok(p) => p,
            Err(e) => {
                eprintln!("{}", e);
                continue;
            }
        };

        let conf = get_config(&config_path_clone);
        for cur_lib_config in conf.libraries {
            let cur_lib = library::Library::new(&cur_lib_config.1);
            if !cur_lib.contains_path(Path::new(&path)) {
                continue;
            }

            let number = match cur_lib.process(Some(Path::new(&path))) {
                Ok(n) => n,
                Err(e) => {
                    eprintln!("{}", e);
                    0
                }
            };

            if number > 0 {
                println!(
                    "Processed '{}' as part of the {} library",
                    path, cur_lib_config.0
                );
            }
        }
    });

    single_shot(config_path);
    notify_obj.watch();
}

fn single_shot(config_path: &String) {
    let conf = get_config(config_path);

    for cur_conf in conf.libraries {
        GLOBAL_THREAD_COUNT.fetch_add(1, Ordering::SeqCst);
        thread::spawn(move || {
            match library::Library::new(&cur_conf.1).process(None) {
                Ok(k) => {
                    println!("Processed {} files in the {} library", k, cur_conf.0);
                }
                Err(e) => {
                    eprintln!("{}", e);
                    GLOBAL_FAILED_TREADS.fetch_add(1, Ordering::SeqCst);
                }
            }

            GLOBAL_THREAD_COUNT.fetch_sub(1, Ordering::SeqCst);
        });
    }

    while GLOBAL_THREAD_COUNT.load(Ordering::SeqCst) != 0 {
        thread::sleep(Duration::from_millis(1));
    }

    if GLOBAL_FAILED_TREADS.load(Ordering::SeqCst) > 0 {
        std::process::exit(exitcode::DATAERR);
    }
}

fn test(test: &Test) {
    match &test.command {
        TestCommands::Mime { file_path } => mime(file_path),
    }
}

fn mime(file_path: &str) {
    match mime_type::File::new(Path::new(file_path)).get_mime_type() {
        Err(e) => {
            eprintln!("{}", e);
            std::process::exit(exitcode::DATAERR);
        }
        Ok(m) => {
            println!("{}", m)
        }
    };
}
