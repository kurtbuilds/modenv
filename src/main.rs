use std::path::PathBuf;
use std::process::exit;
use std::str::FromStr;

use clap::{App, AppSettings, Arg, ArgMatches, ArgSettings};

use crate::command::{missing_keys};
use crate::envfile::{EnvFile, Line, Pair};

mod detection;
mod add;
mod audit;
mod command;
mod envfile;

const VERSION: &'static str = env!("CARGO_PKG_VERSION");

fn exit_with(message: &str) -> ! {
    eprintln!("{}", message);
    exit(1);
}

fn find_local_env() -> PathBuf {
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

fn add_hidden(arg: Arg, hidden: bool) -> Arg {
    if hidden {
        arg.setting(ArgSettings::Hidden)
    } else {
        arg
    }
}

fn add_source_file_args(mut app: App, hidden: bool, add_all: bool) -> App {
    let mut settings = Vec::new();
    if hidden {
        settings.push(ArgSettings::Hidden);
    }
    app = app
        .arg(add_hidden(Arg::new("production")
            .short('p')
            .long("production")
            .about("Use .env.production.")
        , hidden))
        .arg(add_hidden(Arg::new("development")
            .short('d')
            .long("development")
            .about("Use .env.development.")
         , hidden))
        .arg(add_hidden(Arg::new("normal")
            .short('n')
            .long("normal")
            .about("Use .env.")
        , hidden))
        .arg(add_hidden(Arg::new("example")
            .short('x')
            .long("example")
            .about("Use .env.example.")
        , hidden))
        .arg(add_hidden(Arg::new("force")
            .short('f')
            .long("force")
            .about("Overwrite value if already exists.")
        , hidden))
        .arg(add_hidden(Arg::new("env")
            .short('e')
            .long("env")
            .value_name("FILE")
            .about("Use FILE as the envfile.")
        , hidden));
    if add_all {
        app = app
            .arg(add_hidden(Arg::new("all")
                .short('a')
                .long("all")
                .about("Add provided key (leaving value blank) to all other .env files, keeping them consistent.")
            , hidden))
    }
    app
}

fn resolve_file(com_match: &ArgMatches, sub_match: &ArgMatches, default: PathBuf) -> PathBuf {
    if com_match.is_present("production") || sub_match.is_present("production") {
        PathBuf::from_str(".env.production").unwrap()
    } else if com_match.is_present("development") || sub_match.is_present("development") {
        PathBuf::from_str(".env.development").unwrap()
    } else if com_match.is_present("normal") || sub_match.is_present("normal") {
        PathBuf::from_str(".env").unwrap()
    } else if com_match.is_present("example") || sub_match.is_present("example") {
        PathBuf::from_str(".env.example").unwrap()
    } else if com_match.value_of("env").is_some() {
        PathBuf::from_str(com_match.value_of("env").unwrap()).unwrap()
    } else if sub_match.value_of("env").is_some() {
        PathBuf::from_str(sub_match.value_of("env").unwrap()).unwrap()
    } else {
        default
    }
}

fn find_all_envfile_paths(ignore: &PathBuf) -> Vec<PathBuf> {
    let mut v: Vec<PathBuf> = vec![
        ".env.local",
        ".env.development",
        ".env",
    ].into_iter()
        .filter(|p| !p.eq(&ignore.to_str().unwrap()))
        .filter_map(|p| PathBuf::from_str(&p).ok()
            .filter(|p| p.exists()))
        .take(1)
        .collect();
    v.extend(vec![
        ".env.example",
        ".env.production",
    ].into_iter()
        .filter(|p| !p.eq(&ignore.to_str().unwrap()))
        .filter_map(|p| PathBuf::from_str(&p).ok()
            .filter(|p| p.exists()))
    );
    v
}

fn quit_if_exists(pairs: &Vec<Pair>, envfile: &EnvFile, other_envfiles: &Vec<EnvFile>) {
    for pair in pairs {
        if envfile.contains(&pair.key) {
            exit_with(&format!("Key {} already exists in {}. Use -f to force.", pair.key, envfile.path.display()));
        }
        for env in other_envfiles {
            if env.contains(&pair.key) {
                exit_with(&format!("Key {} already exists in {}. Use -f to force.", pair.key, env.path.display()));
            }
        }
    }
}

fn choose_default_envfile(files: &Vec<PathBuf>) -> PathBuf {
    let example_envfile = PathBuf::from_str(".env.example").unwrap();
    if files.contains(&example_envfile) {
        find_local_env()
    } else {
        example_envfile
    }
}

fn main() {
    let com_match = add_source_file_args(App::new("modenv"), true, true)
        .version(VERSION)
        .about("Manage environment files.

By default, modenv will use the add command.")
        .setting(AppSettings::ArgRequiredElseHelp)
        .arg(Arg::new("pairs").setting(ArgSettings::Hidden).min_values(0))
        .subcommand(add_source_file_args(App::new("add"), false, true)
            .arg(Arg::new("pairs").required(true).min_values(1))
            .about("Add an environment variable.")
        )
        .subcommand(add_source_file_args(App::new("audit"), false, false)
            .arg(Arg::new("files").min_values(0))
            .about("Show keys missing from the given files.")
        )
        .subcommand(add_source_file_args(App::new("order"), false, false)
            .arg(Arg::new("files").min_values(0))
            .about("Order the given files.")
        )
        .subcommand(App::new("init")
            .about("Create .env, .env.example, and .env.production, and add .env to .gitignore.")
        )
        .get_matches();

    match com_match.subcommand().unwrap_or(("add", &com_match)) {
        ("add", sub_match) => {
            let force = com_match.is_present("force") || sub_match.is_present("force");
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
            let env_fpath = resolve_file(&com_match, sub_match, find_local_env());

            let other_env_fpaths = if com_match.is_present("all") || sub_match.is_present("all") {
                find_all_envfile_paths(&env_fpath)
            } else {
                vec![]
            };

            let mut envfile = EnvFile::read(env_fpath);
            let mut other_envfiles = other_env_fpaths.into_iter().map(EnvFile::read).collect::<Vec<EnvFile>>();

            if !force {
                quit_if_exists(&pairs, &envfile, &other_envfiles);
            }

            for pair in pairs {
                envfile.add(&pair.key, &pair.value);
                for other_envfile in &mut other_envfiles {
                    other_envfile.add(&pair.key, "");
                }
            }
        }
        ("audit", sub_match) => {
            let force = com_match.is_present("force") || sub_match.is_present("force");
            let dest_files = sub_match.values_of("files")
                .map(|values|
                    values
                        .map(|value|
                            PathBuf::from_str(value).unwrap()
                        )
                        .collect()
                )
                .unwrap_or(find_all_envfile_paths(&PathBuf::from_str(".env.example").unwrap()));
            let source_file = resolve_file(&com_match, &sub_match, choose_default_envfile(&dest_files));

            let source_env = EnvFile::read(source_file);
            let mut dest_envs = dest_files.into_iter().map(EnvFile::read).collect::<Vec<EnvFile>>();

            command::audit(source_env, dest_envs, force);
        }
        ("order", sub_match) => {
            let force = com_match.is_present("force") || sub_match.is_present("force");
            let dest_files = sub_match.values_of("files")
                .map(|values|
                    values
                        .map(|value|
                            PathBuf::from_str(value).unwrap()
                        )
                        .collect()
                )
                .unwrap_or(find_all_envfile_paths(&PathBuf::from_str(".env.example").unwrap()));
            let source_file = resolve_file(&com_match, &sub_match, choose_default_envfile(&dest_files));

            let source_env = EnvFile::read(source_file);
            let mut dest_envs = dest_files.into_iter().map(EnvFile::read).collect::<Vec<EnvFile>>();

            command::order(source_env, dest_envs, force);
        }
        ("init", sub_match) => {
            command::init()
        }
        _ => {
            exit_with("Unrecognized command.");
        }
    }
}
