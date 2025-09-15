CC := gcc
CFLAGS := -std=c11 -O2 -Wall -Wextra -Wpedantic
BINARY := app

SOURCES := main.c linked_list.c

build: $(BINARY)

$(BINARY): $(SOURCES)
ifndef MAKECMDGOALS
	@:
endif
ifneq (,$(filter clean,$(MAKECMDGOALS)))
	@:
else
ifneq (,$(wildcard main.c))
ifneq (,$(wildcard linked_list.c))
	$(CC) $(CFLAGS) -o $@ main.c linked_list.c
else
	$(error Missing linked_list.c in current directory)
endif
else
	$(error Missing main.c in current directory)
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
	$(RM) $(BINARY) *.o

.PHONY: build task1 task2 task3 run clean

