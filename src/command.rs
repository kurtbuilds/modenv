use std::path::{Path, PathBuf};
use std::{io, env};
use std::fs::OpenOptions;
use std::io::Write;
use std::str::FromStr;
use crate::envfile::{EnvFile, Line};
use crate::exit_with;
// use crate::envfile::read_envfile;

fn touch(path: &Path) -> io::Result<()> {
    OpenOptions::new().create(true).write(true).open(path).map(|_| eprintln!("Touched {}", path.display()))
}


fn append(path: &Path, data: &str) -> io::Result<()> {
    OpenOptions::new()
        .append(true)
        .open(path)
        .map(|mut file| {
            file.write(data.as_bytes());
            eprintln!("Appended to {}", path.display())
        })
}

fn find_gitignore() -> PathBuf {
    let mut cur = env::current_dir().unwrap();
    println!("{}", cur.display());
    while cur.parent().is_some() {
        if cur.join(".gitignore").is_file() {
            println!(".gitig file");
            return cur.join(".gitignore");
        } else if cur.join(".git").is_dir() {
            println!(".git dir");
            return cur.join(".gitignore");
        } else {
            cur = cur.parent().unwrap().into();
            println!("parent");
        }
    }
    println!("curdir");
    PathBuf::from_str(".gitignore").unwrap()
}

pub fn init() {
    touch(&Path::new(".env")).unwrap();
    touch(&Path::new(".env.example")).unwrap();
    touch(&Path::new(".env.production")).unwrap();

    let gitignore_fpath = find_gitignore();
    println!(" ignore {}", gitignore_fpath.display());

    append(&gitignore_fpath, "
.env*
!.env.example
").unwrap()
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
            if !dest_env.contains(key) {
                missing.push(key.to_string());
            }
        });
    return missing;
}

pub fn audit(source_env: EnvFile, mut dest_envs: Vec<EnvFile>, force: bool) {
    eprintln!("Using {} as the reference, checking these files: {}\n",
              source_env.path.display(),
              dest_envs.iter().map(|e| e.path.display().to_string()).collect::<Vec<String>>().join(" ")
    );
    let mut consistent = true;
    for dest_env in &mut dest_envs {
        let missing = missing_keys(&source_env, dest_env);
        missing.iter().for_each(|key| {
            println!("Key {} is missing from {}", key, dest_env.path.display());
            if force {
                dest_env.add(&key, "");
            }
        });
        if missing.len() > 0 {
            consistent = false;
        }
    }
    if consistent {
        eprintln!("No missing keys were found. Your environment is consistent.")
    }
}


pub fn order(source_env: EnvFile, mut dest_envs: Vec<EnvFile>, force: bool) {
    eprintln!("Using {} as the reference, reorder these files: {}\n",
              source_env.path.display(),
              dest_envs.iter().map(|e| e.path.display().to_string()).collect::<Vec<String>>().join(" ")
    );
    if !force {
        // do the audit.
        let mut consistent = true;
        for dest_env in &mut dest_envs {
            let missing = missing_keys(&dest_env, &source_env);
            if missing.len() > 0 {
                missing.iter().for_each(|key| {
                    consistent = false;
                    println!("Key {} is missing from {}", key, source_env.path.display());
                });
                println!("
Order would destroy this data from {dest}. Run this command to keep all keys in {dest}:

    modenv audit -fe {dest} {src}
", dest = dest_env.path.display(), src = source_env.path.display());
            }
        }
        if consistent {
            exit_with("Order will remove any comments from non-reference files. Re-run with -f to continue.")
        } else {
            exit_with("If you're okay losing these values, re-run with -f to continue.")
        }
    } else {
        for dest_env in &mut dest_envs {
            dest_env.use_ordering_from(&source_env);
        }
    }
}