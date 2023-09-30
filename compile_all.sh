#!/bin/bash

# apple silicon binary
cargo b --release --target aarch64-apple-darwin
# apple intel binary
cargo b --release --target x86_64-apple-darwin
# windows binary intel 64bit
cargo b --release --target x86_64-pc-windows-gnu
# linux binary aarch64
cargo b --release --target aarch64-unknown-linux-gnu
# linux binary intel 64bit
cargo b --release --target x86_64-unknown-linux-gnu

# remove existing files
rm -rf dist
# make the folder again
mkdir -p dist

# copy files to the dist folder
# win
cp target/x86_64-pc-windows-gnu/release/csv_rec.exe dist/csv_rec_x86-64.exe
# macos
cp target/aarch64-apple-darwin/release/csv_rec dist/csv_rec_macos_aarch64
cp target/x86_64-apple-darwin/release/csv_rec dist/csv_rec_macos_x86-64
# linux
cp target/aarch64-unknown-linux-gnu/release/csv_rec dist/csv_rec_linux_aarch64
cp target/x86_64-unknown-linux-gnu/release/csv_rec dist/csv_rec_linux_x86-64

