RUSTC := rustc
RUSTFLAGS := -O -C debuginfo=0 -C overflow-checks=on
BINARY := app

# Assume all sources are at the working directory root after extraction
SOURCES := main.rs linked_list.rs

build: $(BINARY)

$(BINARY): $(SOURCES)
ifndef MAKECMDGOALS
	@:
endif
ifneq (,$(filter clean,$(MAKECMDGOALS)))
	@:
else
ifneq (,$(wildcard main.rs))
ifneq (,$(wildcard linked_list.rs))
	$(RUSTC) $(RUSTFLAGS) -o $@ main.rs
else
	$(error Missing linked_list.rs in current directory)
endif
else
	$(error Missing main.rs in current directory)
endif
endif

task1: build
	./$(BINARY) task1

task2: build
	./$(BINARY) task2

task3: build
	./$(BINARY) task3

run: build
	./$(BINARY) task1
	./$(BINARY) task2
	./$(BINARY) task3

clean:
	$(RM) $(BINARY)
	$(RM) -r target

.PHONY: build task1 task2 task3 run clean
