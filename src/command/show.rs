use crate::Cli;
use env2::EnvFile;

#[derive(clap::Parser, Debug)]
pub struct Show {}

impl Show {
    pub fn run(self, cli: &Cli) {
        let envfile = cli.provided_envfile();
        let envfile = EnvFile::read(envfile);
        for (key, value) in &envfile {
            println!("{}={}", key, value);
        }
    }
}