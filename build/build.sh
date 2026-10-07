#!/bin/bash

set -e

echo "Checking for dependencies"
bash ./depend.sh

echo "Compiling compiler using rustc..."
rustc -o butterc ../src/main.rs

echo "Moving compiler into user binaries"
sudo mv ./butterc /usr/bin/
