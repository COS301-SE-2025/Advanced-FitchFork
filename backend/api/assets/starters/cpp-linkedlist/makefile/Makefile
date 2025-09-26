# C++ LinkedList starter — Makefile with Valgrind + gcov coverage
CXX       := g++
CXXFLAGS  := -std=c++17 -O0 -g -Wall -Wextra -Wpedantic -fprofile-arcs -ftest-coverage
LDFLAGS   := -lgcov
BINARY    := app

VALGRIND  ?= valgrind
GCOV      ?= gcov

# Resolve sources whether archives were flattened or kept in subfolders
MAIN_SRC  := $(firstword $(wildcard main.cpp main/main.cpp spec/main.cpp))
LL_SRC    := $(firstword $(wildcard LinkedList.cpp memo/LinkedList.cpp spec/LinkedList.cpp))

# All candidate sources (root + common subfolders)
ALL_SRCS  := $(filter %.cpp, \
             $(MAIN_SRC) $(LL_SRC) \
             $(wildcard *.cpp) \
             $(wildcard main/*.cpp) \
             $(wildcard memo/*.cpp) \
             $(wildcard spec/*.cpp))

# Allow headers from root and common subfolders
INCLUDES  := -I. -Imemo -Ispec -Imain

# Where the runner expects outputs
OUTPUT    ?= /output
COV_DIR   := $(OUTPUT)/gcov-report

build: $(BINARY)

$(BINARY): $(ALL_SRCS)
	$(CXX) $(CXXFLAGS) $(INCLUDES) -o $@ $^ $(LDFLAGS)

task1: build
	./$(BINARY) task1

# Memory-leak check
task2: build
	$(VALGRIND) --leak-check=full ./$(BINARY) task2

# Memory-leak check
task3: build
	$(VALGRIND) --leak-check=full ./$(BINARY) task3

# Code coverage: run tasks to generate .gcda/.gcno, then emit gcov reports
task4: build
	./$(BINARY) task1
	./$(BINARY) task2
	./$(BINARY) task3
	@mkdir -p $(COV_DIR)
	# Generate gcov reports for all sources; write summary and move .gcov files
	@echo "=== GCOV SUMMARY ===" > $(COV_DIR)/summary.txt
	@$(GCOV) -o . $(ALL_SRCS) >> $(COV_DIR)/summary.txt 2>&1 || true
	@mv -f *.gcov $(COV_DIR) 2>/dev/null || true
	@echo "GCOV reports in $(COV_DIR)"
	@cat $(COV_DIR)/summary.txt

run: build
	./$(BINARY) task1
	./$(BINARY) task2
	./$(BINARY) task3

clean:
	$(RM) $(BINARY) *.o *.obj
	$(RM) -r *.gcda *.gcno *.gcov $(COV_DIR)

.PHONY: build task1 task2 task3 task4 run clean
