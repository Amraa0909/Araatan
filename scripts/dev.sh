#!/data/data/com.termux/files/usr/bin/bash
# Araatan dev wrapper — Termux/Android-д зориулсан CARGO_HOME/target-dir
set -e
cd "$(dirname "$0")/.."
export CARGO_TARGET_DIR=./target
export CARGO_HOME=./.cargo_home
exec cargo "$@"
