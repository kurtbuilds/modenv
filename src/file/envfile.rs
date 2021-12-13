use std::{fs, io};
use std::path::PathBuf;

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
            lines: s.split('\n')
                .map(|line| {
                    let line = line.trim();
                    if line.starts_with('#') {
                        Line::Comment(line.into())
                    } else if line.is_empty() {
                        Line::Blank
                    } else {
                        let pair = line.splitn(2, '=').collect::<Vec<_>>();
                        if pair[1].starts_with('"') {
                            Line::Pair(
                                pair[0].to_string(),
                                pair[1][1..pair[1].len() - 1].to_string()
                            )
                        } else {
                            Line::Pair(
                                pair[0].to_string(),
                                pair[1].to_string()
                            )
                        }
                    }
                })
                .collect(),
            path,
        }
    }

    pub fn remove(&mut self, key: &str) {
        let path = self.path.display();
        self.lines.retain(|line| {
            match line {
                Line::Pair(k, _) => {
                    if key == k {
                        eprintln!("{}: Removed {}", path, k);
                    }
                    k != key
                },
                _ => true,
            }
        })
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
            Line::Pair(k, _) => k == key,
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
                            return
                        } else if value.is_empty() && !existing_value.is_empty() {
                            eprintln!("{}: {} already exists", self.path.display(), key);
                            return
                        } else {
                            *line = Line::Pair(key.to_string(), value.to_string());
                            eprintln!("{}: Updated {}={}", self.path.display(), key, value);
                            return
                        }
                    }
                }
                Line::Comment(_) => {}
            }
        }
        self.lines.push(Line::Pair(key.into(), value.into()));
        eprintln!("{}: Added {}={}", self.path.display(), key, value);
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
                    let value = self.lookup(key);
                    if value.is_none() {
                        eprintln!("{}: Added {}=", self.path.display(), key);
                    }
                    Line::Pair(key.to_string(), value.unwrap_or("".to_string()))
                }
                Line::Comment(com) => Line::Comment(com.to_string()),
            })
            .collect();
        self.lines = newlines;
    }
}


impl<'a> IntoIterator for &'a EnvFile {
    type Item = (&'a String, &'a String);
    type IntoIter = EnvIter<'a>;

    fn into_iter(self) -> Self::IntoIter {
        EnvIter {
            env: self,
            i: 0,
        }
    }
}


pub struct EnvIter<'a> {
    env: &'a EnvFile,
    i: usize,
}


impl<'a> Iterator for EnvIter<'a> {
    type Item = (&'a String, &'a String);

    fn next(&mut self) -> Option<Self::Item> {
        while self.i < self.env.lines.len() {
            let x = unsafe { self.env.lines.get_unchecked(self.i) };
            self.i += 1;
            match x {
                Line::Blank => {}
                Line::Pair(k, v) => return Some((k, v)),
                Line::Comment(_) => {}
            }
        }
        None
    }
}


impl Drop for EnvFile {
    fn drop(&mut self) {
        self.save().unwrap()
    }
}
