TARGET := riscv64gc-unknown-none-elf
MODE := release
APP_DIR := src/bin
TARGET_DIR := target/$(TARGET)/$(MODE)
APPS := $(wildcard $(APP_DIR)/*.rs)

build: $(APPS)
	@cargo build --release

clean:
	@cargo clean

.PHONY: build clean
