all: libbtree.so btree_bench

btree_bench: libbtree.so

test: all
test: RUSTFLAGS += --test

clean:
	rm -f libbtree.so libbtree-*.so btree_bench

RUSTC := rustc
RUSTFLAGS := -O -L.

lib%.so: %.rs
	rm -f lib$*-*.so
	$(RUSTC) $(RUSTFLAGS) --lib -o $*.so $<
	touch $@

%: %.rs
	$(RUSTC) $(RUSTFLAGS) -o $@ $<
