<p align="center">
<a href="https://github.com/kurtbuilds/modenv/graphs/contributors">
    <img src="https://img.shields.io/github/contributors/kurtbuilds/modenv.svg?style=flat-square" alt="GitHub Contributors" />
</a>
<a href="https://github.com/kurtbuilds/modenv/stargazers">
    <img src="https://img.shields.io/github/stars/kurtbuilds/modenv.svg?style=flat-square" alt="Stars" />
</a>
<a href="https://github.com/kurtbuilds/modenv/actions">
    <img src="https://img.shields.io/github/actions/workflow/status/kurtbuilds/modenv/test.yaml?branch=master&style=flat-square" alt="Build Status" />
</a>
<a href="https://crates.io/crates/modenv">
    <img src="https://img.shields.io/crates/d/modenv?style=flat-square" alt="Downloads" />
</a>
<a href="https://crates.io/crates/modenv">
    <img src="https://img.shields.io/crates/v/modenv?style=flat-square" alt="Crates.io" />
</a>
</p>

# Modenv

`modenv` is a tool to update and keep consistent multiple `.env` files.
It is designed with these objectives in mind:

- **Simple and intuitive user experience.** `modenv` greatly prioritizes a minimal, intuitive user-interface, helpful error messages
  and command suggestions to make it easy to use for beginners and experts.
- **Lighting fast.** `modenv` is written in Rust, compiled natively, and therefore extremely fast.
- **Error resistant.** `modenv` offers dry-runs and does not perform destructive operations without explicit confirmation.
- **Composable.** `modenv` uses reasonable error codes, so it can be used effectively in a CI/CD pipeline or build scripts.

# Usage

It's easiest to understand the usage of `modenv` by following the lifecycle of a typical project.

#### Initialize the environment

    modenv init

This command creates `.env`, `env.example`, and `.env.production` files, and adds `.env*` to your `.gitignore` file 
(still allowing `.env.example`).

#### Add to the environment

    modenv -a PORT=3000 HOST=0.0.0.0

This command adds `PORT=5000` and `HOST=0.0.0.0` to the first default env file found, typically `.env`.
The `-a` flag causes it to add `PORT` and `HOST` with blank values to all other env files found.
If the key already exists, this operation will fail unless `-f` is also passed.
The default env file is the first found of `.env.local`, `.env.development`, and `.env`.

#### Add to the production environment

Next, add values to `.env.production` (specified by `-p`):

    modenv -p PORT=5000 HOST=0.0.0.0

#### Check consistency of your env files

    modenv check

This subcommand checks for missing values from your env files. Using a reference file (chosen implicitly, as described above, or 
explicitly with a command flag), `check` reports on missing keys.

This command returns non-zero if there are missing keys, so it can be used as part of a CI/CD pipeline.

If you want to update files with blank values for missing keys:

    modenv check -f

This command additionally will replicate comments and ordering from the reference file to the other files.

# Installation

    cargo install modenv

# Additional Tips & Tricks

#### Source an env file

    export $(modenv show)

# Contributing

Contributions are what make the open source community such an amazing place to learn, inspire, and create. Any contributions you make are **greatly appreciated**.

If you have a suggestion that would make this better, please fork the repo and create a pull request. You can also simply open an issue with the tag "enhancement".
Don't forget to give the project a star! Thanks again!

1. Fork the Project
2. Create your Feature Branch (`git checkout -b feature/AmazingFeature`)
3. Commit your Changes (`git commit -m 'Add some AmazingFeature'`)
4. Push to the Branch (`git push origin feature/AmazingFeature`)
5. Open a Pull Request
