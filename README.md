Links: [Source](https://github.com/kurtbuilds/modenv) | [crates.io](https://crates.io/crates/modenv)

`modenv` is a tool to update and keep consistent multiple .env files. 
It is designed with these objectives in mind:

- **Simple and intuitive user experience.** `modenv` greatly prioritizes a minimal, intuitive user-interface, helpful error messages 
  and command suggestions to make it easy to use for beginners and experts.
- **Lighting fast.** `modenv` is written in Rust, compiled natively, and therefore extremely fast.
- **Error resistant.** `modenv` offers dry-runs and does not perform destructive operations without explicit confirmation.
- **Composable.** `modenv` uses reasonable error codes, so it can be used effectively in a CI/CD pipeline or build scripts.

# Installation

    cargo install modenv
    
# Quickstart

##### Initialize your environment

    modenv init
    
This command creates .env, env.example, and .env.production files, and adds .env* to your gitignore file.

##### Add to the environment

    modenv -a PORT=3000 HOST=0.0.0.0

This command adds PORT=5000 and HOST=0.0.0.0 to the first default env file found, typically `.env`.
The -a flag causes it to add PORT and HOST with blank values to all other env files found.
If the key already exists, this operation will fail unless -f is also passed.
The default env files are `.env.local`, `.env.development`, and `.env`.
    
##### Add to the production environment

Next, add values to .env.production (specified by -p):

    modenv -p HOST=0.0.0.0 PORT=5000

##### Check consistency of your env files

    modenv check
    
This command checks for missing values from your env files. It uses a single file as the reference file, but you can
specify another file (e.g. `-x` for `.env.example`). This command returns non-zero if there are missing keys, so it can be used
as part of a CI/CD pipeline.
If you want to update files with blank values for missing keys:

    modenv check -f

This command additionally will replicate comments and ordering from the reference file to the other files.