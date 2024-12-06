
ENTRY(start)

_VA_OFFSET = 0xffff80000000000;

SECTIONS {
    . = 0xffff800001000000;

    .text ALIGN(4096): AT(ADDR(.text) - _VA_OFFSET){
        KEEP(*(.text.start));
        *(.text .text.*);
    }

    .rodata ALIGN(4096): AT(ADDR(.rodata) - _VA_OFFSET) {
        *(.rodata .rodata.*);
        . = ALIGN(16);
        *(.srodata .srodata.*);
        KEEP(*(.symbols));
    }

    .data ALIGN(4096): AT(ADDR(.data) - _VA_OFFSET) {
        *(.data .data.*);
        *(.sdata .sdata.*);
    }

    .bss ALIGN(4096): AT(ADDR(.bss) - _VA_OFFSET) {
        *(.bss .bss.);
        . = ALIGN(16);
        *(.sbss .sbss.*);

        . = ALIGN(16); /* https://github.com/nuta/operating-system-in-1000-lines/pull/23 */

        . += 128 * 1024;
        __stack_top = .;

        /*ASSERT(. - _VA_OFFSET < 0x1800000, "hoge large executable");*/
    }

}
