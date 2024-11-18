#!/bin/bash
set -xue

# QEMUのファイルパス
QEMU=qemu-system-riscv64
KERNEL=target/riscv64gc-unknown-none-elf/debug/mios

CFLAGS="-std=c11 -O2 -g3 -Wall -Wextra -ffreestanding -nostdlib"
cargo build #--release
# QEMUを起動
$QEMU -machine virt -bios default -nographic -serial mon:stdio --no-reboot \
  -d unimp,guest_errors,int,cpu_reset -D qemu.log \
        -kernel $KERNEL
