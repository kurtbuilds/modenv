set dotenv-load := false

help:
    @just --list --unsorted

build:
    cargo build
alias b := build

run *args:
    cargo run {{args}}
alias r := run

release:
    cargo build --release

install:
    cargo install --path .

bootstrap:
    cargo install cargo-bump

test:
    cargo test

check:
    cargo check

version:
    export VERSION=$(rg  "version = \"([0-9.]+)\"" -r '$1' Cargo.toml)
    echo $VERSION

# Bump version. level=major,minor,patch
bump level: check test
    #!/usr/bin/env bash
    git diff-index --exit-code HEAD > /dev/null || (echo You have untracked changes. Commit your changes before bumping the version. && exit 1)
    cargo bump {{level}}
    git commit -am "Bump {{level}} version"
    VERSION = $(rg -o "version = \"0.3.5\"" Cargo.toml)
    echo $VERSION
    git tag v

publish:
    cargo publish