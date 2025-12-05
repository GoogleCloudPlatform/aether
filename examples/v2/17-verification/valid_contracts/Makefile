AETHER_COMPILER ?= ../../../../target/release/aether-compiler
AETHER_RUNTIME_PATH ?= ../../../../runtime/target/release

.PHONY: build run clean verify

build:
	AETHER_RUNTIME_PATH=$(AETHER_RUNTIME_PATH) $(AETHER_COMPILER) compile main.aes -o main

run: build
	./main

verify:
	AETHER_RUNTIME_PATH=$(AETHER_RUNTIME_PATH) $(AETHER_COMPILER) compile --verify --verbose main.aes -o main

clean:
	rm -f main main.o main.ll
