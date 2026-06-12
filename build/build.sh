#!/bin/bash

set -e

echo "installing dependencies"
bash ./depend.sh

echo "Compiling compiler using rustc..."
rustc -o butterc ../src/main.rs

echo "Moving compiler into user binaries. May ask for password"
sudo mv ./butterc /usr/bin/
