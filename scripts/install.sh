#!/usr/bin/env sh

cargo build --release
mkdir -p ~/.sea/bin/
cp ./target/release/sea ~/.sea/bin/
cp -r std ~/.sea/
