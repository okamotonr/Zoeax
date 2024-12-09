set confirm off
set history save on
set print pretty on
target remote :7777
file target/riscv64gc-unknown-none-elf/debug/kernel
add-symbol-file target/riscv64gc-unknown-none-elf/debug/user
add-symbol-file target/riscv64gc-unknown-none-elf/debug/pong
