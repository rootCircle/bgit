pub(crate) mod check;
pub(crate) mod create_creds;
pub(crate) mod default;
pub(crate) mod init;
pub(crate) mod log;

use std::io;

use clap::{Command, CommandFactory, Parser, Subcommand};
use clap_complete::{Generator, Shell, generate};
use colored::Colorize;

#[derive(Debug, Parser)] // requires `derive` feature
#[command(name = "bgit", version, author, about, long_about = None)]
#[command(bin_name = "bgit")]
pub struct Cli {
    #[command(subcommand)]
    pub(crate) command: Option<Commands>,

    /// Generate Shell Completions
    #[arg(long = "completions", value_enum)]
    completions: Option<Shell>,

    /// Increase verbosity (-v, -vv, -vvv), 0 = WARN, 1 = INFO, 2 = DEBUG, 3 = TRACE
    #[arg(short, long, action = clap::ArgAction::Count)]
    pub(crate) verbose: u8,
}

#[derive(Debug, Subcommand)]
pub enum Commands {
    /// Print commit history
    Log,

    /// Initialize bgit
    Init,

    /// Do maintenance tasks
    Check,
    #[command(name = "create-creds")]
    CreateCreds,
}

fn print_completions<G: Generator>(generator: G, cmd: &mut Command) {
    generate(
        generator,
        cmd,
        cmd.get_name().to_string(),
        &mut io::stdout(),
    );
}

impl Cli {
    pub fn new() -> Option<Self> {
        let opt = Self::parse();
        if let Some(completions) = opt.completions {
            let mut cmd = Cli::command();
            eprintln!("Generating completion file for {completions:?}...");
            print_completions(completions, &mut cmd);

            match completions {
                Shell::Zsh => {
                    eprintln!("\n\n{}\n    {}",
                        "Run the following command below to add it permanently to your shell:".bright_blue(),
                        "bgit --completions=zsh | sudo tee /usr/local/share/zsh/site-functions/_bgit".yellow()
                    );
                }
                Shell::Bash => {
                    eprintln!(
                        "\n\n{}\n    {}",
                        "Run the following command below to add it permanently to your shell:"
                            .bright_blue(),
                        "bgit --completions=bash | sudo tee /etc/bash_completion.d/bgit.bash"
                            .yellow()
                    );
                }
                Shell::Fish => {
                    eprintln!("\n\n{}\n    {}",
                        "Run the following command below to add it permanently to your shell:".bright_blue(),
                        "bgit --completions=fish > ~/.local/share/fish/generated_completions/bgit.fish".yellow()
                    );
                }
                Shell::PowerShell => {
                    eprintln!(
                        "{}\n    {}",
                        "Run the following command below to add it permanently to your shell:"
                            .bright_blue(),
                        "bgit --completions=powershell | Out-File -FilePath $PROFILE -Append"
                            .yellow()
                    );
                }
                // Figure it out yourself XD
                _ => {}
            }

            None
        } else {
            Some(opt)
        }
    }
}
