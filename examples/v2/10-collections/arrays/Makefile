AETHER = ../../../../target/release/aether-compiler

all: main

main: main.aes
	$(AETHER) compile main.aes

run: main
	./main

clean:
	rm -f main main.o main.s main.ll