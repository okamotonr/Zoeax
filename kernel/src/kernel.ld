ENTRY(boot)

K_OFFSET = 0xffff800000000000;
PROVIDE(_RAM_START = 0x80000000);

SECTIONS {
    . = 0x80200000;

    .boot.text : {
      __boot_text = .;
      . = ALIGN(16);
      KEEP(*(.boot.text));
      __boot_end = .;
    }

    __current = .;

    . = K_OFFSET + __current;

    .text : AT(ADDR(.text) - K_OFFSET){
        __text = .;
        
        . = ALIGN(16);
        *(.text .text.*);

        . = ALIGN(4096);
        __text_end = .;
    }

    
    .rodata : AT(ADDR(.rodata) - K_OFFSET){
        __rodata = .;
        *(.rodata .rodata.*);
        . = ALIGN(16);
        *(.srodata .srodata.*);

        KEEP(*(.symbols))

        . = ALIGN(4096);
        __rodata_end = .;
    }

    __data = .;
    .data : AT(ADDR(.data) - K_OFFSET){
        . = ALIGN(16);
        *(.data .data.*);
        . = ALIGN(16);
        *(.sdata .sdata.*);
    }

    .bss : AT(ADDR(.data) - K_OFFSET){
        __bss = .;
        . = ALIGN(16);
        *(.bss .bss.*);
        . = ALIGN(16);
        *(.sbss .sbss.*);
        __bss_end = .;
        . = ALIGN(4096);
    }


    . = ALIGN(8);
    . += 128 * 1024; /* 128KB */
    __stack_top = .;

    . = ALIGN(4096);
    __data_end = .;
    PROVIDE(__free_ram = .);
    . += 64 * 1024 * 1024; /* 64MB */
    __free_ram_end = .;
}
