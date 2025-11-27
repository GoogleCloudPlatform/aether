COMPILER = ../../../../target/debug/aether-compiler
SOURCE = async_io.aether
OUTPUT = async_io
RUNTIME_PATH = $(shell pwd)/../../../../runtime/target/debug

all: build

build:
	export AETHER_RUNTIME_PATH=$(RUNTIME_PATH) && $(COMPILER) compile $(SOURCE) -o $(OUTPUT)

run: build
	./$(OUTPUT)

clean:
	rm -f $(OUTPUT) $(OUTPUT).o
