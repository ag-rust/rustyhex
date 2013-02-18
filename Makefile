RUSTC ?= rustc

#LOG_FLAGS ?= RUST_LOG=rustc::metadata::creader
RUST_ENV = "--cfg image"
all: rustyhex

run: all
	./rustyhex

rustyhex: main.rs *.rs
	$(LOG_FLAGS) $(RUST_FLAGS) $(RUSTC) -o $@ $<
