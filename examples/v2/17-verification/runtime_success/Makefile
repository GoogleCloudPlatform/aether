AETHER_COMPILER ?= ../../../../target/release/aether-compiler
AETHER_RUNTIME_PATH ?= ../../../../runtime/target/release

.PHONY: build run clean

build:
	AETHER_RUNTIME_PATH=$(AETHER_RUNTIME_PATH) $(AETHER_COMPILER) compile main.aes -o main

run: build
	./main

clean:
	rm -f main main.o main.ll
