#!/bin/sh -e
# Simple installation script for krabby

INSTALL_DIR="/usr/local/bin"

# build binary
cargo build --release

# install binary
cp target/release/krabby $INSTALL_DIR
