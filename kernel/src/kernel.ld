ENTRY(boot)

_RAM_START = 0x80000000;
_KERNEL_START = _RAM_START + 0x2000000;
_KERNEL_CODE_VIRTUAL = 0xffffffff80000000;
_KERNEL_VIRTUAL_RAM_START = _KERNEL_CODE_VIRTUAL + (_KERNEL_START - _RAM_START);

SECTIONS {
    . = _KERNEL_START;

    .boot.text : {
      __boot_text = .;
      . = ALIGN(16);
      KEEP(*(.boot.text));
      __boot_end = ABSOLUTE(.);
    }


    _KERNEL_VIRTUAL_CODE_START = _KERNEL_VIRTUAL_RAM_START + (__boot_end - _KERNEL_START);
    PROVIDE(_KOFF = _KERNEL_VIRTUAL_CODE_START - __boot_end);
    . = _KERNEL_VIRTUAL_CODE_START;

    .text ALIGN(4096) : AT(ADDR(.text) - _KOFF){
        __text = .;
        
        . = ALIGN(16);
        *(.text .text.*);

        . = ALIGN(4096);
        __text_end = .;
    }
    
    .rodata ALIGN(4096) : AT(ADDR(.rodata) - _KOFF){
        __rodata = .;
        *(.rodata .rodata.*);
        . = ALIGN(16);
        *(.srodata .srodata.*);

        KEEP(*(.symbols))

        . = ALIGN(4096);
        __rodata_end = .;
    }

    __data = .;
    .data ALIGN(4096) : AT(ADDR(.data) - _KOFF){
        . = ALIGN(16);
        PROVIDE(__global_pointer$ = . + 0x800);
        *(.data .data.*);
        . = ALIGN(16);
        *(.sdata .sdata.*);
    }

    .bss : AT(ADDR(.bss) - _KOFF){
        __bss = .;
        PROVIDE(__bss_phys = __bss - _KOFF);
        . = ALIGN(16);
        *(.bss .bss.*);
        . = ALIGN(16);
        *(.sbss .sbss.*);
        __bss_end = .;
        PROVIDE(__bss_end_phys = __bss_end - _KOFF);
        . = ALIGN(4096);
    }


    . = ALIGN(8);
    __stack_bottom = .;
    PROVIDE(__stack_bottom_phys = __stack_bottom - _KOFF);
    . += 128 * 1024; /* 128KB */
    __stack_top = .;
    PROVIDE(__stack_top_phys = __stack_top - _KOFF);

    . = ALIGN(4096);
    __data_end = .;
    __free_ram = .;
    __free_ram_phys = __free_ram - _KOFF;
    . += 64 * 1024 * 1024; /* 64MB */
    __free_ram_end = .;
    __free_ram_end_phys = __free_ram_end - _KOFF;
}
