.PHONY: all
all:
	$(MAKE) -C roboarm
	$(MAKE) -C teleop

.PHONY: roboarm
roboarm:
	$(MAKE) -C roboarm

.PHONY: teleop
teleop:
	$(MAKE) -C teleop

.PHONY: clean
clean:
	$(MAKE) -C roboarm clean
	$(MAKE) -C teleop clean

.PHONY: openocd
openocd:
	$(MAKE) -C roboarm openocd

.PHONY: gdb
gdb:
	$(MAKE) -C roboarm gdb
