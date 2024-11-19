#! /usr/bin/env bash
set -eux

cargo fix --allow-dirty
cargo clippy --fix --allow-dirty
cargo fmt
