# Makefile for the pointers_simple example

AETHER_COMPILER ?= ../../target/release/aether-compiler
EXECUTABLE = pointers_simple
SOURCE = main.aether

.PHONY: all build run clean

all: build

build:
	@echo "Building $(EXECUTABLE)..."
	@if [ ! -f "$(AETHER_COMPILER)" ]; then \
		echo "Compiler not found, trying debug version..."; \
		AETHER_COMPILER=../../target/debug/aether-compiler; \
	fi
	@$(AETHER_COMPILER) compile $(SOURCE) -o $(EXECUTABLE)

run: build
	@echo "Running $(EXECUTABLE)..."
	@./$(EXECUTABLE)

clean:
	@echo "Cleaning up..."
	@rm -f $(EXECUTABLE)

