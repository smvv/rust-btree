all: btree

test: all
test: RUSTFLAGS += --test

clean:
	rm -f btree

RUSTC := rustc
RUSTFLAGS := -O -L.

lib%.so: %.rs
	rm -f lib$*-*.so
	$(RUSTC) $(RUSTFLAGS) --lib -o $*.so $<
	touch $@

%: %.rs
	$(RUSTC) $(RUSTFLAGS) -o $@ $<
