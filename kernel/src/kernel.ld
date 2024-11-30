ENTRY(boot)

SECTIONS {
    . = 0x80200000;
    PROVIDE(__kernel_base = .);

    .text :{
        __text = .;
        KEEP(*(.text.boot));

        
        . = ALIGN(16);
        *(.text .text.*);

        . = ALIGN(4096);
        __text_end = .;
    }

    
    .rodata :{
        __rodata = .;
        *(.rodata .rodata.*);
        . = ALIGN(16);
        *(.srodata .srodata.*);

        KEEP(*(.symbols))

        . = ALIGN(4096);
        __rodata_end = .;
    }

    __data = .;
    .data :{
        . = ALIGN(16);
        *(.data .data.*);
        . = ALIGN(16);
        *(.sdata .sdata.*);
    }

    .bss : {
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
    __free_ram = .;
    . += 64 * 1024 * 1024; /* 64MB */
    __free_ram_end = .;
}
