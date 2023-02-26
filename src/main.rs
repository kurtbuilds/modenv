mod command;

use std::path::{Path, PathBuf};
use std::str::FromStr;

use clap::{Parser, Subcommand};

use command::*;

/// Manage environment files and keep them consistent.
///
/// If you do not provide a subcommand, modenv uses the add command.
///
/// Example workflow:
///
/// 1. Create the .env files.
///
///     modenv init
///
/// 2. Add KEY=VALUE to .env. -a adds the KEY, but not the value, to all other .env files.
/// The add command is assumed if no subcommand is provided, so it is optional.
///
///     modenv -a KEY=VALUE
///     modenv add -a KEY=VALUE
///
/// 3. Once you have the production value, add KEY=PROD_VALUE to .env.production.
///
///     modenv -p KEY=PROD_VALUE
///
/// 4. Check keys in other envfiles based on .env. This is a dry-run. Use -f to perform
/// changes.
///
///     modenv check
#[derive(Parser)]
#[command(author, version, about)]
pub struct Cli {
    #[clap(subcommand)]
    command: Option<Command>,

    #[clap(short, long, global = true)]
    verbose: bool,

    /// Use .env.production as the reference file.
    #[clap(short, long, global = true)]
    production: bool,

    /// Use .env.development as the reference file.
    #[clap(short, long, global = true)]
    development: bool,

    /// Use .env as the reference file.
    #[clap(short, long, global = true)]
    normal: bool,

    /// Use .env as the reference file.
    #[clap(short = 'x', long, global = true)]
    example: bool,

    /// Use provided path as the reference file. Can be multiple paths separated by commas.
    #[clap(short, long, global = true)]
    env_file: Vec<String>,

    pairs: Vec<String>,
}

impl Cli {
    fn provided_envfile(&self) -> PathBuf {
        self.all_provided_envfiles().remove(0)
    }

    fn all_provided_envfiles(&self) -> Vec<PathBuf> {
        let mut paths = vec![];
        if self.production {
            paths.push(PathBuf::from_str(".env.production").unwrap());
        }
        if self.development {
            paths.push(PathBuf::from_str(".env.development").unwrap());
        }
        if self.normal {
            paths.push(PathBuf::from_str(".env").unwrap());
        }
        if self.example {
            paths.push(PathBuf::from_str(".env.example").unwrap());
        }
        if !self.env_file.is_empty() {
            for f in &self.env_file {
                paths.extend(f.split(',').map(|f| PathBuf::from_str(f).unwrap()));
            }
        }
        if paths.is_empty() {
            paths.push(default_envfile());
        }
        paths
    }

    fn fs_envfiles_excluding(&self, exclude: &Path) -> Vec<PathBuf> {
        std::fs::read_dir(".")
            .unwrap()
            .filter_map(|entry| entry.ok())
            .map(|entry| PathBuf::from(entry.path().file_name().unwrap()))
            .filter(|path| path.file_name().unwrap().to_str().unwrap().starts_with(".env"))
            .filter(|path| path != exclude)
            .collect()
    }
}

#[derive(Subcommand, Debug)]
enum Command {
    /// Create .env, .env.example, and .env.production, and add .env to .gitignore.
    Init(Init),
    /// Add key/value pair(s) to an envfile.
    Add(Add),
    /// Check envfiles for inconsistencies.
    Check(Check),
    #[clap(alias = "rm")]
    Remove(Remove),
    /// Run a command with the environment variables set. If a envfile is not provided,
    /// defaults to .env. Run `modenv run -h` for information on variable priority.
    Run(Run),
    /// Prints the pairs from an envfile. Can be used for export: $ export "$(modenv show)"
    Show(Show),
}

fn default_envfile() -> PathBuf {
    for path in &[".env.local",
        ".env.development",
        ".env",
        "../.env",
    ] {
        let p = PathBuf::from_str(path).unwrap();
        if p.exists() {
            return p;
        }
    }
    panic!("Could not find .env.local, .env.development, .env, or ../.env file.")
}

fn main() {
    let mut cli = Cli::parse();

    let command: Option<Command> = std::mem::replace(&mut cli.command, None);
    match command.unwrap_or_else(|| Command::Add(Add {
        force: false,
        all: false,
        pairs: std::mem::replace(&mut cli.pairs, vec![])
    })) {
        Command::Init(init) => init.run(),
        Command::Add(add) => add.run(&cli),
        Command::Check(check) => check.run(&cli),
        Command::Remove(remove) => remove.run(&cli),
        Command::Run(run) => run.run(&cli),
        Command::Show(show) => show.run(&cli),
    }
}

pub fn resolve_pairs(pairs: &[String]) -> Vec<env2::Pair> {
    pairs.iter()
        .map(|pair| {
            if pair.contains('=') {
                env2::Pair::from(pair.as_str())
            } else {
                env2::Pair { key: pair.to_string(), value: "".to_string() }
            }
        })
        .collect()
}
