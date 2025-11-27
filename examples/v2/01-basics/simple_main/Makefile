COMPILER = ../../../../target/release/aether-compiler
SOURCE = main.aes
OUTPUT = main
# Use absolute path for runtime or relative from the example dir
RUNTIME_PATH = $(shell pwd)/../../../../runtime/target/release

all: build

build:
	export AETHER_RUNTIME_PATH=$(RUNTIME_PATH) && $(COMPILER) compile $(SOURCE) -o $(OUTPUT)

run: build
	./$(OUTPUT)

clean:
	rm -f $(OUTPUT) $(OUTPUT).o