use std::path::{Path, PathBuf};
use std::{env, fs, io};

use std::fs::OpenOptions;
use std::io::Write;
use std::process::exit;
use std::str::FromStr;
use crate::file::envfile::{EnvFile, Line};


fn touch(path: &Path) -> io::Result<()> {
    OpenOptions::new().create(true).write(true).open(path).map(|_| eprintln!("{}: Touched file", path.display()))
}


fn append(path: &Path, data: &str) -> io::Result<usize> {
    OpenOptions::new()
        .append(true)
        .open(path)
        .and_then(|mut file| {
            let r = file.write(data.as_bytes());
            eprintln!("{}: Added dotenv rules to gitignore.", path.display());
            r
        })
}

fn path_exists(path: &str) -> bool {
    fs::metadata(Path::new(path)).is_ok()
}

fn find_gitignore() -> PathBuf {
    let mut cur = env::current_dir().unwrap();
    while cur.parent().is_some() {
        if cur.join(".gitignore").is_file() {
            return cur.join(".gitignore");
        } else if cur.join(".git").is_dir() {
            return cur.join(".gitignore");
        } else {
            cur = cur.parent().unwrap().into();
        }
    }
    PathBuf::from_str(".gitignore").unwrap()
}

const DEFAULT_GITIGNORE_RULES: &str = "
.env*
!.env.example
";

pub fn init() {
    let gitignore_path = find_gitignore();
    let gitignore_content = fs::read_to_string(&gitignore_path).unwrap();
    if gitignore_content.contains(".env") {
        // Frameworks with conventions about .env files 
        // (eg Next.js that tracks .env but not .env.local)
        // are expected to set their .gitignore properly in the first place
        eprintln!("{}: .gitignore already contains dotenv rules. Skipping addition of rules.", gitignore_path.display());
    } else {
        append(&find_gitignore(), DEFAULT_GITIGNORE_RULES).unwrap();
    }

    if !path_exists(".env") {
        if path_exists(".env.example") {
            let example = EnvFile::read(PathBuf::from(".env.example"));
            let env = EnvFile {
                modified: false,
                lines: example.lines.clone(),
                path: PathBuf::from(".env"),
            };
            env.save().unwrap();
            eprintln!(".env: Created using values from .env.example.");
        } else {
            touch(Path::new(".env")).unwrap();
        }
        if !path_exists(".env.production") {
            touch(Path::new(".env.production")).unwrap();
            if path_exists(".env.example") {
                check(EnvFile::read(PathBuf::from(".env.example")), vec![
                    EnvFile::read(PathBuf::from(".env.production")),
                ], CheckOptions { force: true, quiet: true });
            }
        }
        if !path_exists(".env.example") {
            touch(Path::new(".env.example")).unwrap();
        }
    }
}

pub fn missing_keys(source_env: &EnvFile, dest_env: &EnvFile) -> Vec<String> {
    let mut missing = Vec::new();
    source_env.lines.iter()
        .filter_map(|line| match line {
            Line::Blank => None,
            Line::Pair(key, _) => Some(key),
            Line::Comment(_) => None,
        })
        .for_each(|key| {
            if !dest_env.has_key(key) {
                missing.push(key.to_string());
            }
        });
    missing
}

pub struct CheckOptions {
    pub quiet: bool,
    pub force: bool,
}

pub fn check(source_env: EnvFile, mut dest_envs: Vec<EnvFile>, opts: CheckOptions) {
    if !opts.quiet {
        eprintln!("Using {} as the reference, checking files: {}",
                  source_env.path.display(),
                  dest_envs.iter().map(|e| e.path.display().to_string()).collect::<Vec<String>>().join(" ")
        );
    }

    let mut has_missing = false;
    for dest_env in &mut dest_envs {
        let missing = missing_keys(&source_env, dest_env);
        for key in missing {
            if !opts.force {
                eprintln!("{}: {} is missing.", dest_env.path.display(), key);
            }
            has_missing = true;
        }
    }

    let mut all_missing: Vec<String> = Vec::new();
    for dest_env in &mut dest_envs {
        let missing = missing_keys(dest_env, &source_env);
        all_missing.extend(missing.iter().cloned());
    }
    if !all_missing.is_empty() {
        all_missing.sort_unstable();
        all_missing.dedup();
        eprintln!("WARNING: {source}: {keys} {verb} missing but exist in other env files. Run this command to keep those values

    modenv add -e {source} {add_args}

Or run this command to remove them from all files

    modenv rm -a {del_args}
", source = source_env.path.display(),
                  keys = all_missing.join(", "),
                  verb = if all_missing.len() == 1 { "is" } else { "are" },
                  add_args = all_missing.iter().map(|s| format!("{}=", s)).collect::<Vec<_>>().join(" "),
                  del_args = all_missing.join(" "));
        exit(1);
    }

    if opts.force {
        for dest_env in &mut dest_envs {
            dest_env.use_ordering_from(&source_env, opts.quiet);
        }
        // I think the exit is causing them not to save because we save on drop? Forcibly drop them Yay!
        drop(dest_envs);
        exit(0);
    } else {
        exit(if has_missing { 1 } else { 0 });
    }
}
