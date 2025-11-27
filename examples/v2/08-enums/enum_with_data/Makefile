AETHER_COMPILER = ../../../../target/release/aether-compiler
AETHER_RUNTIME = ../../../../runtime/target/release

.PHONY: all clean run

all: main

main: main.aes
	AETHER_RUNTIME_PATH=$(AETHER_RUNTIME) $(AETHER_COMPILER) compile $< -o $@

run: main
	./main

clean:
	rm -f main *.ll *.o
