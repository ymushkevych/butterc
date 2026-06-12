#!/bin/bash

set -e

echo "Removing raw assembly"
[ -f "$./*.asm" ] && rm ./*.asm
[ -f "$./*.wasm" ] && rm ./*.wasm #in case someone modifies the source code to produce .wasm files
[ -f "$./*.nasm" ] && rm ./*.nasm #in case someone modifies the source code to produce .nasm files
[ -f "$./*.fasm" ] && rm ./*.fasm #in case someone modifies the source code to produce .fasm files

echo "Removing object files"
[ -f "$./*.o" ] && rm ./*.o
[ -f "$./*.out" ] && rm ./*.out #in case someone modifies the source code to produce .out files
[ -f "$./*.a" ] && rm ./*.a #in case someone modifies the source code to produce .a files
