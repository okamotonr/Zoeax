#!/bin/bash
set -xue

# QEMUのファイルパス
QEMU=qemu-system-riscv64
KERNEL=target/riscv64gc-unknown-none-elf/debug/kernel
USER=target/riscv64gc-unknown-none-elf/debug/user
OBJCOPY=llvm-objcopy

CFLAGS="-std=c11 -O2 -g3 -Wall -Wextra -ffreestanding -nostdlib"
(cd user && cargo build ) 
cp $USER kernel/shell
# $OBJCOPY --set-section-flags .bss=alloc,contents -O binary $USER shell.bin
# $OBJCOPY -Ibinary -Oelf64-littleriscv shell.bin shell.bin.o.single_float
# cp -f shell.bin.o.single_float shell.bin.o && printf '\x05\x00\x00\x00' | dd of=shell.bin.o bs=1 seek=48 conv=notrunc
# cp shell.bin.o kernel/

pushd kernel
cargo build #--release
popd
# QEMUを起動
$QEMU -machine virt -bios default -nographic -serial mon:stdio --no-reboot \
  -drive id=drive0,file=lorem.txt,format=raw,if=none \
  -device virtio-blk-device,drive=drive0,bus=virtio-mmio-bus.0 \
  -d unimp,guest_errors,int,cpu_reset -D qemu.log \
  -kernel $KERNEL
