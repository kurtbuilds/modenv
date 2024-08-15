use crate::Cli;
use std::{env, process};
use env2::EnvFile;

/// Run a command with the environment variables set. If a envfile is not provided, defaults to .env.
///
/// If the command has command line flags, you might need to use -- to separate the command from modenv flags.
///
/// Precedence for the env var values flows from left to right, so lowest is existing env values,
/// then env files from left to right, and highest is any passed in vars.
///
/// Example:
///
///     # .env
///     # FOO=2
///
///     # .env.local
///     # FOO=3
///
///     FOO=1 modenv run -e .env,.env.local -- FOO=4 echo hi
///
/// Will run with FOO=4, because it is the highest precedence.
#[derive(clap::Parser, Debug)]
pub struct Run {
    /// The command to run. The command can begin with variables in KEY=VALUE format.
    pub command: Vec<String>,
}

impl Run {
    pub fn run(self, cli: &Cli) {
        let paths = cli.all_provided_envfiles();
        for path in paths {
            let envfile = EnvFile::read(path);
            for (key, value) in &envfile {
                env::set_var(key, value);
            }
        }
        let mut rest_of_command = self.command.into_iter();
        let mut maybe_command = rest_of_command.next().expect("No command provided");
        while maybe_command.contains('=') {
            let pair = maybe_command.splitn(2, '=').collect::<Vec<_>>();
            env::set_var(pair[0], pair[1]);
            maybe_command = rest_of_command.next().expect("No command provided");
        }
        let command = maybe_command;
        if cli.verbose {
            for (key, value) in env::vars() {
                println!("{}={}", key, value);
            }
            println!("{} {}", command, rest_of_command.clone().collect::<Vec<_>>().join(" "));
        }
        process::exit(process::Command::new(command)
            .args(rest_of_command)
            .status()
            .expect("Failed to execute command")
            .code().unwrap())

    }
}