CC ?= $(shell which cc)
CARGO ?= $(shell which cargo)
OS := $(shell uname)

run: clean build
	./target/release/embed_quantiles

clean:
	$(CARGO) clean
	rm -f ./target/release/embed_quantiles

build:
	$(CARGO) build --release
ifeq ($(OS),Darwin) # Assumption: CC == Clang
	$(CC) -std=c11 -Wall -Werror -Wpedantic -O3 c_src/main.c \
		-L target/release/ -l embed_quantiles \
		-o target/release/embed_quantiles
else # Assumption: CC == GCC
	$(CC) -std=c11 -Wall -Werror -Wpedantic -O3 c_src/main.c \
		-L target/release/ -lpthread -Wl,--no-as-needed -ldl -lm -lembed_quantiles \
		-o target/release/embed_quantiles
endif

.PHONY: clean run
