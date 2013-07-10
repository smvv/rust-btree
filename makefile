#all: libbtree.so

#all: btree_main
#btree_main: libbtree.so

all: btree

test: all
test: RUSTFLAGS += --test

clean:
	rm -f libbtree.so

RUSTC := rustc
RUSTFLAGS := -O -L.

lib%.so: %.rs
	rm -f lib$*-*.so
	$(RUSTC) $(RUSTFLAGS) --lib -o $@ $<
	touch $@

%: %.rs
	$(RUSTC) $(RUSTFLAGS) -o $@ $<
