# Basic Variables

PROJECT									?= $(CURDIR)/..

include $(PROJECT)/mk/setup-envs.mk

TARGET_PATH								:= $(CURDIR)/target/$(ARCH)/$(BUILD_MODE)
TARGET									:= $(TARGET_PATH)/jrinx

# Runner / Debugger Parameters

ARGS									?=
QEMU_GDB_PORT							?= 1234

QEMU									:= qemu-system-$(ARCH)
QEMU_FLAGS								:= -nographic -no-reboot
QEMU_FLAGS								+= -M $(BOARD) -smp $(SMP) -m $(MEMORY)
QEMU_FLAGS								+= -kernel $(TARGET)

ifneq ($(filter dbg,$(MAKECMDGOALS)),)
QEMU_FLAGS								+= -gdb tcp::$(QEMU_GDB_PORT) -S
endif

ifneq ($(ARGS),)
QEMU_FLAGS								+= -append "$(ARGS)"
endif

# Build Environment Variables / Flags

FEATURES								:= board_$(BOARD)

export CARGO_BUILD_TARGET				:= $(CURDIR)/tgt/$(ARCH).json

ifneq ($(VERBOSE),true)
export CARGO_TERM_QUIET					:= true
endif

CARGO_BUILD_FLAGS						:= --features $(FEATURES)

ifeq ($(BUILD_MODE),release)
CARGO_BUILD_FLAGS						+= --release
endif

# Make Goals

.PHONY: build
build:
	cargo build $(CARGO_BUILD_FLAGS)

.PHONY: clean
clean:
	cargo clean

.PHONY: run dbg
run:
	$(QEMU) $(QEMU_FLAGS)

dbg: run
