use std::path::{PathBuf};
use crate::Cli;
use std::process::exit;
use env2::EnvFile;
use std::str::FromStr;

#[derive(clap::Parser, Debug)]
pub struct Check {
    #[clap(short, long)]
    pub quiet: bool,
    #[clap(short, long)]
    pub force: bool,

    pub files: Vec<String>,
}

#[derive(Debug)]
pub struct CheckOptions {
    pub force: bool,
    pub quiet: bool,
}

impl Check {
    pub fn run(self, cli: &Cli) {
        let force = self.force;
        let envfile = cli.provided_envfile();

        let to_check = if self.files.is_empty() {
            cli.fs_envfiles_excluding(&envfile)
        } else {
            self.files.into_iter().map(|f| PathBuf::from_str(&f).unwrap()).collect()
        };

        let envfile = EnvFile::read(envfile);
        let mut to_check = to_check
            .into_iter()
            .map(EnvFile::read)
            .collect::<Vec<EnvFile>>();
        check(&envfile, &mut to_check, CheckOptions {
            force,
            quiet: false,
        });
    }
}

pub fn check(source_env: &EnvFile, dest_envs: &mut [EnvFile], opts: CheckOptions) {
    if !opts.quiet {
        eprintln!("Using {} as the reference, checking files: {}",
                  source_env.path.display(),
                  dest_envs.iter().map(|e| e.path.display().to_string()).collect::<Vec<String>>().join(" ")
        );
    }

    let mut has_missing = false;
    for dest_env in dest_envs.iter_mut() {
        let missing = missing_keys(&source_env, dest_env);
        for key in missing {
            if !opts.force {
                eprintln!("{}: {} is missing.", dest_env.path.display(), key);
            }
            has_missing = true;
        }
    }

    let mut all_missing: Vec<String> = Vec::new();
    for dest_env in dest_envs.iter_mut() {
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
        for dest_env in dest_envs.iter_mut() {
            dest_env.reorder_based_on(&source_env);
        }
        // I think the exit is causing them not to save because we save on drop? Forcibly drop them Yay!
        for dest_env in dest_envs.iter_mut() {
            dest_env.save_if_modified().unwrap();
        }
    } else if has_missing {
        exit(1)
    }
}

/// Return list of keys in source_env that are missing in dest_env.
pub fn missing_keys(source_env: &EnvFile, dest_env: &EnvFile) -> Vec<String> {
    let mut missing = Vec::new();
    for (key, _) in source_env.iter() {
        if !dest_env.has_key(key) {
            missing.push(key.to_string());
        }
    }
    missing
}
