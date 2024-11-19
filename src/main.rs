#![feature(more_qualified_paths)]
use std::env;
use std::fs;
use std::process;

use clap::{Parser, Subcommand};

mod cmds;
mod config;
mod events;
mod fetchers;
mod gitlab_ref;

use crate::config::*;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true)]
struct Cli {
    #[clap(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    Pipelines(cmds::pipelines::PipelinesArgs),
    Pipeline(cmds::pipeline::PipelineArgs),
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    let config_file = env::var("HOME").expect("$HOME not set") + "/.config/gmon/config.toml";
    let config_str = match fs::read_to_string(&config_file) {
        Ok(s) => s,
        Err(s) => {
            eprintln!("failed reading config {} due to: {}", &config_file, &s);
            process::exit(2);
        },
    };
    let config: Config = match toml::from_str(&config_str) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("failed loading config {} due to: {}", &config_file, e.message());
            process::exit(2);
        },
    };

    let gapi = match gitlab::GitlabBuilder::new(config.host, config.token)
        .cert_insecure()
        .build_async()
        .await {

            Ok(api) => api,
            Err(err) => {
                eprintln!("connection to gitlab failed due to: {}", err);
                process::exit(1);
            },
    };

    let original_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        ratatui::restore();
        original_hook(info);
    }));

    match &cli.command {
        Command::Pipelines(args) => {
            cmds::pipelines::run(gapi, args).await;
        }
        Command::Pipeline(args) => {
            cmds::pipeline::run(gapi, args).await;
        }
    }
    ratatui::restore();
}
