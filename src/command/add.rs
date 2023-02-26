use std::process;
use crate::Cli;
use env2::{EnvFile, Pair};
use crate::resolve_pairs;

#[derive(clap::Parser, Debug)]
pub struct Add {
    #[clap(short, long)]
    pub force: bool,

    #[clap(short, long)]
    pub all: bool,

    pub pairs: Vec<String>,
}

impl Add {
    pub fn run(self, cli: &Cli) {
        let pairs = resolve_pairs(&self.pairs);
        let target = cli.provided_envfile();

        let add_key_envfiles = if self.all {
            cli.fs_envfiles_excluding(&target)
        } else {
            vec![]
        };

        let mut envfile = EnvFile::read(target);
        let mut add_key_envfiles = add_key_envfiles
            .into_iter()
            .map(EnvFile::read)
            .collect::<Vec<EnvFile>>();

        if !self.force {
            quit_if_value_exists(&pairs, &envfile, &add_key_envfiles);
        }

        for pair in pairs {
            envfile.add(&pair.key, &pair.value);
            for envfile in &mut add_key_envfiles {
                envfile.add(&pair.key, "");
            }
        }
        envfile.save_if_modified().unwrap();
        for envfile in &mut add_key_envfiles {
            envfile.save_if_modified().unwrap();
        }
    }
}


fn quit_if_value_exists(pairs: &Vec<Pair>, envfile: &EnvFile, other_envfiles: &Vec<EnvFile>) {
    for pair in pairs {
        if envfile.has_value(&pair.key) {
            eprintln!("{}: Key {} already exists. Use -f to force.", envfile.path.display(), pair.key);
            process::exit(1);
        }
        for env in other_envfiles {
            if env.has_value(&pair.key) {
                eprintln!("{}: Key {} already exists. Use -f to force.", env.path.display(), pair.key);
                process::exit(1);
            }
        }
    }
}
