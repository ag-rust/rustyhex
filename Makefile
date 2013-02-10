RUSTC ?= rustc

#LOG_FLAGS ?= RUST_LOG=rustc::metadata::creader

all: rustyhex

run: all
	./rustyhex

rustyhex: main.rs *.rs
	$(LOG_FLAGS) $(RUSTC) -o $@ $<
