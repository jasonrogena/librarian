mod config;
mod error;
mod library;
mod template;
use clap::{Parser, Subcommand};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::thread;
use std::time::Duration;

static GLOBAL_THREAD_COUNT: AtomicUsize = AtomicUsize::new(0);
static GLOBAL_FAILED_TREADS: AtomicUsize = AtomicUsize::new(0);

/// Goes through files inside directories and does with them as you wish
#[derive(Debug, Parser)]
#[clap(name = "librarian")]
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
    OneOff {
        /// Path to the configuration file to use
        config_path: String,
    },
}

fn main() {
    let args = Cli::parse();
    match args.command {
        Commands::Watch { config_path } => {
            println!("Not implemented yet for '{}'", config_path);
        }
        Commands::OneOff { config_path } => {
            one_off(&config_path);
        }
    }
}

fn one_off(config_path: &String) {
    let conf = match config::Config::new(config_path) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("{}", e);
            std::process::exit(exitcode::CONFIG);
        }
    };

    for cur_conf in conf.libraries {
        GLOBAL_THREAD_COUNT.fetch_add(1, Ordering::SeqCst);
        thread::spawn(move || {
            match library::Library::new(cur_conf.1).process() {
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
