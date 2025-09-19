JAVAC := javac
JAVA  := java
BINDIR := build

# Compile ONLY student files in the root (e.g., Main.java, LinkedList.java)
SOURCES := $(wildcard *.java)

build: $(BINDIR)/.stamp

$(BINDIR)/.stamp: $(SOURCES)
	@mkdir -p $(BINDIR)
	$(JAVAC) -d $(BINDIR) $(SOURCES)
	@touch $@

task1: build
	$(JAVA) -cp $(BINDIR) Main task1

task2: build
	$(JAVA) -cp $(BINDIR) Main task2

task3: build
	$(JAVA) -cp $(BINDIR) Main task3

run: build
	$(JAVA) -cp $(BINDIR) Main task1
	$(JAVA) -cp $(BINDIR) Main task2
	$(JAVA) -cp $(BINDIR) Main task3

clean:
	$(RM) -r $(BINDIR)

.PHONY: build task1 task2 task3 run clean
