#!/bin/bash

set -xeuf -o pipefail
sudo chmod -R a+rwx ./
rm -rf cachefiles
mkdir -p cachefiles
touch log.txt
truncate -s 0 log.txt
cargo run --release
