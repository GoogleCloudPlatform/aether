COMPILER = ../../../../target/debug/aether-compiler
SOURCE = nested_structs.aether
OUTPUT = nested_structs
RUNTIME_PATH = $(shell pwd)/../../../../runtime/target/debug

all: build

build:
	export AETHER_RUNTIME_PATH=$(RUNTIME_PATH) && $(COMPILER) compile $(SOURCE) -o $(OUTPUT)

run: build
	./$(OUTPUT) || (echo "Exit code $$?"; exit 0)

clean:
	rm -f $(OUTPUT) $(OUTPUT).o