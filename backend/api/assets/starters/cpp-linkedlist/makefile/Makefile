# C++ LinkedList starter — Makefile
CXX      := g++
CXXFLAGS := -std=c++17 -O2 -Wall -Wextra -Wpedantic
BINARY   := app

# Resolve sources whether archives were flattened or kept in subfolders
MAIN_SRC := $(firstword $(wildcard main.cpp main/main.cpp spec/main.cpp))
LL_SRC   := $(firstword $(wildcard LinkedList.cpp memo/LinkedList.cpp spec/LinkedList.cpp))

# Allow headers from root and common subfolders
INCLUDES := -I. -Imemo -Ispec -Imain

build: $(BINARY)

$(BINARY): $(MAIN_SRC) $(LL_SRC)
	$(CXX) $(CXXFLAGS) $(INCLUDES) -o $@ $^

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
	$(RM) $(BINARY) *.o *.obj

.PHONY: build task1 task2 task3 run clean
