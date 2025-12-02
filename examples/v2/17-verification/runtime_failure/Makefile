AETHER_COMPILER ?= ../../../../target/release/aether-compiler
AETHER_RUNTIME_PATH ?= ../../../../runtime/target/release

.PHONY: build run clean

build:
	AETHER_RUNTIME_PATH=$(AETHER_RUNTIME_PATH) $(AETHER_COMPILER) compile main.aes -o main

# Running this will show assertion failure and abort
run: build
	./main || echo "Program aborted as expected (contract violation)"

clean:
	rm -f main main.o main.ll
