# Build Parameters

ARCH									?= riscv64
BOARD									?= virt
MODE									?= release
SMP										?= 5
MEMORY									?= 1G

TARGET_PATH								:= $(CURDIR)/target/$(ARCH)/$(MODE)
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

CARGO_BUILD_FLAGS						:= --features $(FEATURES)

ifeq ($(MODE),release)
CARGO_BUILD_FLAGS						+= --release
endif

# Make Goals

.PHONY: build
build: export BUILD_MODE=$(MODE)
build: export BUILD_TIME=$(shell date "+%Y-%m-%d %H:%M:%S %z")
build:
	cargo build $(CARGO_BUILD_FLAGS)

.PHONY: clean
clean:
	cargo clean

.PHONY: run dbg
run:
	$(QEMU) $(QEMU_FLAGS)

dbg: run
