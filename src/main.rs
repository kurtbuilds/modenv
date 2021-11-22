use std::{env, fs};
use std::path::PathBuf;
use std::process::exit;
use std::str::FromStr;

use clap::{App, AppSettings, Arg, ArgMatches, ArgSettings};

use crate::command::missing_keys;
use crate::file::{EnvFile, Pair};

mod command;
mod file;

const VERSION: &'static str = env!("CARGO_PKG_VERSION");

fn exit_with(message: &str) -> ! {
    eprintln!("{}", message);
    exit(1);
}

fn default_reference_envfile() -> PathBuf {
    for path in vec![
        ".env.local",
        ".env.development",
        ".env",
    ] {
        let p = PathBuf::from_str(path).unwrap();
        if p.exists() {
            return p;
        }
    }
    exit_with("Could not find .env.local, .env.development, or .env file.")
}

fn add_reference_file_args(app: App) -> App {
    app
        .arg(Arg::new("production")
            .short('p')
            .long("production")
            .about("Use .env.production as the reference file.")
        )
        .arg(Arg::new("development")
            .short('d')
            .long("development")
            .about("Use .env.development as the reference file.")
        )
        .arg(Arg::new("normal")
            .short('n')
            .long("normal")
            .about("Use .env as the reference file.")
        )
        .arg(Arg::new("example")
            .short('x')
            .long("example")
            .about("Use .env.example.")
        )
        .arg(Arg::new("env")
            .short('e')
            .long("env")
            .value_name("FILE")
            .about("Use FILE as the reference envfile.")
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

fn choose_default_envfile(files: &Vec<PathBuf>) -> PathBuf {
    let example_envfile = PathBuf::from_str(".env.example").unwrap();
    if files.contains(&example_envfile) {
        default_reference_envfile()
    } else {
        example_envfile
    }
}


fn resolve_pairs(sub_match: &ArgMatches) -> Vec<Pair> {
    let mut input = sub_match.values_of("pairs").unwrap().peekable();
    let mut pairs = Vec::new();
    while input.peek().is_some() {
        let inp = input.next().unwrap();
        if inp.contains("=") {
            let mut split = inp.splitn(2, "=");
            pairs.push(Pair { key: split.next().unwrap().into(), value: split.next().unwrap().into() });
        } else {
            pairs.push(Pair { key: inp.into(), value: input.next().unwrap().into() });
        }
    }
    pairs
}


fn main() {
    let mut args = env::args_os().collect::<Vec<_>>();
    if args.len() > 1 && !vec!["add", "check", "init"].contains(&args[1].to_str().unwrap()) {
        args.insert(1, "add".into())
    }
    let com_match = App::new("modenv")
        .version(VERSION)
        .about("Manage environment files and keep them consistent.

        If you do not provide a subcommand, modenv uses the add command.

        Example workflow:

            modenv init

            # Add KEY=VALUE to .env. -a adds the KEY, but not the value, to all other .env files.
            # The add command is assumed if no subcommand is provided, so it is optional.

            modenv -a KEY=VALUE
            modenv add -a KEY=VALUE

            # Once you have the production value, add KEY=PROD_VALUE to .env.production.
            modenv -p KEY=PROD_VALUE

            # Check keys in other envfiles based on .env. This is a dry-run. Use -f to perform
            # changes.
            modenv check")
        .setting(AppSettings::ArgRequiredElseHelp)
        .subcommand(add_reference_file_args(App::new("add"))
            .arg(Arg::new("force")
                .short('f')
                .long("force")
                .takes_value(false)
                .about("Overwrite key value if it already exists.")
            )
            .arg(Arg::new("all")
                .short('a')
                .long("all")
                .takes_value(false)
            )
            .arg(Arg::new("pairs").required(true).min_values(1))
            .about("Add environment variable. Can be passed as KEY=VALUE or as two args, KEY VALUE.")
        )
        .subcommand(add_reference_file_args(App::new("check"))
            .arg(Arg::new("FILES")
                .min_values(0)
                .about("By default, modenv scans all env files. If you specify FILES, it only checks those files.")
            )
            .arg(Arg::new("force")
                .short('f')
                .long("force")
                .about("Update both the reference file and other env files to have the same ordering, set of keys, and comments.")
            )
            .about("Show missing keys.")
        ).subcommand(App::new("init")
            .about("Create .env, .env.example, and .env.production, and add .env to .gitignore.")
        )
        .get_matches_from(args.into_iter());

    match com_match.subcommand().unwrap_or(("add", &com_match)) {
        ("add", sub_match) => {
            let force = sub_match.is_present("force");
            let pairs = resolve_pairs(sub_match);
            let reference_env_fpath = resolve_reference_file(sub_match);

            let other_env_fpaths = if com_match.is_present("all") || sub_match.is_present("all") {
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
        ("check", sub_match) => {
            let force = sub_match.is_present("force");
            let reference_env_fpath = resolve_reference_file(sub_match);

            let other_env_fpaths = sub_match.values_of("files")
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
            command::check(source_env, dest_envs, force);
        }
        ("init", _) => {
            command::init()
        }
        ("rm", sub_match) => {
            let pairs = resolve_pairs(sub_match);
            let reference_env_fpath = resolve_reference_file(sub_match);

            let other_env_fpaths = if com_match.is_present("all") || sub_match.is_present("all") {
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
        _ => {
            exit_with("Unrecognized command.");
        }
    }
}
