export PROJECT			:= $(CURDIR)
export MAKEFLAGS		:= -j$(shell nproc) -s $(MAKEFLAGS) -r

include $(PROJECT)/mk/setup-envs.mk

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
