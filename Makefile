export MAKEFLAGS		:= -j$(shell nproc) -s $(MAKEFLAGS) -r

export ARCH				?= riscv64
export BOARD			?= virt
export BUILD_MODE		?= release
export BUILD_TIME		?= $(shell date "+%Y-%m-%d %H:%M:%S")
export RAND_SEED		?= $(shell echo $$RANDOM)
export SMP				?= 5
export MEMORY			?= 1G
export ARGS				?=

.PHONY: build
build:
	make -C kern build

.PHONY: clean
clean:
	make -C kern clean

.PHONY: run dbg
run:
	make -C kern run

dbg:
	make -C kern dbg
