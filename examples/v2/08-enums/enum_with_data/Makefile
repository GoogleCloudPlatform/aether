COMPILER = ../../../../target/debug/aether-compiler
SOURCE = enum_with_data.aether
OUTPUT = enum_with_data
RUNTIME_PATH = $(shell pwd)/../../../../runtime/target/debug

all: build

build:
	export AETHER_RUNTIME_PATH=$(RUNTIME_PATH) && $(COMPILER) compile $(SOURCE) -o $(OUTPUT)

run: build
	./$(OUTPUT) || (echo "Exit code $$?"; exit 0)

clean:
	rm -f $(OUTPUT) $(OUTPUT).o