#!/bin/bash
set -e

cargo fmt --all
cargo clippy --fix
source ./scripts/run_in_subprojects.sh ./contracts/token-contracts/cw-hub-bnusd ./contracts/core-contracts/cw-asset-manager
cargo clean
