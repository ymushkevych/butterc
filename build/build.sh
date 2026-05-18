#!/bin/bash

set -xe

echo "Compiling compiler using rustc"
rustc -o butterc ../src/main.rs
