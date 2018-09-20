# Top-level build tool for this project. Use this file to do all things related to building, debugging, and loading code.
.PHONY: all
all:
	cargo build

.PHONY: clean
clean:
	cargo clean

.PHONY: openocd
openocd:
	@echo "TODO: This subcommand :P"  #openocd -f interface/stlink-v2-1.cfg -f target/stm32f3x.cfg

.PHONY: gdb
gdb:
	arm-none-eabi-gdb -q target/thumbv7em-none-eabihf/debug/roboarm -ex "target remote :3333"
