# Prospectus Graphicus — common tasks

default:
    @just --list

build:
    cargo build

build-release:
    cargo build --release

test:
    cargo test --all-features

fmt:
    cargo fmt --all

fmt-check:
    cargo fmt --all -- --check

clippy:
    cargo clippy --all-targets --all-features -- -D warnings

run *ARGS:
    cargo run -- {{ARGS}}

install:
    cargo install --path . --locked

ci: fmt-check clippy test
