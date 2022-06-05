use std::{env, fs};
use std::path::{Path, PathBuf};
use std::process::exit;
use std::process;
use std::str::FromStr;

use clap::{Arg, ArgMatches, Command};

use crate::command::CheckOptions;
use crate::file::{EnvFile, Pair};

mod command;
mod file;

const VERSION: &str = env!("CARGO_PKG_VERSION");
const NAME: &str = env!("CARGO_PKG_NAME");

fn exit_with(message: &str) -> ! {
    eprintln!("{}", message);
    exit(1);
}

fn default_reference_envfile() -> PathBuf {
    for path in &[".env.local",
        ".env.development",
        ".env"] {
        let p = PathBuf::from_str(path).unwrap();
        if p.exists() {
            return p;
        }
    }
    exit_with("Could not find .env.local, .env.development, or .env file.")
}

fn add_single_reference_file_args(app: Command) -> Command {
    app
        .arg(Arg::new("production")
            .short('p')
            .long("production")
            .help("Use .env.production as the reference file.")
        )
        .arg(Arg::new("development")
            .short('d')
            .long("development")
            .help("Use .env.development as the reference file.")
        )
        .arg(Arg::new("normal")
            .short('n')
            .long("normal")
            .help("Use .env as the reference file.")
        )
        .arg(Arg::new("example")
            .short('x')
            .long("example")
            .help("Use .env.example.")
        )
}


fn add_reference_file_args(app: Command) -> Command {
    add_single_reference_file_args(app)
        .arg(Arg::new("env")
            .short('e')
            .long("env")
            .value_name("FILE")
            .help("Use FILE as the reference envfile.")
        )
}

fn resolve_reference_file(sub_match: &ArgMatches) -> PathBuf {
    if sub_match.is_present("production") {
        PathBuf::from_str(".env.production").unwrap()
    } else if sub_match.is_present("development") {
        PathBuf::from_str(".env.development").unwrap()
    } else if sub_match.is_present("normal") {
        PathBuf::from_str(".env").unwrap()
    } else if sub_match.is_present("example") {
        PathBuf::from_str(".env.example").unwrap()
    } else if sub_match.value_of("env").is_some() {
        PathBuf::from_str(sub_match.value_of("env").unwrap()).unwrap()
    } else {
        default_reference_envfile()
    }
}


fn resolve_multiple_reference_files(sub_match: &ArgMatches) -> Vec<PathBuf> {
    let mut reference_files = Vec::new();
    if sub_match.is_present("production") {
        reference_files.push(PathBuf::from_str(".env.production").unwrap());
    } else if sub_match.is_present("development") {
        reference_files.push(PathBuf::from_str(".env.development").unwrap());
    } else if sub_match.is_present("normal") {
        reference_files.push(PathBuf::from_str(".env").unwrap());
    } else if sub_match.is_present("example") {
        reference_files.push(PathBuf::from_str(".env.example").unwrap());
    } else if let Some(env) = sub_match.values_of("env") {
        reference_files.extend(env.into_iter().map(|s| PathBuf::from_str(s).unwrap()));
    }
    reference_files
}


fn find_all_envfile_paths(ignore: &PathBuf) -> Vec<PathBuf> {
    fs::read_dir(".")
        .unwrap()
        .filter_map(|entry| entry.ok())
        .map(|entry| PathBuf::from(entry.path().file_name().unwrap()))
        .filter(|path| path.file_name().unwrap().to_str().unwrap().starts_with(".env") && path != ignore)
        .collect()
}


fn quit_if_value_exists(pairs: &Vec<Pair>, envfile: &EnvFile, other_envfiles: &Vec<EnvFile>) {
    for pair in pairs {
        if envfile.has_value(&pair.key) {
            exit_with(&format!("Key {} already exists in {}. Use -f to force.", pair.key, envfile.path.display()));
        }
        for env in other_envfiles {
            if env.has_value(&pair.key) {
                exit_with(&format!("Key {} already exists in {}. Use -f to force.", pair.key, env.path.display()));
            }
        }
    }
}


