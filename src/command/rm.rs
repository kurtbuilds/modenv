use crate::Cli;
use env2::EnvFile;

#[derive(clap::Parser, Debug)]
pub struct Remove {
    #[clap(short, long)]
    pub all: bool,

    /// Keys to remove. Keys can contain =, but the value will be ignored.
    pub keys: Vec<String>,
}

impl Remove {
    pub fn run(self, cli: &Cli) {
        let pairs = crate::resolve_pairs(&self.keys);
        let envfile = cli.provided_envfile();

        let mut envfiles = if self.all {
            cli.fs_envfiles_excluding(&envfile)
        } else {
            vec![]
        };
        envfiles.insert(0, envfile);

        let mut envfiles = envfiles
            .into_iter()
            .map(EnvFile::read)
            .collect::<Vec<EnvFile>>();

        for pair in pairs {
            for other_envfile in &mut envfiles {
                other_envfile.remove(&pair.key);
            }
        }
        for env in envfiles {
            env.save_if_modified().unwrap();
        }
    }
}