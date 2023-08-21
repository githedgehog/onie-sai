/* DO NOT replace this file with what you found in the SDK! */
OUTPUT_FORMAT("elf32-littlearm", "elf32-littlearm", "elf32-littlearm")
OUTPUT_ARCH(arm)
ENTRY(custom_led_handler)
SECTIONS
{
    . = 0x3800;

    /* The run-time address could be specified with '-Ttext' in LDFLAGS */
    . = ALIGN(4);
    .text :
    {
        *(.text.custom_led_handler)
        *(.text*)
        *(.rodata*)
    }

    /* The run-time address could be specified with '-Tbss' in LDFLAGS */
    . = ALIGN(4);
    .bss :
    {
        *(.bss*)
    }

    /* The run-time address could be specified with '-Tdata' in LDFLAGS */
    . = ALIGN(4);
    .data :
    {
        *(.data*)
    }

    /DISCARD/ : { *(.dynstr*) }
    /DISCARD/ : { *(.dynamic*) }
    /DISCARD/ : { *(.plt*) }
    /DISCARD/ : { *(.interp*) }
    /DISCARD/ : { *(.gnu*) }
}
