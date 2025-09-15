GO := go
BINARY := app

SOURCES := main.go linked_list.go

build: $(BINARY)

$(BINARY): $(SOURCES)
ifndef MAKECMDGOALS
	@:
endif
ifneq (,$(filter clean,$(MAKECMDGOALS)))
	@:
else
ifneq (,$(wildcard main.go))
ifneq (,$(wildcard linked_list.go))
	GO111MODULE=off $(GO) build -o $@ .
else
	$(error Missing linked_list.go in current directory)
endif
else
	$(error Missing main.go in current directory)
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

.PHONY: build task1 task2 task3 run clean
