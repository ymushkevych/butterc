#!/bin/bash

set -e

echo "Compiling compiler using rustc..."
rustc -o butterc ../src/main.rs

echo "Moving compiler into user binaries. May ask for password"
sudo mv ./butterc /usr/bin/
