#!/usr/bin/env bash
set -e

if [[ ! -z $CI ]]; then
    export CARGO_HUSKY_DONT_INSTALL_HOOKS=true
fi

cargo fmt -- --check
cargo clippy --all-targets --all-features -- -D warnings
cargo check --frozen
cargo test --all-targets --all-features
cargo test --all-targets --all-features -- --ignored
