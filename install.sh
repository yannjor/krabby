#!/bin/sh -e
# Install script for krabby

INSTALL_DIR="/usr/local/opt"
BIN_DIR="/usr/local/bin"

# deleting directory if it already exists
rm -rf $INSTALL_DIR/krabby || return 1

# making the necessary folder structure
mkdir -p $INSTALL_DIR/krabby || return 1

# build binary
cargo build --release

# moving all the files to appropriate locations
cp -rf colorscripts $INSTALL_DIR/krabby
cp target/release/krabby $INSTALL_DIR/krabby
cp pokemon.json $INSTALL_DIR/krabby

# create symlink in usr/bin
rm -rf $BIN_DIR/krabby || return 1
ln -s $INSTALL_DIR/krabby/krabby $BIN_DIR/krabby
