#!/bin/bash

set -e

echo "Removing raw assembly"
rm ./*.asm
rm ./*.wasm #in case someone modifies the source code to produce .wasm files
rm ./*.nasm #in case someone modifies the source code to produce .nasm files
rm ./*.fasm #in case someone modifies the source code to produce .fasm files

echo "Removing object files"
rm ./*.o
rm ./*.out #in case someone modifies the source code to produce .out files
rm ./*.a #in case someone modifies the source code to produce .a files
