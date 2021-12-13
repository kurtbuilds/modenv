use std::path::{Path, PathBuf};
use std::{env, fs, io};
use std::collections::HashMap;
use std::fs::OpenOptions;
use std::io::Write;
use std::process::exit;
use std::str::FromStr;
use crate::file::envfile::{EnvFile, Line};
use crate::exit_with;

fn touch(path: &Path) -> io::Result<()> {
    OpenOptions::new().create(true).write(true).open(path).map(|_| eprintln!("{}: Touched file", path.display()))
}


fn append(path: &Path, data: &str) -> io::Result<()> {
    OpenOptions::new()
        .append(true)
        .open(path)
        .map(|mut file| {
            file.write(data.as_bytes());
            eprintln!("{}: Added dotenv rules to gitignore.", path.display())
        })
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

pub fn init() {
    append(&find_gitignore(), "
.env*
!.env.example
").unwrap();

    if fs::metadata(Path::new(".env.example")).is_ok() {
        touch(&Path::new(".env")).unwrap();
        touch(&Path::new(".env.production")).unwrap();
        check(EnvFile::read(PathBuf::from(".env.example")), vec![
            EnvFile::read(PathBuf::from(".env")),
            EnvFile::read(PathBuf::from(".env.production")),
        ], true);
    } else {
        touch(&Path::new(".env")).unwrap();
        touch(&Path::new(".env.example")).unwrap();
        touch(&Path::new(".env.production")).unwrap();
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
    return missing;
}

pub fn check(source_env: EnvFile, mut dest_envs: Vec<EnvFile>, force: bool) {
    eprintln!("Using {} as the reference, checking files: {}",
              source_env.path.display(),
              dest_envs.iter().map(|e| e.path.display().to_string()).collect::<Vec<String>>().join(" ")
    );

    let mut has_missing = false;
    for dest_env in &mut dest_envs {
        let missing = missing_keys(&source_env, dest_env);
        for key in missing {
            if !force {
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

    if force {
        for dest_env in &mut dest_envs {
            dest_env.use_ordering_from(&source_env);
        }
        // I think the exit is causing them not to save because we save on drop? Forcibly drop them Yay!
        drop(dest_envs);
        exit(0);
    } else {
        exit(if has_missing { 1 } else { 0 });
    }
}


pub fn show() {

}

pub fn run() {}