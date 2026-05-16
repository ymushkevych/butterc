#!/bin/bash

set -e

echo "Compiling compiler using rustc"
rustc -o butterc ../src/main.rs
