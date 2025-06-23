use crate::cmd::check::check;
use crate::cmd::default::default_cmd_workflow;
use crate::cmd::init::init;
use crate::cmd::log::log;
use crate::cmd::{Cli, Commands};
use crate::config::BGitConfig;

mod auth;
mod bgit_error;
mod cmd;
mod config;
mod constants;
mod events;
mod flags;
mod hook_executor;
mod llm_tools;
mod rules;
mod step;
mod util;
mod workflow_queue;
mod workflows;

fn main() {
    let cli_instance_wrap = Cli::new();

    if let Some(cli_instance) = cli_instance_wrap {
        env_logger::Builder::from_env(env_logger::Env::default().default_filter_or(
            match cli_instance.verbose {
                0 => "warn",
                1 => "info",
                2 => "debug",
                _ => "trace",
            },
        ))
        .format_timestamp_secs()
        .init();

        let bgit_config = BGitConfig::load().unwrap_or_else(|err| {
            err.print_error();
            std::process::exit(1);
        });

        match cli_instance.command {
            Some(Commands::Log) => log(bgit_config),
            Some(Commands::Init) => init(bgit_config),
            Some(Commands::Check) => check(bgit_config),
            None => default_cmd_workflow(bgit_config),
        }
    }
}
