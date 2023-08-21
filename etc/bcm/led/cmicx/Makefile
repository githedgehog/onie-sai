TOP     := $(shell pwd)

SDK     ?= $(TOP)/../sdk-build/sdk
CROSS   ?= arm-none-eabi
AS      := $(CROSS)-as
CC      := $(CROSS)-gcc
LD      := $(CROSS)-ld
NM      := $(CROSS)-nm
OBJDUMP := $(CROSS)-objdump
OBJCOPY := $(CROSS)-objcopy

CFLAGS  := -Os -g -mthumb -mcpu=cortex-m0 -ffunction-sections -I$(SDK)/include
LDFLAGS := -T custom_led.lds -Bstatic -gc-sections -nostdlib
LDFLAGS += -Ttext 0x3800

TARGETS := $(patsubst %.c,%,$(wildcard *.c))

all: $(TARGETS)

$(TARGETS):
	$(CC) $(CFLAGS) -c $@.c -o $@.o
	$(LD) $(LDFLAGS) -Map $@.map -o $@.elf $@.o $(LIBOBJS)
	$(OBJCOPY) -O binary $@.elf $@.bin
	$(OBJDUMP) -D $@.elf > $@.S
	$(NM) -S $@.elf > $@.nm
	@echo ""
	@echo " CMICX LEDuP Memory Layout"
	@echo "----------------------------------------------------"
	@cat $@.map | grep -E "^.text|^.data|^.bss"
	@echo ""
#	@echo " CMICX LEDuP Variables in BSS/DATA sections"
#	@echo "----------------------------------------------------"
#	@cat $@.nm | grep -E ' [Bb] | [Dd] '
#	@echo ""
#	@echo " CMICX LEDuP Binary in C Array format"
#	@echo "----------------------------------------------------"
#	@echo ""
#	@xxd -i -c 8 $@.bin

clean:
	rm -f *.elf *.o *.map *.bin *.S *.nm
