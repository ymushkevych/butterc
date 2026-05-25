#!/bin/bash

set -e

if ! command -v rustc &> /dev/null; then
	echo "Installing rustc"
	curl --proto '=https' --tlsv1.2 https://sh.rustup.rs -sSf | sh
	rustup update
fi

if ! command -v nasm &> /dev/null; then
	echo "installing nasm"
	sudo apt install nasm
fi

if ! command -v ld &> /dev/null; then
	echo "installing ld"
	echo "... and other binary utils"
	sud apt install binutils
fi

