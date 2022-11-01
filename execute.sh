#!/bin/bash

set -xeuf -o pipefail
sudo chmod -R a+rwx ./
rm -rf cachefiles
mkdir -p cachefiles
touch log.txt
cargo run --release
