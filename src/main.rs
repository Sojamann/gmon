#![feature(more_qualified_paths)]
use std::env;
use std::fs;

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

    let config_file = env::var("HOME").expect("$HOME not set") + "/.config/gmon/config";
    let config_str = fs::read_to_string(config_file).expect("no config file");
    let config: Config = toml::from_str(&config_str).expect("invalid config file");

    let gapi = gitlab::GitlabBuilder::new(config.host, config.token)
        .build_async()
        .await
        .expect("gitlab err");

    match &cli.command {
        Command::Pipelines(args) => {
            cmds::pipelines::run(gapi, ratatui::init(), args).await;
        }
        Command::Pipeline(args) => {
            cmds::pipeline::run(gapi, args).await;
        }
    }
    std::panic::set_hook(Box::new(move |_| {
        ratatui::restore();
    }));
    ratatui::restore();
}
