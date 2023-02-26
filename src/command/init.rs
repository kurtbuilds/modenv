use std::path::{Path, PathBuf};
use std::{env, fs, io};
use std::fs::OpenOptions;
use std::io::Write;
use std::str::FromStr;
use crate::command::{CheckOptions, check};
use env2::EnvFile;

pub fn touch(path: &Path) -> io::Result<()> {
    OpenOptions::new().create(true).write(true).open(path).map(|_| eprintln!("{}: Touched file", path.display()))
}


pub fn append(path: &Path, data: &str) -> io::Result<usize> {
    OpenOptions::new()
        .append(true)
        .open(path)
        .and_then(|mut file| {
            let r = file.write(data.as_bytes());
            eprintln!("{}: Added dotenv rules to gitignore.", path.display());
            r
        })
}

pub fn path_exists(path: &str) -> bool {
    fs::metadata(Path::new(path)).is_ok()
}

pub fn find_gitignore() -> PathBuf {
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

pub const DEFAULT_GITIGNORE_RULES: &str = "
.env*
!.env.example
";

#[derive(clap::Parser, Debug)]
pub struct Init {}

impl Init {
    pub fn run(self) {
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
                let env = example.clone_to_path(&Path::new(".env"));
                env.save_if_modified().unwrap();
                eprintln!(".env: Created using values from .env.example.");
            } else {
                touch(Path::new(".env")).unwrap();
            }
            if !path_exists(".env.production") {
                touch(Path::new(".env.production")).unwrap();
                if path_exists(".env.example") {
                    let source = EnvFile::read(PathBuf::from(".env.example"));
                    let mut dest = vec![
                        EnvFile::read(PathBuf::from(".env.production")),
                    ];
                    check(&source, &mut dest, CheckOptions { force: true, quiet: true });
                }
            }
            if !path_exists(".env.example") {
                touch(Path::new(".env.example")).unwrap();
            }
        }
    }
}

