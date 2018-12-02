.PHONY: all
all:
	cargo build

.PHONY: release
release:
	cargo build --release

.PHONY: clean
clean:
	cargo clean

.PHONY: openocd
openocd:
	openocd -f board/ek-tm4c123gxl.cfg

.PHONY: gdb
gdb:
	arm-none-eabi-gdb -q target/thumbv7em-none-eabihf/debug/roboarm -ex "target remote :3333"
