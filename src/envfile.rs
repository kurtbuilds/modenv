use std::{fs, io};
use std::path::PathBuf;
use crate::exit_with;

#[derive(Clone, Debug)]
pub struct Pair {
    pub(crate) key: String,
    pub(crate) value: String,
}

pub enum Line {
    Blank,
    Pair(String, String),
    Comment(String),
}

pub struct EnvFile {
    pub(crate) lines: Vec<Line>,
    pub(crate) path: PathBuf,
}


impl EnvFile {
    pub fn read(path: PathBuf) -> Self {
        let s = fs::read_to_string(&path).unwrap();
        // println!("DEBUG: {}", path.display());
        EnvFile {
            lines: s.split("\n")
                .map(|line| {
                    let line = line.trim();
                    if line.starts_with("#") {
                        Line::Comment(line.into())
                    } else if line.is_empty() {
                        Line::Blank
                    } else {
                        let mut split = line.splitn(2, "=");
                        let pair = (split.next().unwrap().into(), split.next().unwrap().into());
                        // println!("DEBUG: {}={}", pair.0, pair.1);
                        Line::Pair(pair.0, pair.1)
                    }
                })
                .collect(),
            path,
        }
    }

    pub fn has_value(&self, key: &str) -> bool {
        self.lines.iter().any(|p| match p {
            Line::Blank => false,
            Line::Pair(k, v) => k == key && !v.is_empty(),
            Line::Comment(_) => false,
        })
    }

    pub fn has_key(&self, key: &str) -> bool {
        self.lines.iter().any(|p| match p {
            Line::Blank => false,
            Line::Pair(k, v) => k == key,
            Line::Comment(_) => false,
        })
    }

    pub fn lookup(&self, key: &str) -> Option<String> {
        self.lines.iter().find_map(|p| match p {
            Line::Blank => None,
            Line::Pair(k, value) => if k == key {
                Some(value.to_string())
            } else {
                None
            },
            Line::Comment(_) => None,
        })
    }

    pub fn add(&mut self, key: &str, value: &str) {
        for line in &mut self.lines {
            match line {
                Line::Blank => {}
                Line::Pair(k, existing_value) => {
                    if key == k {
                        if value == existing_value {
                            eprintln!("Key {} already contains this value in {}", &key, self.path.display());
                            return
                        }
                        *line = Line::Pair(key.to_string(), value.to_string());
                        eprintln!("Updated value for {} in {}", &key, self.path.display());
                        return
                    }
                }
                Line::Comment(_) => {}
            }
        }
        self.lines.push(Line::Pair(key.into(), value.into()));
        eprintln!("Added key {}{} to {}", key, if value == "" { " with blank value" } else {""}, self.path.display());
    }

    pub fn save(&mut self) -> io::Result<()> {
        fs::write(&self.path, self.lines
            .iter()
            .map(|line| match line {
                Line::Blank => String::new(),
                Line::Pair(key, value) => format!("{}={}", key, value),
                Line::Comment(line) => line.to_string(),
            })
            .collect::<Vec<String>>()
            .join("\n")
        )
    }

    pub fn use_ordering_from(&mut self, envfile: &EnvFile) {
        let newlines = envfile.lines.iter()
            .map(|line| match line {
                Line::Blank => Line::Blank,
                Line::Pair(key, _) => {
                    let value = self.lookup(&key);
                    if value.is_none() {
                        eprintln!("Added key {} with blank value to {}", key, self.path.display());
                    }
                    Line::Pair(key.to_string(), value.unwrap_or("".to_string()))
                }
                Line::Comment(com) => Line::Comment(com.to_string()),
            })
            .collect();
        self.lines = newlines;
    }
}


impl Drop for EnvFile {
    fn drop(&mut self) {
        self.save().unwrap()
    }
}