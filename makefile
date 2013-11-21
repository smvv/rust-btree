all: libbtree.so btree_bench
all: btree

btree_bench: libbtree.so
btree: RUSTFLAGS += --test

clean:
	rm -f libbtree.so libbtree-*.so btree_bench

docs:
	rustdoc -o doc/ btree.rs

RUSTC := rustc
RUSTFLAGS := -O -L.

lib%.so: %.rs
	rm -f lib$*-*.so
	$(RUSTC) $(RUSTFLAGS) --lib -o $*.so $<
	touch $@

%: %.rs
	$(RUSTC) $(RUSTFLAGS) -o $@ $<
