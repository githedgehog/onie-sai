TARGETS := $(patsubst %.asm,%.hex,$(wildcard *.asm))

.PHONY: all
all: $(TARGETS)

.NOTPARALLEL: install
.PHONY: clean
clean:
	rm -f *.c *.hex *.lst

%.hex: %.asm
	ledasm $(basename $@)