fn resolve_pairs(sub_match: &ArgMatches) -> Vec<Pair> {
    let mut input = sub_match.values_of("pairs").unwrap().peekable();
    let mut pairs = Vec::new();
    while input.peek().is_some() {
        let inp = input.next().unwrap();
        if inp.contains('=') {
            let mut split = inp.splitn(2, '=');
            pairs.push(Pair { key: split.next().unwrap().into(), value: split.next().unwrap().into() });
        } else {
            pairs.push(Pair { key: inp.into(), value: input.next().unwrap_or("").into() });
        }
    }
    pairs
}


fn main() {
    let app = Command::new(NAME)
        .version(VERSION)
        .about("Manage environment files and keep them consistent.

If you do not provide a subcommand, modenv uses the add command.

Example workflow:

1. Create the .env files.

    modenv init

2. Add KEY=VALUE to .env. -a adds the KEY, but not the value, to all other .env files.
The add command is assumed if no subcommand is provided, so it is optional.

    modenv -a KEY=VALUE
    modenv add -a KEY=VALUE

3. Once you have the production value, add KEY=PROD_VALUE to .env.production.

    modenv -p KEY=PROD_VALUE

4. Check keys in other envfiles based on .env. This is a dry-run. Use -f to perform
changes.

    modenv check")
        .arg_required_else_help(true)
        .subcommand(add_reference_file_args(Command::new("add"))
            .about("Add key/value pair(s) to an envfile.")
            .arg(Arg::new("force")
                .short('f')
                .long("force")
                .takes_value(false)
                .help("Overwrite key value if it already exists.")
            )
            .arg(Arg::new("all")
                .short('a')
                .long("all")
                .takes_value(false)
            )
            .arg(Arg::new("pairs").required(true).min_values(1))
        )
        .subcommand(add_reference_file_args(Command::new("check"))
            .about("Check envfiles for inconsistencies.")
            .arg(Arg::new("files")
                .min_values(0)
                .help("By default, modenv scans all env files. If you specify FILES, it only checks those files.")
            )
            .arg(Arg::new("force")
                .short('f')
                .long("force")
                .help("Update both the reference file and other env files to have the same ordering, set of keys, and comments.")
            )
        )
        .subcommand(add_reference_file_args(Command::new("rm"))
            .arg(Arg::new("all")
                .short('a')
                .long("all")
                .takes_value(false)
            )
            .arg(Arg::new("pairs").required(true).min_values(1))
            .after_help("Remove key(s) from an envfile.")
        )
        .subcommand(Command::new("init")
            .about("Create .env, .env.example, and .env.production, and add .env to .gitignore.")
        )
        .subcommand(add_single_reference_file_args(Command::new("show"))
            .about("Prints the pairs from an envfile. Can be used for export: $ export \"$(modenv show)\"")
        )
        .subcommand(add_single_reference_file_args(Command::new("run"))
            .about("Run a command with the environment variables set. If a envfile is not provided, defaults to .env. Run `modenv run -h` for information on variable priority.")
            .after_help("Run a command with the environment variables set. If a envfile is not provided, defaults to .env.

If the command has command line flags, you might need to use -- to separate the command from modenv flags.

Precedence for the env var values flows from left to right
, so lowest is existing env values, then env files from left to right,
and highest is any passed in vars.

Example:

    # .env
    # FOO=2

    # .env.local
    # FOO=3

    FOO=1 modenv run -e .env,.env.local -- FOO=4 echo hi

Will run with FOO=4, because it is the highest precedence.")
            .arg(Arg::new("env")
                .short('e')
                .long("env")
                .value_name("FILE")
                .use_value_delimiter(true)
                .multiple_occurrences(true)
                .help("Use FILE as the reference envfile.")
            )
            .arg(Arg::new("command")
                .required(true)
                .min_values(1)
                .multiple_occurrences(true)
                .help("The command to run. The command can be prefixed with variables in KEY=VALUE format.")
            )
        )
        .arg(Arg::new("verbose")
            .short('v')
            .long("verbose")
            .global(true)
            .takes_value(false)
        )
        ;

    let matches = app.clone().try_get_matches().unwrap_or_else(|_| {
        let mut args = env::args().collect::<Vec<_>>();
        args.insert(1, "add".into());
        app.get_matches_from(args)
    });
    let verbose = matches.is_present("verbose");

    match matches.subcommand().unwrap_or(("add", &matches)) {
        ("add", matches) => {
            let force = matches.is_present("force");
            let pairs = resolve_pairs(matches);
            let reference_env_fpath = resolve_reference_file(matches);

            let other_env_fpaths = if matches.is_present("all") {
                find_all_envfile_paths(&reference_env_fpath)
            } else {
                vec![]
            };

            let mut envfile = EnvFile::read(reference_env_fpath);
            let mut other_envfiles = other_env_fpaths.into_iter().map(EnvFile::read).collect::<Vec<EnvFile>>();

            if !force {
                quit_if_value_exists(&pairs, &envfile, &other_envfiles);
            }

            for pair in pairs {
                envfile.add(&pair.key, &pair.value);
                for other_envfile in &mut other_envfiles {
                    other_envfile.add(&pair.key, "");
                }
            }
        }
        ("check", matches) => {
            let force = matches.is_present("force");
            let reference_env_fpath = resolve_reference_file(matches);

            let other_env_fpaths = matches.values_of("files")
                .map(|values|
                    values
                        .map(|value|
                            PathBuf::from_str(value).unwrap()
                        )
                        .collect()
                )
                .unwrap_or(find_all_envfile_paths(&reference_env_fpath));

            let source_env = EnvFile::read(reference_env_fpath);
            let dest_envs = other_env_fpaths.into_iter().map(EnvFile::read).collect::<Vec<EnvFile>>();
            command::check(source_env, dest_envs, CheckOptions {
                force,
                quiet: false,
            });
        }
        ("init", _) => {
            command::init()
        }
        ("rm", matches) => {
            let pairs = resolve_pairs(matches);
            let reference_env_fpath = resolve_reference_file(matches);

            let other_env_fpaths = if matches.is_present("all") || matches.is_present("all") {
                find_all_envfile_paths(&reference_env_fpath)
            } else {
                vec![]
            };

            let mut envfile = EnvFile::read(reference_env_fpath);
            let mut other_envfiles = other_env_fpaths.into_iter().map(EnvFile::read).collect::<Vec<EnvFile>>();

            for pair in pairs {
                envfile.remove(&pair.key);
                for other_envfile in &mut other_envfiles {
                    other_envfile.remove(&pair.key);
                }
            }
        }
        ("show", matches) => {
            let reference_env_fpath = resolve_reference_file(matches);
            let envfile = EnvFile::read(reference_env_fpath);
            for (key, value) in &envfile {
                println!("{}={}", key, value);
            }
        }
        ("run", matches) => {
            let mut paths = resolve_multiple_reference_files(matches);
            if paths.is_empty() && Path::new(".env").exists() {
                paths.push(PathBuf::from(".env"));
            }
            for path in paths {
                let envfile = EnvFile::read(path);
                for (key, value) in &envfile {
                    env::set_var(key, value);
                }
            }
            let mut command = matches.values_of("command").unwrap();
            let mut maybe_command = command.next().expect("No command provided");
            while maybe_command.contains('=') {
                let pair = maybe_command.splitn(2, '=').collect::<Vec<_>>();
                env::set_var(pair[0], pair[1]);
                maybe_command = command.next().expect("No command provided");
            }
            if verbose {
                for (key, value) in env::vars() {
                    println!("{}={}", key, value);
                }
                println!("{} {}", maybe_command, command.clone().collect::<Vec<_>>().join(" "));
            }
            exit(process::Command::new(maybe_command)
                .args(command)
                .status()
                .expect("Failed to execute command")
                .code().unwrap())
        }
        _ => {
            exit_with("Unrecognized command.");
        }
    }
}
