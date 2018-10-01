# Top-level build tool for this project. Use this file to do all things related to building, debugging, and loading code.
binary := target/thumbv7em-none-eabihf/debug/roboarm
.PHONY: all
all:
	cargo build

.PHONY: clean
clean:
	cargo clean

.PHONY: openocd
openocd:
	openocd -f target/ek-tm4c123gxl.cfg

.PHONY: gdb
gdb:
	arm-none-eabi-gdb -q target/thumbv7em-none-eabihf/debug/roboarm -ex "target remote :3333"
