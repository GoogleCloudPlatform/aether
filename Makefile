# Makefile for AetherScript

# Environment variables
export Z3_SYS_Z3_HEADER=/opt/homebrew/include/z3.h

.PHONY: all build test clean check fmt stdlib

all: build stdlib

build:
	cargo build

stdlib:
	./target/debug/aether-compiler check stdlib/arrays.aether

test:
	cargo test

clean:
	cargo clean

check:
	cargo clippy

fmt:
	cargo fmt
