#!/bin/bash

set -e

echo "Removing raw assembly"
ls *.asm &> /dev/null && rm ./*.asm
ls *.wasm &> /dev/null && rm ./*.wasm #in case someone modifies the source code to produce .wasm files
ls *.nasm &> /dev/null && rm ./*.nasm #in case someone modifies the source code to produce .nasm files
ls *.fasm &> /dev/null && rm ./*.fasm #in case someone modifies the source code to produce .fasm files

echo "Removing object files"
ls *.o &> /dev/null && rm ./*.o
ls *.out &> /dev/null && rm ./*.out #in case someone modifies the source code to produce .out files
ls *.a &> /dev/null && rm ./*.a #in case someone modifies the source code to produce .a files
