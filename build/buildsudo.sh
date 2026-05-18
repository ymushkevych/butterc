#!/bin/bash

set -xe

echo "Compiling compiler using rustc..."
rustc -o butterc ../src/main.rs

echo "Moving compiler into user binaries. May ask for password"
sudo mv ./butterc /usr/bin/
