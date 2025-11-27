# Makefile for enum_methods example
COMPILER := ../../../../target/release/aether-compiler
RUNTIME_PATH := ../../../../runtime/target/release
TARGET := main
SOURCE := main.aes

.PHONY: all clean run

all: $(TARGET)

$(TARGET): $(SOURCE)
	AETHER_RUNTIME_PATH=$(RUNTIME_PATH) $(COMPILER) compile $(SOURCE) -o $(TARGET)

run: $(TARGET)
	./$(TARGET); echo "Exit code: $$?"

clean:
	rm -f $(TARGET)
