.PHONY: all
all:
	$(MAKE) -C roboarm
	$(MAKE) -C teleop
	$(MAKE) -C experiment

.PHONY: release
release:
	$(MAKE) release -C roboarm
	$(MAKE) release -C teleop
	$(MAKE) release -C experiment

.PHONY: roboarm
roboarm:
	$(MAKE) -C roboarm

.PHONY: teleop
teleop:
	$(MAKE) -C teleop

.PHONY: experiment
experiment:
	$(MAKE) -C experiment

.PHONY: clean
clean:
	$(MAKE) -C roboarm clean
	$(MAKE) -C teleop clean
	$(MAKE) -C experiment clean
	cargo clean

.PHONY: openocd
openocd:
	$(MAKE) -C roboarm openocd

.PHONY: gdb
gdb:
	$(MAKE) -C roboarm gdb

.PHONY: run-teleop
run-teleop:
	$(MAKE) -C teleop run

.PHONY: run
run:
	$(MAKE) -C experiment run

.PHONY: test
test:
	$(MAKE) -C experiment test

.PHONY: ci
ci: roboarm teleop experiment
	$(MAKE) -C experiment test

.PHONY: date
date:
	@echo %%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%
	@echo %%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%
	@echo %%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%
	@echo %%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%
	@echo %%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%
	@echo %%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%
	@echo %%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%
	@echo %%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%
	@echo %%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%
	@echo %%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%
