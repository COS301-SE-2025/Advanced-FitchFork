JAVAC := javac
JAVA  := java
BINDIR := build

# Compile ONLY student files in the root (e.g., Main.java, LinkedList.java)
SOURCES := $(wildcard *.java)

# JaCoCo (paths provided by the runner image)
JACOCO_AGENT ?= /opt/jacoco/lib/jacocoagent.jar
JACOCO_CLI   ?= /opt/jacoco/lib/jacococli.jar
OUTPUT       ?= /output
COVERAGE_FILE := $(OUTPUT)/jacoco.exec
REPORT_DIR    := $(OUTPUT)/jacoco-report

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

# Code coverage: run tasks under JaCoCo, then emit a report and print the CSV
task4: build
	$(JAVA) -javaagent:$(JACOCO_AGENT)=destfile=$(COVERAGE_FILE) -cp $(BINDIR) Main task1
	$(JAVA) -javaagent:$(JACOCO_AGENT)=destfile=$(COVERAGE_FILE),append=true -cp $(BINDIR) Main task2
	$(JAVA) -javaagent:$(JACOCO_AGENT)=destfile=$(COVERAGE_FILE),append=true -cp $(BINDIR) Main task3
	@mkdir -p $(REPORT_DIR)
	$(JAVA) -jar $(JACOCO_CLI) report $(COVERAGE_FILE) \
		--classfiles $(BINDIR) \
		--sourcefiles . \
		--csv $(REPORT_DIR)/jacoco.csv \
		--html $(REPORT_DIR) \
		--xml $(REPORT_DIR)/jacoco.xml
	@cat $(REPORT_DIR)/jacoco.csv

run: build
	$(JAVA) -cp $(BINDIR) Main task1
	$(JAVA) -cp $(BINDIR) Main task2
	$(JAVA) -cp $(BINDIR) Main task3

clean:
	$(RM) -r $(BINDIR)

.PHONY: build task1 task2 task3 task4 run clean
