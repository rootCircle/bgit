use crate::cmd::check::check;
use crate::cmd::default::default_cmd_workflow;
use crate::cmd::init::init;
use crate::cmd::log::log;
use crate::cmd::{Cli, Commands};

mod bgit_error;
mod cmd;
mod events;
mod hook_executor;
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

        match cli_instance.command {
            Some(Commands::Log) => log(),
            Some(Commands::Init) => init(),
            Some(Commands::Check) => check(),
            None => default_cmd_workflow(),
        }
    }
}
